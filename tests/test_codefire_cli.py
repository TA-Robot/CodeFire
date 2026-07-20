import json
import hashlib
import hmac
import contextlib
import gc
import io
import os
import runpy
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"
INSTALL = ROOT / "install.sh"


class CodeFireCliTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.processes = []

    def tearDown(self):
        for proc in self.processes:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
            if proc.stdout:
                proc.stdout.close()
            if proc.stderr:
                proc.stderr.close()
        shutil.rmtree(self.tmp)

    def run_cf(self, *args, cwd=None, check=True, env=None):
        proc_env = os.environ.copy()
        if env:
            proc_env.update(env)
        proc = subprocess.run(
            [str(CODEFIRE), *args],
            cwd=cwd or self.tmp,
            env=proc_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if check and proc.returncode != 0:
            self.fail(f"codefire {' '.join(args)} failed\nstdout={proc.stdout}\nstderr={proc.stderr}")
        return proc

    def exchange_http_handler(
        self, namespace, profile, request, configure_handler=None
    ):
        observations = []
        handler = namespace["make_http_handler"](
            self.tmp / "bounded-http",
            profile,
            _decision_work_observer=observations.append,
        )
        if configure_handler is not None:
            configure_handler(handler)
        server = namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        thread = threading.Thread(target=server.handle_request, daemon=True)
        thread.start()
        try:
            with socket.create_connection(server.server_address, timeout=5) as client:
                client.settimeout(5)
                client.sendall(request)
                client.shutdown(socket.SHUT_WR)
                chunks = []
                while True:
                    chunk = client.recv(64 * 1024)
                    if not chunk:
                        break
                    chunks.append(chunk)
        finally:
            server.server_close()
            thread.join(timeout=5)
        self.assertFalse(thread.is_alive(), "bounded HTTP handler did not terminate")
        self.assertEqual(len(observations), 1)
        return b"".join(chunks), observations[0]

    def assert_http_work_closed(self, observation):
        snapshot = observation["snapshot"]
        self.assertIsNotNone(snapshot)
        self.assertTrue(all(value == 0 for value in snapshot["current"].values()))
        self.assertEqual(observation["open_permit_count"], 0)

    def object_digest(self, type_tag, payload):
        data = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
        h = hashlib.sha256()
        h.update(type_tag.encode("utf-8"))
        h.update(b"\0")
        h.update(data)
        return h.hexdigest()

    def remote_object_id(self, type_tag, payload):
        digest = self.object_digest(type_tag, payload)
        prefixes = {"commit": "CF-COMMIT", "policy": "CF-POLICY"}
        return f"{prefixes.get(type_tag, 'CF-OBJECT')}-{digest[:24]}"

    def write_demo_files(self, open_dir: Path, minutes: int = 30):
        (open_dir / "docs/spec").mkdir(parents=True, exist_ok=True)
        (open_dir / "docs/design").mkdir(parents=True, exist_ok=True)
        (open_dir / "src").mkdir(exist_ok=True)
        (open_dir / "tests").mkdir(exist_ok=True)
        (open_dir / "docs/spec/auth.md").write_text(
            "## REQ-AUTH-001: Session expiration\n\n"
            f"Sessions expire after {minutes} minutes.\n",
            encoding="utf-8",
        )
        (open_dir / "docs/design/auth.md").write_text(
            "## DES-AUTH-001: Session policy\n\n"
            f"SessionPolicy returns {minutes} minutes.\n",
            encoding="utf-8",
        )
        (open_dir / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            f"        return {minutes}\n",
            encoding="utf-8",
        )
        (open_dir / "tests/test_auth.py").write_text(
            "from src.auth import SessionPolicy\n\n"
            "# cf-atom: TEST-session-expiration\n"
            "def test_session_expiration():\n"
            f"    assert SessionPolicy().expires_after_minutes() == {minutes}\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - path: docs/spec/**/*.md\n"
            "      kind: requirement_document\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: REQ-AUTH-001\n"
            "    to: DES-AUTH-001\n"
            "    type: refined_by\n"
            "  - from: DES-AUTH-001\n"
            "    to: CODE-SessionPolicy\n"
            "    type: implemented_by\n"
            "  - from: REQ-AUTH-001\n"
            "    to: TEST-session-expiration\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: true\n"
            "verification:\n"
            "  required:\n"
            "    - id: unit-tests\n"
            "      command: \"python3 -m unittest discover -s tests\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )

    def commit_demo_version(self, open_dir: Path, minutes: int, message: str):
        self.write_demo_files(open_dir, minutes)
        scan = self.run_cf("scan", cwd=open_dir)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", message, cwd=open_dir)
        self.run_cf("verify", cwd=open_dir)
        self.run_cf("commit", "-m", message, cwd=open_dir)

    def generate_self_signed_cert(self):
        cert = self.tmp / "codefire-test.crt"
        key = self.tmp / "codefire-test.key"
        proc = subprocess.run(
            [
                "openssl",
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-keyout",
                str(key),
                "-out",
                str(cert),
                "-days",
                "1",
                "-subj",
                "/CN=127.0.0.1",
                "-addext",
                "subjectAltName=IP:127.0.0.1",
            ],
            cwd=self.tmp,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return cert, key

    def start_http_remote(self, tls=False, storage_root=None):
        storage = storage_root or self.tmp / (
            "https-storage" if tls else "http-storage"
        )
        cmd = [str(CODEFIRE), "serve", str(storage), "--host", "127.0.0.1", "--port", "0"]
        marker = "cf+http://"
        if tls:
            cert, key = self.generate_self_signed_cert()
            cmd.extend(["--tls-cert", str(cert), "--tls-key", str(key)])
            marker = "cf+https://"
        proc = subprocess.Popen(
            cmd,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.processes.append(proc)
        line = proc.stdout.readline()
        if proc.poll() is not None:
            self.fail(f"codefire serve exited early\nstdout={line}\nstderr={proc.stderr.read()}")
        if marker not in line:
            self.fail(f"unexpected codefire serve output: {line}")
        return line[line.index(marker) :].strip()

    def replace_main_head_with_mutated_commit(self, mutate):
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        mutate(bad_payload)
        bad_commit_id = self.remote_object_id("commit", bad_payload)
        bad_record = {
            "object_id": bad_commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", bad_payload),
            "payload": bad_payload,
        }
        bad_path = self.tmp / ".codefire/objects/commits" / f"{bad_commit_id}.json"
        bad_path.write_text(json.dumps(bad_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return bad_commit_id, bad_payload

    def write_commit_object(self, payload):
        commit_id = self.remote_object_id("commit", payload)
        record = {
            "object_id": commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", payload),
            "payload": payload,
        }
        path = self.tmp / ".codefire/objects/commits" / f"{commit_id}.json"
        path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return commit_id

    def write_commit_object_to(self, objects_root: Path, payload):
        commit_id = self.remote_object_id("commit", payload)
        record = {
            "object_id": commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", payload),
            "payload": payload,
        }
        path = objects_root / "commits" / f"{commit_id}.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return commit_id

    def object_record_for(self, objects_root: Path, oid: str):
        matches = list(objects_root.glob(f"*/*{oid}.json"))
        self.assertEqual(len(matches), 1, oid)
        record = json.loads(matches[0].read_text(encoding="utf-8"))
        return {
            "object_id": record["object_id"],
            "type": record["type"],
            "hash": record["hash"],
            "payload": record["payload"],
        }

    def reachable_object_records(self, objects_root: Path, root_oid: str):
        records = []
        seen = set()
        stack = [root_oid]
        while stack:
            oid = stack.pop()
            if oid in seen:
                continue
            seen.add(oid)
            record = self.object_record_for(objects_root, oid)
            records.append(record)
            payload = record["payload"]
            if payload.get("type") == "commit":
                stack.extend(parent for parent in payload.get("parents", []) if isinstance(parent, str))
                stack.extend(value for value in payload.get("roots", {}).values() if isinstance(value, str))
            elif payload.get("type") == "content_manifest":
                stack.extend(entry["blob"] for entry in payload.get("entries", []) if isinstance(entry, dict) and entry.get("blob"))
        return sorted(records, key=lambda record: record["object_id"])

    def remote_request_signature(
        self,
        actor,
        operation,
        target,
        payload,
        key_id,
        key,
        nonce="fixed-nonce",
        timestamp=None,
    ):
        unsigned_payload = dict(payload)
        unsigned_payload.pop("request_signature", None)
        payload_digest = "sha256:" + hashlib.sha256(
            json.dumps(
                unsigned_payload,
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest()
        signature = {
            "type": "remote_request_signature",
            "version": 2,
            "algorithm": "hmac-sha256",
            "subject_type": "codefire_remote_mutation_subject",
            "subject_version": 1,
            "actor": actor,
            "operation": operation,
            "target": target,
            "key_id": key_id,
            "timestamp": timestamp or time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "nonce": nonce,
            "payload_digest": payload_digest,
            "idempotency_key": payload.get("idempotency_key") or None,
        }
        subject = {
            "type": signature["subject_type"],
            "version": signature["subject_version"],
            "operation": operation,
            "actor": actor,
            "target": target,
            "payload_digest": payload_digest,
            "idempotency_key": signature["idempotency_key"],
            "key_id": key_id,
            "timestamp": signature["timestamp"],
            "nonce": nonce,
        }
        signature["subject_digest"] = "sha256:" + hashlib.sha256(
            json.dumps(subject, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        data = json.dumps(signature, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
        signature["signature"] = hmac.new(key.encode("utf-8"), data, hashlib.sha256).hexdigest()
        return signature

    def test_python_common_ancestor_prefers_generation_before_public_id(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))

        low = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "generation": 1,
            "label": "low-generation",
        }
        low_id = self.remote_object_id("commit", low)
        for index in range(100):
            high = {
                "type": "commit",
                "version": 1,
                "parents": [],
                "generation": 2,
                "label": f"high-generation-{index}",
            }
            high_id = self.remote_object_id("commit", high)
            if low_id < high_id:
                break
        else:
            self.fail("could not construct a generation-vs-ID tie-break fixture")

        self.assertEqual(self.write_commit_object(low), low_id)
        self.assertEqual(self.write_commit_object(high), high_id)
        left = self.write_commit_object(
            {
                "type": "commit",
                "version": 1,
                "parents": [low_id, high_id],
                "generation": 3,
                "label": "left",
            }
        )
        right = self.write_commit_object(
            {
                "type": "commit",
                "version": 1,
                "parents": [high_id, low_id],
                "generation": 3,
                "label": "right",
            }
        )

        self.assertEqual(namespace["common_ancestor"](self.tmp, left, right), high_id)

    def test_python_decision_work_typed_exit_60_matches_rust_golden_schema(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].local_v1(
            {"graph_nodes": 2}
        )
        ledger = namespace["DecisionWorkLedger"](profile, "trace golden")
        ledger.charge("graph_nodes", 2, "graph_discovery")
        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            ledger.charge("graph_nodes", 1, "graph_discovery")
        expected = {
            "kind": "resource_budget_exceeded",
            "exit_code": 60,
            "profile": "codefire.decision-work.v1",
            "dimension": "graph_nodes",
            "observed": 3,
            "limit": 2,
            "phase": "graph_discovery",
            "command": "trace golden",
            "continuation_supported": False,
        }
        self.assertEqual(raised.exception.diagnostic(), expected)
        self.assertEqual(raised.exception.exit_code, 60)
        self.assertEqual(ledger.snapshot()["terminal_reason"], "budget_exceeded")

        class FailingParser:
            @staticmethod
            def parse_args(_argv):
                class Args:
                    @staticmethod
                    def func(_args):
                        fresh = namespace["DecisionWorkLedger"](profile, "trace golden")
                        fresh.charge("graph_nodes", 3, "graph_discovery")

                return Args()

        original = namespace["main"].__globals__["build_parser"]
        namespace["main"].__globals__["build_parser"] = lambda: FailingParser()
        stderr = io.StringIO()
        try:
            with contextlib.redirect_stderr(stderr):
                exit_code = namespace["main"]([])
        finally:
            namespace["main"].__globals__["build_parser"] = original
        self.assertEqual(exit_code, 60)
        self.assertEqual(json.loads(stderr.getvalue()), expected)

    def test_python_decision_work_trace_node_limit_minus_exact_plus_one(self):
        namespace = runpy.run_path(str(CODEFIRE))
        limit = 3
        for count, succeeds in [(limit - 1, True), (limit, True), (limit + 1, False)]:
            atom_index = {
                "atoms": [
                    {"atom_id": f"REQ-{index}", "kind": "requirement"}
                    for index in range(count)
                ]
            }
            profile = namespace["DecisionWorkProfile"].local_v1(
                {"graph_nodes": limit}
            )
            ledger = namespace["DecisionWorkLedger"](
                profile, "trace node boundary"
            )
            if succeeds:
                index = namespace["compile_trace_evaluation_index"](
                    atom_index, {"links": []}, ledger
                )
                self.assertEqual(
                    ledger.snapshot()["cumulative"]["graph_nodes"], count
                )
                self.assertEqual(
                    ledger.snapshot()["counters"]["atoms_indexed"], count
                )
                index.close()
                self.assertEqual(
                    ledger.snapshot()["current"].get("retained_bytes", 0), 0
                )
            else:
                with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                    namespace["compile_trace_evaluation_index"](
                        atom_index, {"links": []}, ledger
                    )
                diagnostic = raised.exception.diagnostic()
                self.assertEqual(diagnostic["dimension"], "graph_nodes")
                self.assertEqual(diagnostic["observed"], limit + 1)
                self.assertEqual(diagnostic["limit"], limit)
                self.assertEqual(diagnostic["phase"], "graph_discovery")
                self.assertEqual(
                    ledger.snapshot()["cumulative"]["graph_nodes"], limit
                )
                self.assertEqual(
                    ledger.snapshot()["counters"]["atoms_indexed"], limit
                )

        counter_profile = namespace["DecisionWorkProfile"].local_v1(
            {"counter_values": ("maximum", 2)}
        )
        counter_ledger = namespace["DecisionWorkLedger"](
            counter_profile, "trace atomic reservation"
        )
        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            namespace["compile_trace_evaluation_index"](
                {
                    "atoms": [
                        {"atom_id": f"REQ-{index}", "kind": "requirement"}
                        for index in range(3)
                    ]
                },
                {"links": []},
                counter_ledger,
            )
        self.assertEqual(raised.exception.diagnostic()["dimension"], "counter_values")
        self.assertEqual(
            counter_ledger.snapshot()["cumulative"]["graph_nodes"],
            2,
            "the failed counter reservation must not partially charge its graph node",
        )

    def test_python_decision_work_profile_order_maximum_and_permit_contract(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].local_v1(
            {
                "graph_nodes": 0,
                "graph_edges": 0,
                "retained_bytes": 16,
            }
        )
        with self.assertRaises(TypeError):
            profile.limits["graph_nodes"] = ("cumulative", 1)
        ledger = namespace["DecisionWorkLedger"](profile, "canonical order")
        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            ledger.apply(
                "graph_discovery",
                charges={"graph_edges": 1, "graph_nodes": 1},
            )
        self.assertEqual(raised.exception.diagnostic()["dimension"], "graph_nodes")

        maximum_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "maximum delta"
        )
        maximum_ledger.observe_maximum("logical_depth", 5, "validation")
        maximum_ledger.observe_maximum("logical_depth", 3, "query")
        maximum_ledger.observe_maximum("logical_depth", 8, "query")
        snapshot = maximum_ledger.snapshot()
        self.assertEqual(snapshot["phases"]["validation"]["logical_depth"], 5)
        self.assertEqual(snapshot["phases"]["query"]["logical_depth"], 3)

        permit_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(
                {"retained_bytes": 16}
            ),
            "publication permit",
        )
        permit = permit_ledger.reserve(
            "publication_preflight", acquires={"retained_bytes": 16}
        )
        permit_ledger.seal_prepublication()
        with self.assertRaises(namespace["CFError"]):
            permit_ledger.acquire("retained_bytes", 1, "publication_preflight")
        permit_ledger.consume_reserved(
            permit,
            phase="publication_preflight",
            acquires={"retained_bytes": 8},
        )
        permit_ledger.release("retained_bytes", 8)
        permit_ledger.cancel_permit(permit)
        permit_ledger.terminate("complete")
        self.assertEqual(
            permit_ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

    def test_python_decision_work_opaque_owner_and_immutable_state(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].local_v1()
        with self.assertRaises(AttributeError):
            profile.profile_id = "mutated"
        local = namespace["DecisionWorkLedger"](profile, "opaque local")
        other = namespace["DecisionWorkLedger"](profile, "opaque other")
        permit = local.reserve(
            "publication_preflight", acquires={"retained_bytes": 8}
        )
        before_local = local.snapshot()
        before_other = other.snapshot()
        with self.assertRaises(namespace["CFError"]):
            other.consume_reserved(
                permit,
                phase="publication_preflight",
                acquires={"retained_bytes": 1},
            )
        self.assertEqual(local.snapshot(), before_local)
        self.assertEqual(other.snapshot(), before_other)
        with self.assertRaises(AttributeError):
            permit.any_field = 1
        with self.assertRaises(TypeError):
            local.current["retained_bytes"] = 9
        with self.assertRaises(AttributeError):
            local.profile = profile
        with self.assertRaises(namespace["CFError"]):
            local.apply("encoding", counters={"response_bytes_encoded": 1})
        local.cancel_permit(permit)

    def test_python_decision_work_permit_supports_partial_consumption(self):
        namespace = runpy.run_path(str(CODEFIRE))
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(
                {"retained_bytes": 10}
            ),
            "partial permit",
        )
        permit = ledger.reserve(
            "publication_preflight", acquires={"retained_bytes": 10}
        )
        ledger.consume_reserved(
            permit,
            phase="publication_preflight",
            acquires={"retained_bytes": 4},
        )
        with self.assertRaises(namespace["ResourceBudgetExceeded"]):
            ledger.acquire("retained_bytes", 1, "publication_preflight")
        ledger.release("retained_bytes", 4)
        # The capacity failure is terminal by contract; use a fresh ledger to
        # prove the same opaque permit remains consumable in multiple steps.
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(
                {"retained_bytes": 10}
            ),
            "partial permit success",
        )
        permit = ledger.reserve(
            "publication_preflight", acquires={"retained_bytes": 10}
        )
        for amount in [4, 6]:
            ledger.consume_reserved(
                permit,
                phase="publication_preflight",
                acquires={"retained_bytes": amount},
            )
            ledger.release("retained_bytes", amount)
        ledger.terminate("complete")
        self.assertEqual(ledger.snapshot()["current"].get("retained_bytes", 0), 0)

    def test_python_decision_work_rejects_phase_u64_overflow_atomically(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].local_v1(
            {"retained_bytes": ("concurrent", namespace["U64_MAX"])}
        )
        ledger = namespace["DecisionWorkLedger"](profile, "phase overflow")
        ledger.acquire(
            "retained_bytes", namespace["U64_MAX"], "query"
        )
        ledger.release("retained_bytes", namespace["U64_MAX"])
        before = ledger.snapshot()

        with self.assertRaisesRegex(namespace["CFError"], "phase accounting overflow"):
            ledger.acquire("retained_bytes", 1, "query")

        self.assertEqual(ledger.snapshot(), before)

    def test_python_decision_work_u64_max_overflow_saturates_diagnostic_atomically(self):
        namespace = runpy.run_path(str(CODEFIRE))
        maximum = namespace["U64_MAX"]
        profile = namespace["DecisionWorkProfile"].local_v1(
            {"counter_values": ("maximum", maximum)}
        )
        ledger = namespace["DecisionWorkLedger"](profile, "u64 saturation")
        ledger.count("files_read", maximum, "query")
        before = ledger.snapshot()

        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            ledger.count("files_read", 1, "query")

        after = ledger.snapshot()
        self.assertEqual(raised.exception.diagnostic()["observed"], maximum)
        self.assertEqual(after["counters"], before["counters"])
        self.assertEqual(after["phase_counters"], before["phase_counters"])
        self.assertEqual(after["terminal_reason"], "budget_exceeded")
        self.assertEqual(after["current"].get("retained_bytes", 0), 0)
        self.assertEqual(after["current"].get("temporary_bytes", 0), 0)

    def test_python_decision_work_u64_maximum_input_overflow_is_atomic(self):
        namespace = runpy.run_path(str(CODEFIRE))
        maximum = namespace["U64_MAX"]
        profile = namespace["DecisionWorkProfile"].local_v1(
            {"logical_depth": ("maximum", maximum)}
        )
        ledger = namespace["DecisionWorkLedger"](profile, "maximum u64 domain")
        before = ledger.snapshot()

        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            ledger.observe_maximum("logical_depth", maximum + 1, "query")

        after = ledger.snapshot()
        diagnostic = raised.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "logical_depth")
        self.assertEqual(diagnostic["observed"], maximum)
        self.assertEqual(diagnostic["limit"], maximum)
        self.assertEqual(after["cumulative"], before["cumulative"])
        self.assertEqual(after["maximum"], before["maximum"])
        self.assertEqual(after["current"], before["current"])
        self.assertEqual(after["counters"], before["counters"])
        self.assertEqual(after["phases"], before["phases"])
        self.assertEqual(after["phase_counters"], before["phase_counters"])
        self.assertEqual(after["terminal_reason"], "budget_exceeded")

    def test_python_http_request_header_boundaries_are_request_wide(self):
        namespace = runpy.run_path(str(CODEFIRE))
        limit = 256
        profile = namespace["DecisionWorkProfile"].http_v1(
            {"request_header_bytes": limit}
        )
        prefix = b"GET /v1/projects/org/app/nope HTTP/1.0\r\nX-Fill: "
        suffix = b"\r\n\r\n"

        for size in [limit - 1, limit, limit + 1]:
            request = prefix + (b"x" * (size - len(prefix) - len(suffix))) + suffix
            response, observation = self.exchange_http_handler(
                namespace, profile, request
            )
            self.assert_http_work_closed(observation)
            if size <= limit:
                self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))
                self.assertEqual(
                    observation["snapshot"]["cumulative"][
                        "request_header_bytes"
                    ],
                    size,
                )
                self.assertIsNone(observation["budget_diagnostic"])
            else:
                self.assertTrue(
                    response.startswith(b"HTTP/1.0 413 Request Entity Too Large\r\n")
                )
                diagnostic = observation["budget_diagnostic"]
                self.assertEqual(diagnostic["kind"], "resource_budget_exceeded")
                self.assertEqual(diagnostic["exit_code"], 60)
                self.assertEqual(diagnostic["dimension"], "request_header_bytes")
                self.assertEqual(diagnostic["observed"], limit + 1)
                self.assertEqual(diagnostic["limit"], limit)

    def test_python_http_body_boundaries_reserve_before_read(self):
        namespace = runpy.run_path(str(CODEFIRE))
        limit = 64
        profile = namespace["DecisionWorkProfile"].http_v1(
            {"request_body_bytes": limit}
        )
        target = b"POST /v1/projects/org/app/nope HTTP/1.0\r\n"

        for size in [limit - 1, limit, limit + 1]:
            body = b'"' + (b"x" * (size - 2)) + b'"'
            request = (
                target
                + f"Content-Length: {size}\r\n\r\n".encode("ascii")
                + body
            )
            response, observation = self.exchange_http_handler(
                namespace, profile, request
            )
            self.assert_http_work_closed(observation)
            if size <= limit:
                self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))
                self.assertEqual(observation["request_body_bytes_read"], size)
                self.assertEqual(
                    observation["snapshot"]["cumulative"][
                        "request_body_bytes"
                    ],
                    size,
                )
                self.assertGreaterEqual(
                    observation["snapshot"]["peak"]["temporary_bytes"],
                    size * 2,
                )
            else:
                self.assertTrue(
                    response.startswith(b"HTTP/1.0 413 Request Entity Too Large\r\n")
                )
                self.assertEqual(observation["request_body_bytes_read"], 0)
                diagnostic = observation["budget_diagnostic"]
                self.assertEqual(diagnostic["dimension"], "request_body_bytes")
                self.assertEqual(diagnostic["observed"], limit + 1)

    def test_python_http_rejects_bad_content_length_before_body_read(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].http_v1()
        target = b"POST /v1/projects/org/app/nope HTTP/1.0\r\n"
        cases = [
            b"Content-Length: nope\r\n",
            b"Content-Length: -1\r\n",
            f"Content-Length: {namespace['U64_MAX'] + 1}\r\n".encode("ascii"),
            b"Content-Length: " + (b"9" * 5000) + b"\r\n",
            b"Content-Length: 2\r\nContent-Length: 2\r\n",
            b"Transfer-Encoding: chunked\r\n",
            b"Transfer-Encoding:\r\n",
            b"Transfer-Encoding:   \r\n",
        ]

        for headers in cases:
            response, observation = self.exchange_http_handler(
                namespace, profile, target + headers + b"\r\n{}"
            )
            self.assertTrue(response.startswith(b"HTTP/1.0 400 Bad Request\r\n"))
            self.assertEqual(observation["request_body_bytes_read"], 0)
            self.assertIsNone(observation["budget_diagnostic"])
            self.assert_http_work_closed(observation)

    def test_python_http_response_body_is_all_or_typed_budget_error(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_list"]
        limit = 1024
        request = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
        try:
            for size in [limit - 1, limit, limit + 1]:
                payload = {"data": "x" * (size - len(b'{"data":""}'))}
                expected = json.dumps(
                    payload,
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
                self.assertEqual(len(expected), size)
                globals_["handle_http_list"] = lambda *args, value=payload: (
                    200,
                    value,
                )
                profile = namespace["DecisionWorkProfile"].http_v1(
                    {"response_bytes": limit}
                )
                response, observation = self.exchange_http_handler(
                    namespace, profile, request
                )
                self.assert_http_work_closed(observation)
                wire_body = response.split(b"\r\n\r\n", 1)[1]
                if size <= limit:
                    self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"))
                    self.assertEqual(wire_body, expected)
                    self.assertEqual(
                        observation["snapshot"]["cumulative"][
                            "response_bytes"
                        ],
                        size,
                    )
                    self.assertGreaterEqual(
                        observation["snapshot"]["peak"]["retained_bytes"],
                        size,
                    )
                    self.assertGreaterEqual(
                        observation["snapshot"]["peak"]["temporary_bytes"],
                        size,
                    )
                else:
                    self.assertTrue(
                        response.startswith(
                            b"HTTP/1.0 413 Request Entity Too Large\r\n"
                        )
                    )
                    self.assertNotIn(expected, response)
                    diagnostic = json.loads(wire_body)["diagnostics"][0]
                    self.assertEqual(diagnostic, observation["budget_diagnostic"])
                    self.assertEqual(diagnostic["dimension"], "response_bytes")
                    self.assertEqual(diagnostic["exit_code"], 60)
        finally:
            globals_["handle_http_list"] = original

    def test_python_http_get_budget_error_keeps_typed_parity(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_list"]
        error = namespace["ResourceBudgetExceeded"](
            profile="http_v1",
            dimension="graph_nodes",
            observed=11,
            limit=10,
            phase="query",
            command="http request",
            continuation_supported=False,
        )
        request = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
        try:
            def reject(*args):
                raise error

            globals_["handle_http_list"] = reject
            response, observation = self.exchange_http_handler(
                namespace,
                namespace["DecisionWorkProfile"].http_v1(),
                request,
            )
        finally:
            globals_["handle_http_list"] = original
        self.assertTrue(
            response.startswith(b"HTTP/1.0 413 Request Entity Too Large\r\n")
        )
        payload = json.loads(response.split(b"\r\n\r\n", 1)[1])
        self.assertEqual(payload["exit_code"], 60)
        self.assertEqual(payload["kind"], "resource_budget_exceeded")
        self.assertEqual(payload["diagnostics"], [error.diagnostic()])
        self.assertEqual(observation["budget_diagnostic"], error.diagnostic())
        self.assert_http_work_closed(observation)

    def test_python_http_compact_unicode_response_boundaries(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_list"]
        payload = {"é": "雪\n\\\"" + ("界" * 180)}
        expected = json.dumps(
            payload,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        request = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
        try:
            globals_["handle_http_list"] = lambda *args: (200, payload)
            for limit in [len(expected) - 1, len(expected), len(expected) + 1]:
                profile = namespace["DecisionWorkProfile"].http_v1(
                    {"response_bytes": limit}
                )
                response, observation = self.exchange_http_handler(
                    namespace, profile, request
                )
                self.assert_http_work_closed(observation)
                if limit < len(expected):
                    self.assertTrue(
                        response.startswith(
                            b"HTTP/1.0 413 Request Entity Too Large\r\n"
                        )
                    )
                    self.assertNotIn(expected, response)
                else:
                    self.assertEqual(
                        response.split(b"\r\n\r\n", 1)[1], expected
                    )
        finally:
            globals_["handle_http_list"] = original

    def test_python_http_parser_temporary_is_reserved_before_read(self):
        namespace = runpy.run_path(str(CODEFIRE))
        body = json.dumps(
            {"value": "x" * 240}, separators=(",", ":")
        ).encode()
        physical = len(body) * 2
        head = (
            b"POST /v1/projects/org/app/nope HTTP/1.0\r\n"
            + f"Content-Length: {len(body)}\r\n\r\n".encode()
        )
        for limit in [physical - 1, physical, physical + 1]:
            profile = namespace["DecisionWorkProfile"].http_v1(
                {"temporary_bytes": ("concurrent", limit)}
            )
            response, observation = self.exchange_http_handler(
                namespace, profile, head + body
            )
            self.assert_http_work_closed(observation)
            if limit < physical:
                self.assertEqual(observation["request_body_bytes_read"], 0)
                self.assertEqual(
                    observation["budget_diagnostic"]["dimension"],
                    "temporary_bytes",
                )
            else:
                self.assertEqual(observation["request_body_bytes_read"], len(body))
                self.assertEqual(
                    observation["snapshot"]["peak"]["temporary_bytes"],
                    physical,
                )
                self.assertTrue(
                    observation["request_raw_dropped_before_temp_release"]
                )
                self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))

    def test_python_http_canonical_ast_retained_boundaries(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_gc"]
        cases = [
            b"  { \"value\" : \"" + (b"x" * 300) + b"\" }  ",
            '{ "é" : "\\u96ea\\n\\\\\\\"'.encode("utf-8")
            + (b"z" * 300)
            + b'" }',
            b'"' + (b"y" * 320) + b'"',
            b"",
        ]
        try:
            globals_["handle_http_gc"] = lambda *args: (200, {})
            for body in cases:
                head = (
                    b"POST /v1/projects/org/app/gc HTTP/1.0\r\n"
                    + f"Content-Length: {len(body)}\r\n\r\n".encode()
                )
                value = json.loads(body.decode()) if body else {}
                canonical_size = len(
                    json.dumps(
                        value,
                        ensure_ascii=False,
                        sort_keys=True,
                        separators=(",", ":"),
                    ).encode()
                )
                ownership = (len(head) * 2) + canonical_size
                for limit in [ownership - 1, ownership, ownership + 1]:
                    profile = namespace["DecisionWorkProfile"].http_v1(
                        {"retained_bytes": ("concurrent", limit)}
                    )
                    response, observation = self.exchange_http_handler(
                        namespace, profile, head + body
                    )
                    self.assert_http_work_closed(observation)
                    if limit < ownership:
                        self.assertEqual(
                            observation["budget_diagnostic"]["dimension"],
                            "retained_bytes",
                        )
                    else:
                        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"))
                        self.assertEqual(
                            observation["snapshot"]["peak"]["retained_bytes"],
                            ownership,
                        )
        finally:
            globals_["handle_http_gc"] = original

    def test_python_http_owners_live_until_their_release_points(self):
        namespace = runpy.run_path(str(CODEFIRE))
        body = b'{"value":"owned"}'
        head = (
            b"POST /v1/projects/org/app/nope HTTP/1.0\r\n"
            + f"Content-Length: {len(body)}\r\n\r\n".encode()
        )
        response, observation = self.exchange_http_handler(
            namespace,
            namespace["DecisionWorkProfile"].http_v1(),
            head + body,
        )
        self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))
        owner = observation["response_owner"]
        self.assertEqual(owner["request_header_retained"], len(head) * 2)
        self.assertEqual(owner["request_ast_retained"], 0)
        self.assertTrue(owner["raw_requestline_live"])
        self.assertTrue(owner["headers_live"])
        self.assertTrue(observation["request_raw_dropped_before_temp_release"])
        self.assertTrue(observation["response_body_dropped_before_release"])
        self.assert_http_work_closed(observation)

    def test_python_http_preflights_encoder_and_header_allocations(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original_ledger = globals_["DecisionWorkLedger"]
        ledgers = []
        encoder_reserved = []
        header_accounted = []

        class RecordingLedger(original_ledger):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                ledgers.append(self)

        def configure(handler):
            original_upper = handler.json_encoder_chunk_upper.__func__
            original_send = handler.send_response

            def checked_upper(cls, payload):
                reservations = list(ledgers[-1]._reserved.values())
                encoder_reserved.append(
                    any(
                        item["acquires"].get("temporary_bytes", 0) > 0
                        for item in reservations
                    )
                )
                return original_upper(cls, payload)

            def checked_send(self, *args, **kwargs):
                header_accounted.append(
                    self.decision_work.current.get("temporary_bytes", 0) > 0
                )
                return original_send(self, *args, **kwargs)

            handler.json_encoder_chunk_upper = classmethod(checked_upper)
            handler.send_response = checked_send

        try:
            globals_["DecisionWorkLedger"] = RecordingLedger
            response, observation = self.exchange_http_handler(
                namespace,
                namespace["DecisionWorkProfile"].http_v1(),
                b"GET /v1/projects/org/app/nope HTTP/1.0\r\n\r\n",
                configure_handler=configure,
            )
        finally:
            globals_["DecisionWorkLedger"] = original_ledger
        self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))
        self.assertTrue(encoder_reserved)
        self.assertTrue(all(encoder_reserved))
        self.assertEqual(header_accounted, [True])
        self.assert_http_work_closed(observation)

    def test_python_http_header_reader_uses_no_extra_lookahead_byte(self):
        namespace = runpy.run_path(str(CODEFIRE))
        error_type = namespace["ResourceBudgetExceeded"]
        limit = 32

        def budget_error(observed):
            return error_type(
                profile="http_v1",
                dimension="request_header_bytes",
                observed=observed,
                limit=limit,
                phase="request_headers",
                command="http request",
                continuation_supported=False,
            )

        source = io.BytesIO(b"x" * (limit + 20))
        reader = namespace["_BoundedHTTPHeaderReader"](
            source, limit, budget_error
        )
        with self.assertRaises(error_type) as raised:
            reader.readline()
        self.assertEqual(raised.exception.diagnostic()["observed"], limit + 1)
        self.assertEqual(source.tell(), limit)

        exact_source = io.BytesIO((b"x" * (limit - 1)) + b"\nmore")
        exact_reader = namespace["_BoundedHTTPHeaderReader"](
            exact_source, limit, budget_error
        )
        self.assertEqual(len(exact_reader.readline()), limit)
        self.assertEqual(exact_source.tell(), limit)

    def test_python_bounded_json_freezer_stops_at_limit_plus_one(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].http_v1()
        ledger = namespace["DecisionWorkLedger"](
            profile, "http request", command_kind="http_request"
        )

        def budget_error(dimension, observed, phase):
            return namespace["ResourceBudgetExceeded"](
                profile=profile.profile_id,
                dimension=dimension,
                observed=observed,
                limit=profile.limit(dimension),
                phase=phase,
                command="http request",
                continuation_supported=False,
            )

        limit = 64
        freezer = namespace["_BoundedCanonicalJSONFreezer"](
            ledger, profile, budget_error, limit
        )
        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            freezer.freeze({"data": "x" * 1_000_000})
        diagnostic = raised.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "response_bytes")
        self.assertEqual(diagnostic["observed"], limit + 1)
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertFalse(ledger.has_open_permits())

    def test_python_bounded_json_freezer_preflights_deep_frontier(self):
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].http_v1(
            {"temporary_bytes": ("concurrent", 1)}
        )
        ledger = namespace["DecisionWorkLedger"](
            profile, "http request", command_kind="http_request"
        )

        def budget_error(dimension, observed, phase):
            return namespace["ResourceBudgetExceeded"](
                profile=profile.profile_id,
                dimension=dimension,
                observed=observed,
                limit=profile.limit(dimension),
                phase=phase,
                command="http request",
                continuation_supported=False,
            )

        payload = None
        for _ in range(100):
            payload = [payload]
        freezer = namespace["_BoundedCanonicalJSONFreezer"](
            ledger, profile, budget_error, 1024
        )
        with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
            freezer.freeze(payload)
        diagnostic = raised.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "temporary_bytes")
        self.assertEqual(diagnostic["observed"], freezer.STACK_BASE)
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertFalse(ledger.has_open_permits())

    def test_python_http_physical_body_ownership_is_bounded(self):
        namespace = runpy.run_path(str(CODEFIRE))
        body = b'"' + (b"x" * 510) + b'"'
        physical = len(body) * 2
        request = (
            b"POST /v1/projects/org/app/nope HTTP/1.0\r\n"
            + f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
            + body
        )

        for retained_limit in [physical - 1, physical, physical + 1]:
            profile = namespace["DecisionWorkProfile"].http_v1(
                {"temporary_bytes": ("concurrent", retained_limit)}
            )
            response, observation = self.exchange_http_handler(
                namespace, profile, request
            )
            self.assert_http_work_closed(observation)
            if retained_limit < physical:
                self.assertTrue(
                    response.startswith(b"HTTP/1.0 413 Request Entity Too Large\r\n")
                )
                self.assertEqual(observation["request_body_bytes_read"], 0)
                diagnostic = observation["budget_diagnostic"]
                self.assertEqual(diagnostic["dimension"], "temporary_bytes")
                self.assertEqual(diagnostic["observed"], physical)
                self.assertEqual(diagnostic["limit"], retained_limit)
            else:
                self.assertTrue(response.startswith(b"HTTP/1.0 404 Not Found\r\n"))
                self.assertEqual(observation["request_body_bytes_read"], len(body))
                self.assertEqual(
                    observation["snapshot"]["peak"]["temporary_bytes"],
                    physical,
                )

    def test_python_http_request_ast_stays_owned_through_dispatch(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original_ledger = globals_["DecisionWorkLedger"]
        original_gc = globals_["handle_http_gc"]
        ledgers = []
        retained_during_dispatch = []

        class RecordingLedger(original_ledger):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                ledgers.append(self)

        body = json.dumps({"value": "owned request"}, sort_keys=True).encode()
        request = (
            b"POST /v1/projects/org/app/gc HTTP/1.0\r\n"
            + f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
            + body
        )
        try:
            globals_["DecisionWorkLedger"] = RecordingLedger
            for rejected in [False, True]:
                def observe(*args, fail=rejected):
                    retained_during_dispatch.append(
                        ledgers[-1].current.get("retained_bytes", 0)
                    )
                    if fail:
                        raise namespace["CFError"]("injected dispatch rejection")
                    return 200, {"ok": True}

                globals_["handle_http_gc"] = observe
                response, observation = self.exchange_http_handler(
                    namespace,
                    namespace["DecisionWorkProfile"].http_v1(),
                    request,
                )
                expected_status = b"400 Bad Request" if rejected else b"200 OK"
                self.assertIn(expected_status, response.split(b"\r\n", 1)[0])
                self.assert_http_work_closed(observation)
        finally:
            globals_["DecisionWorkLedger"] = original_ledger
            globals_["handle_http_gc"] = original_gc
        header_size = len(request) - len(body)
        canonical_body_size = len(
            json.dumps(
                json.loads(body),
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            ).encode()
        )
        expected_retained = (header_size * 2) + canonical_body_size
        self.assertEqual(
            retained_during_dispatch,
            [expected_retained, expected_retained],
        )

    def test_python_http_response_physical_limits_are_atomic(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_list"]
        request = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
        body_size = 1024
        payload = {"data": "x" * (body_size - len(b'{"data":""}'))}
        expected = json.dumps(
            payload, sort_keys=True, separators=(",", ":")
        ).encode("utf-8")
        self.assertEqual(len(expected), body_size)
        globals_["handle_http_list"] = lambda *args: (200, payload)
        try:
            encoding_upper = (len(json.dumps(payload["data"])) + 4) * 2
            for dimension, physical in [
                ("retained_bytes", (len(request) * 2) + body_size),
                ("temporary_bytes", encoding_upper),
            ]:
                for configured in [physical - 1, physical, physical + 1]:
                    profile = namespace["DecisionWorkProfile"].http_v1(
                        {dimension: ("concurrent", configured)}
                    )
                    response, observation = self.exchange_http_handler(
                        namespace, profile, request
                    )
                    self.assert_http_work_closed(observation)
                    if configured < physical:
                        self.assertTrue(
                            response.startswith(
                                b"HTTP/1.0 413 Request Entity Too Large\r\n"
                            )
                        )
                        self.assertNotIn(expected, response)
                        diagnostic = observation["budget_diagnostic"]
                        self.assertEqual(diagnostic["dimension"], dimension)
                        self.assertEqual(diagnostic["observed"], physical)
                        self.assertEqual(diagnostic["limit"], configured)
                    else:
                        self.assertTrue(
                            response.startswith(b"HTTP/1.0 200 OK\r\n")
                        )
                        self.assertEqual(
                            response.split(b"\r\n\r\n", 1)[1], expected
                        )
                        self.assertLessEqual(
                            observation["snapshot"]["peak"][dimension],
                            configured,
                        )
        finally:
            globals_["handle_http_list"] = original

    def test_python_http_response_header_is_never_partially_sent(self):
        namespace = runpy.run_path(str(CODEFIRE))
        globals_ = namespace["make_http_handler"].__globals__
        original = globals_["handle_http_list"]
        globals_["handle_http_list"] = lambda *args: (200, {"ok": True})
        request = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
        try:
            baseline, baseline_observation = self.exchange_http_handler(
                namespace,
                namespace["DecisionWorkProfile"].http_v1(),
                request,
            )
            self.assertTrue(baseline.startswith(b"HTTP/1.0 200 OK\r\n"))
            actual = baseline_observation["snapshot"]["cumulative"][
                "response_header_bytes"
            ]
            for header_limit in [actual - 1, actual, actual + 1]:
                profile = namespace["DecisionWorkProfile"].http_v1(
                    {"response_header_bytes": header_limit}
                )
                response, observation = self.exchange_http_handler(
                    namespace, profile, request
                )
                self.assert_http_work_closed(observation)
                if header_limit < actual:
                    self.assertEqual(response, b"")
                    diagnostic = observation["budget_diagnostic"]
                    self.assertEqual(
                        diagnostic["dimension"], "response_header_bytes"
                    )
                    self.assertEqual(diagnostic["observed"], actual)
                    self.assertEqual(diagnostic["limit"], actual - 1)
                else:
                    self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"))
                    self.assertEqual(
                        observation["snapshot"]["cumulative"][
                            "response_header_bytes"
                        ],
                        actual,
                    )
        finally:
            globals_["handle_http_list"] = original

    def test_python_verify_budget_failure_does_not_publish_scan_plan(self):
        self.run_cf("init")
        open_dir = self.tmp / "main"
        self.run_cf("open", "main", str(open_dir))
        (open_dir / "docs").mkdir()
        (open_dir / "docs/only.md").write_text(
            "## REQ-ONLY: Only requirement\n", encoding="utf-8"
        )
        (open_dir / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - path: docs/**/*.md\n"
            "      kind: requirement_document\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.links.yaml").write_text(
            "version: 1\nlinks:\n", encoding="utf-8"
        )
        rules = "".join(
            "    - type: missing_{}\n"
            "      target_kind: design\n"
            "      min: 1\n".format(index)
            for index in range(4)
        )
        (open_dir / "codefire.policy.yaml").write_text(
            "version: 1\nrequired_links:\n  requirement:\n" + rules,
            encoding="utf-8",
        )
        active = self.tmp / ".codefire/active"
        before = {
            path.relative_to(self.tmp).as_posix(): path.read_bytes()
            for path in [
                *active.glob("*/fires.json"),
                *active.glob("*/scan.json"),
                *active.glob("*/state.json"),
                *self.tmp.glob(".codefire/opened/*.json"),
            ]
        }
        namespace = runpy.run_path(str(CODEFIRE))
        profile = namespace["DecisionWorkProfile"].local_v1(
            {"counter_values": ("maximum", 3)}
        )
        ledger = namespace["DecisionWorkLedger"](profile, "verify no effect")
        old_cwd = Path.cwd()
        try:
            os.chdir(open_dir)
            with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                namespace["run_verify"](ledger)
        finally:
            os.chdir(old_cwd)
        diagnostic = raised.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "counter_values")
        self.assertEqual(diagnostic["observed"], 4)
        self.assertEqual(diagnostic["phase"], "query")
        after = {
            path.relative_to(self.tmp).as_posix(): path.read_bytes()
            for path in [
                *active.glob("*/fires.json"),
                *active.glob("*/scan.json"),
                *active.glob("*/state.json"),
                *self.tmp.glob(".codefire/opened/*.json"),
            ]
        }
        self.assertEqual(after, before)

    def test_python_scan_full_surface_releases_success_failure_and_preflight(self):
        self.run_cf("init")
        open_dir = self.tmp / "main"
        self.run_cf("open", "main", str(open_dir))
        (open_dir / "docs").mkdir()
        (open_dir / "docs/only.md").write_text(
            "## REQ-SCAN-OWNER: Scan owner\n", encoding="utf-8"
        )
        (open_dir / "codefire.yaml").write_text(
            "version: 1\nartifacts:\n  requirements:\n"
            "    - path: docs/**/*.md\n      kind: requirement_document\n",
            encoding="utf-8",
        )
        namespace = runpy.run_path(str(CODEFIRE))
        old_cwd = Path.cwd()
        try:
            os.chdir(open_dir)
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(), "scan success owner"
            )
            namespace["run_scan"](ledger, publish=False)
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

            failing = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(), "scan failure owner"
            )
            function = namespace["run_scan"]
            original_changed = function.__globals__["changed_atoms"]
            original_adjacent = function.__globals__["adjacent_atoms"]
            function.__globals__["changed_atoms"] = lambda current, base: [
                current["atoms"][0]["atom_id"]
            ]
            function.__globals__["adjacent_atoms"] = lambda *_args: (_ for _ in ()).throw(
                namespace["CFError"]("injected adjacent failure")
            )
            try:
                with self.assertRaisesRegex(namespace["CFError"], "injected"):
                    function(failing, publish=False)
            finally:
                function.__globals__["changed_atoms"] = original_changed
                function.__globals__["adjacent_atoms"] = original_adjacent
            snapshot = failing.snapshot()
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

            preflight = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(
                    {"temporary_bytes": ("concurrent", 128 * 1024 * 1024 - 1)}
                ),
                "scan preflight",
            )
            with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                function(preflight, publish=False)
            self.assertEqual(raised.exception.diagnostic()["dimension"], "temporary_bytes")
            self.assertEqual(
                preflight.snapshot()["current"].get("temporary_bytes", 0), 0
            )
        finally:
            os.chdir(old_cwd)

    def test_python_local_doctor_uses_one_terminal_zero_current_ledger(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        function = namespace["cmd_doctor"]
        original = function.__globals__["DecisionWorkLedger"]
        ledgers = []

        def recording_ledger(*args, **kwargs):
            ledger = original(*args, **kwargs)
            ledgers.append(ledger)
            return ledger

        class Args:
            target = str(self.tmp)

        function.__globals__["DecisionWorkLedger"] = recording_ledger
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(function(Args()), 0)
        finally:
            function.__globals__["DecisionWorkLedger"] = original
        self.assertEqual(len(ledgers), 1)
        snapshot = ledgers[0].snapshot()
        self.assertEqual(snapshot["terminal_reason"], "complete")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)
        self.assertGreater(snapshot["counters"].get("record_reads", 0), 0)
        self.assertGreater(
            snapshot["cumulative"].get("snapshot_revalidation_records", 0), 0
        )

    def test_python_remote_list_and_doctor_use_one_zero_current_ledger(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Accounted remote")
        project_url = f"cf://{self.tmp / 'server'}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        namespace = runpy.run_path(str(CODEFIRE))
        original = namespace["DecisionWorkLedger"]

        class ListArgs:
            server_url = project_url

        for function, invoke in [
            (namespace["cmd_list"], lambda fn: fn(ListArgs())),
            (
                namespace["remote_doctor_payload"],
                lambda fn: fn(
                    namespace["parse_cf_project_url"](project_url)["project_root"],
                    project_url,
                ),
            ),
        ]:
            ledgers = []

            def recording_ledger(*args, **kwargs):
                ledger = original(*args, **kwargs)
                ledgers.append(ledger)
                return ledger

            function.__globals__["DecisionWorkLedger"] = recording_ledger
            try:
                with contextlib.redirect_stdout(io.StringIO()):
                    result = invoke(function)
            finally:
                function.__globals__["DecisionWorkLedger"] = original
            if function is namespace["cmd_list"]:
                self.assertEqual(result, 0)
            else:
                self.assertTrue(result["ok"])
            self.assertEqual(len(ledgers), 1)
            snapshot = ledgers[0].snapshot()
            if function is namespace["cmd_list"]:
                self.assertEqual(snapshot["terminal_reason"], "complete")
                self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            else:
                self.assertIsNone(snapshot["terminal_reason"])
                self.assertGreater(snapshot["current"].get("retained_bytes", 0), 0)
                result.close()
                snapshot = ledgers[0].snapshot()
                self.assertEqual(snapshot["terminal_reason"], "complete")
                self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)
            self.assertGreater(
                snapshot["cumulative"].get("snapshot_revalidation_records", 0),
                0,
            )

    def test_python_remote_doctor_exception_closes_all_ledger_ownership(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Doctor exception cleanup")
        project_url = f"cf://{self.tmp / 'server'}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        namespace = runpy.run_path(str(CODEFIRE))
        function = namespace["remote_doctor_payload"]
        original_ledger = function.__globals__["DecisionWorkLedger"]
        original_paths = function.__globals__["accounted_sorted_json_paths"]
        ledgers = []
        calls = 0

        def recording_ledger(*args, **kwargs):
            ledger = original_ledger(*args, **kwargs)
            ledgers.append(ledger)
            return ledger

        def fail_after_first_owned_paths(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise namespace["CFError"]("injected remote doctor failure")
            return original_paths(*args, **kwargs)

        function.__globals__["DecisionWorkLedger"] = recording_ledger
        function.__globals__["accounted_sorted_json_paths"] = fail_after_first_owned_paths
        try:
            with self.assertRaisesRegex(
                namespace["CFError"], "injected remote doctor failure"
            ):
                function(
                    namespace["parse_cf_project_url"](project_url)["project_root"],
                    project_url,
                )
        finally:
            function.__globals__["DecisionWorkLedger"] = original_ledger
            function.__globals__["accounted_sorted_json_paths"] = original_paths

        self.assertEqual(len(ledgers), 1)
        snapshot = ledgers[0].snapshot()
        self.assertEqual(snapshot["terminal_reason"], "corrupt_subject")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_remote_doctor_exception_drops_inputs_before_release(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Doctor owner cleanup")
        project_url = f"cf://{self.tmp / 'server'}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        namespace = runpy.run_path(str(CODEFIRE))
        function = namespace["remote_doctor_payload"]
        original_ledger = function.__globals__["DecisionWorkLedger"]
        original_paths = function.__globals__["accounted_sorted_json_paths"]
        original_read = function.__globals__["read_json_accounted"]
        original_pending = function.__globals__["remote_pending_object_roots"]
        ledgers = []
        owners = []
        release_states = []
        pending_failed = False

        class ObservedList(list):
            cleared = False

            def clear(self):
                self.cleared = True
                super().clear()

        class ObservedDict(dict):
            cleared = False

            def clear(self):
                self.cleared = True
                super().clear()

        class RecordingLedger(original_ledger):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                ledgers.append(self)

            def release(self, dimension, amount):
                if pending_failed and dimension == "retained_bytes":
                    release_states.append(all(owner.cleared for owner in owners))
                return super().release(dimension, amount)

        def observed_paths(*args, **kwargs):
            paths, retained = original_paths(*args, **kwargs)
            owner = ObservedList(paths)
            owners.append(owner)
            return owner, retained

        def observed_read(*args, **kwargs):
            value, retained = original_read(*args, **kwargs)
            if retained and isinstance(value, dict):
                owner = ObservedDict(value)
                owners.append(owner)
                return owner, retained
            return value, retained

        def fail_pending(*args, **kwargs):
            nonlocal pending_failed
            pending_failed = True
            raise RuntimeError("injected pending-root failure")

        function.__globals__["DecisionWorkLedger"] = RecordingLedger
        function.__globals__["accounted_sorted_json_paths"] = observed_paths
        function.__globals__["read_json_accounted"] = observed_read
        function.__globals__["remote_pending_object_roots"] = fail_pending
        try:
            with self.assertRaisesRegex(
                RuntimeError, "injected pending-root failure"
            ):
                function(
                    namespace["parse_cf_project_url"](project_url)["project_root"],
                    project_url,
                )
        finally:
            function.__globals__["DecisionWorkLedger"] = original_ledger
            function.__globals__["accounted_sorted_json_paths"] = original_paths
            function.__globals__["read_json_accounted"] = original_read
            function.__globals__["remote_pending_object_roots"] = original_pending

        self.assertTrue(owners)
        self.assertTrue(release_states)
        self.assertTrue(all(release_states))
        self.assertTrue(all(owner.cleared for owner in owners))
        snapshot = ledgers[0].snapshot()
        self.assertEqual(snapshot["terminal_reason"], "corrupt_subject")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_remote_doctor_payload_constructor_failure_drops_payload_before_release(self):
        namespace = runpy.run_path(str(CODEFIRE))
        function = namespace["remote_doctor_payload"]
        original_ledger = function.__globals__["DecisionWorkLedger"]
        original_payload_type = function.__globals__["AccountedDoctorPayload"]
        ledgers = []
        payloads = []
        nested_payloads = []
        release_lengths = []

        class RecordingLedger(original_ledger):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                ledgers.append(self)

            def release(self, dimension, amount):
                if payloads:
                    release_lengths.append(
                        (dict.__len__(payloads[0]), dict.__len__(nested_payloads[0]))
                    )
                return super().release(dimension, amount)

        class FailingPayload(dict):
            def __init__(self, payload, *args, **kwargs):
                payloads.append(payload)
                nested_payloads.append(payload["layout"])
                raise MemoryError("injected payload construction failure")

        function.__globals__["DecisionWorkLedger"] = RecordingLedger
        function.__globals__["AccountedDoctorPayload"] = FailingPayload
        try:
            with self.assertRaisesRegex(
                MemoryError, "injected payload construction failure"
            ):
                function(
                    self.tmp / "missing-remote",
                    f"cf://{self.tmp / 'missing-server'}/org/app",
                )
        finally:
            function.__globals__["DecisionWorkLedger"] = original_ledger
            function.__globals__["AccountedDoctorPayload"] = original_payload_type

        self.assertTrue(release_lengths)
        self.assertEqual(release_lengths, [(0, 0), (0, 0)])
        snapshot = ledgers[0].snapshot()
        self.assertEqual(snapshot["terminal_reason"], "corrupt_subject")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_trace_index_preserves_reference_order_duplicates_and_self_loop(self):
        namespace = runpy.run_path(str(CODEFIRE))
        atom_index = {
            "atoms": [
                {"atom_id": "REQ-1", "kind": "requirement"},
                {"atom_id": "DES-1", "kind": "design"},
                {"atom_id": "DES-2", "kind": "design"},
            ]
        }
        trace_graph = {
            "links": [
                {"from": "REQ-1", "to": "DES-1", "type": "refined_by"},
                {"from": "REQ-1", "to": "DES-1", "type": "refined_by"},
                {"from": "DES-1", "to": "REQ-1", "type": "supports"},
                {"from": "REQ-1", "to": "REQ-1", "type": "self"},
            ]
        }
        rules = {
            "requirement": [
                {"type": "refined_by", "target_kind": "design", "min": 2},
                {"type": "verified_by", "target_kind": "test", "min": 1},
            ]
        }
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "trace equivalence"
        )
        index = namespace["compile_trace_evaluation_index"](
            atom_index, trace_graph, ledger
        )
        missing = namespace["required_link_missing"](
            atom_index, trace_graph, rules, ledger, index
        )
        self.assertEqual(
            missing,
            [
                {
                    "atom_id": "REQ-1",
                    "required_type": "verified_by",
                    "target_kind": "test",
                    "min": 1,
                    "found": 0,
                }
            ],
        )
        adjacent = namespace["adjacent_atoms"]("REQ-1", index)
        self.assertEqual(
            adjacent,
            [
                ("DES-1", ["REQ-1", "DES-1"]),
                ("DES-1", ["REQ-1", "DES-1"]),
                ("DES-1", ["REQ-1", "DES-1"]),
                ("REQ-1", ["REQ-1", "REQ-1"]),
            ],
        )
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["counters"]["atoms_indexed"], 3)
        self.assertEqual(snapshot["counters"]["trace_links_indexed"], 4)
        self.assertEqual(snapshot["counters"]["rules_evaluated"], 2)
        adjacent.close()
        index.close()
        self.assertEqual(
            ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

    def test_python_trace_owners_drop_payload_and_internal_index_closes(self):
        namespace = runpy.run_path(str(CODEFIRE))
        atom_index = {
            "atoms": [
                {"atom_id": "REQ-OWNER", "kind": "requirement"},
                {"atom_id": "DES-OWNER", "kind": "design"},
            ]
        }
        trace_graph = {
            "links": [
                {
                    "from": "REQ-OWNER",
                    "to": "DES-OWNER",
                    "type": "refined_by",
                }
            ]
        }
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "owner lifecycle"
        )
        index = namespace["compile_trace_evaluation_index"](
            atom_index, trace_graph, ledger
        )
        adjacent = namespace["adjacent_atoms"]("REQ-OWNER", index)
        adjacent.close()
        self.assertEqual(list.__len__(adjacent), 0)
        with self.assertRaises(namespace["CFError"]):
            list(adjacent)
        index.close()
        with self.assertRaises(namespace["CFError"]):
            _ = index.atoms
        self.assertEqual(
            ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

        second = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "internal owner"
        )
        self.assertEqual(
            namespace["required_link_missing"](
                atom_index,
                trace_graph,
                {"requirement": []},
                ledger=second,
            ),
            [],
        )
        self.assertEqual(
            second.snapshot()["current"].get("retained_bytes", 0), 0
        )

    def test_python_trace_owners_release_on_gc_without_explicit_close(self):
        namespace = runpy.run_path(str(CODEFIRE))
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "trace gc owner"
        )
        index = namespace["compile_trace_evaluation_index"](
            {"atoms": [{"atom_id": "REQ-GC", "kind": "requirement"}]},
            {"links": []},
            ledger,
        )
        adjacent = namespace["adjacent_atoms"]("REQ-GC", index)
        self.assertGreater(
            ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

        del adjacent
        del index
        gc.collect()

        self.assertEqual(
            ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

    def test_python_trace_index_duplicate_occurrence_order_matches_rust(self):
        namespace = runpy.run_path(str(CODEFIRE))
        atom_index = {
            "atoms": [
                {"atom_id": "DUP", "kind": "requirement"},
                {"atom_id": "TARGET", "kind": "design"},
                {"atom_id": "DUP", "kind": "design"},
            ]
        }
        trace_graph = {
            "links": [
                {"from": "DUP", "to": "DUP", "type": "refined_by"}
            ]
        }
        rules = {
            "requirement": [
                {"type": "refined_by", "target_kind": "design", "min": 1}
            ],
            "design": [
                {"type": "implemented_by", "target_kind": "code", "min": 1}
            ],
        }
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(),
            "duplicate occurrence vector",
        )
        index = namespace["compile_trace_evaluation_index"](
            atom_index, trace_graph, ledger
        )
        self.assertEqual(
            namespace["required_link_missing"](
                atom_index, trace_graph, rules, ledger, index
            ),
            [
                {
                    "atom_id": "TARGET",
                    "required_type": "implemented_by",
                    "target_kind": "code",
                    "min": 1,
                    "found": 0,
                },
                {
                    "atom_id": "DUP",
                    "required_type": "implemented_by",
                    "target_kind": "code",
                    "min": 1,
                    "found": 0,
                },
            ],
        )
        index.close()
        self.assertEqual(
            ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

    def test_python_trace_index_retained_limit_minus_exact_plus_one(self):
        namespace = runpy.run_path(str(CODEFIRE))
        atom_index = {
            "atoms": [
                {"atom_id": "REQ-BOUNDARY", "kind": "requirement"},
                {"atom_id": "DES-BOUNDARY", "kind": "design"},
            ]
        }
        trace_graph = {
            "links": [
                {
                    "from": "REQ-BOUNDARY",
                    "to": "DES-BOUNDARY",
                    "type": "refined_by",
                }
            ]
        }
        baseline_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "trace retained baseline"
        )
        baseline = namespace["compile_trace_evaluation_index"](
            atom_index, trace_graph, baseline_ledger
        )
        retained_bytes = baseline.retained_bytes
        baseline.close()
        limit = namespace["DecisionWorkProfile"].local_v1().limit("retained_bytes")
        exact_filler = limit - retained_bytes

        for adjustment, succeeds in [(-1, True), (0, True), (1, False)]:
            filler = exact_filler + adjustment
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(),
                "trace retained boundary",
            )
            ledger.acquire("retained_bytes", filler, "validation")
            if succeeds:
                index = namespace["compile_trace_evaluation_index"](
                    atom_index, trace_graph, ledger
                )
                index.close()
            else:
                with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                    namespace["compile_trace_evaluation_index"](
                        atom_index, trace_graph, ledger
                    )
                self.assertEqual(
                    raised.exception.diagnostic()["observed"], limit + 1
                )
                self.assertEqual(
                    ledger.snapshot()["current"]["retained_bytes"], filler
                )
            ledger.release("retained_bytes", filler)
            self.assertEqual(
                ledger.snapshot()["current"].get("retained_bytes", 0), 0
            )

    def test_python_sealed_session_object_record_limit_minus_exact_plus_one(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]

        baseline_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "sealed baseline"
        )
        baseline = namespace["SealedCommitValidationSession"](
            objects_root, baseline_ledger
        )
        baseline.validate_commit(head)
        record_count = baseline_ledger.snapshot()["cumulative"]["object_records"]
        baseline.finalize()
        self.assertEqual(
            baseline_ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )

        for limit, succeeds in [
            (record_count - 1, False),
            (record_count, True),
            (record_count + 1, True),
        ]:
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(
                    {"object_records": limit}
                ),
                "sealed object boundary",
            )
            session = namespace["SealedCommitValidationSession"](
                objects_root, ledger
            )
            try:
                if succeeds:
                    session.validate_commit(head)
                    session.finalize()
                    self.assertEqual(
                        ledger.snapshot()["cumulative"]["object_records"],
                        record_count,
                    )
                else:
                    with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                        session.validate_commit(head)
                    self.assertEqual(
                        raised.exception.diagnostic()["dimension"], "object_records"
                    )
            finally:
                session.close(None)
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_32_heads_share_document_and_commit_memos(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        base = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        template = json.loads(
            (objects_root / "commits" / f"{base}.json").read_text(encoding="utf-8")
        )["payload"]
        heads = []
        for index in range(32):
            payload = json.loads(json.dumps(template))
            payload["parents"] = [base]
            payload["message"] = f"shared-dag-head-{index}"
            heads.append(self.write_commit_object_to(objects_root, payload))

        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "32 shared heads"
        )
        session = namespace["SealedCommitValidationSession"](
            objects_root, ledger
        )
        for head in heads:
            session.validate_commit(head)
        before_finalize = ledger.snapshot()
        expected_documents = 33 + len(set(template["roots"].values()))
        self.assertEqual(
            before_finalize["cumulative"]["object_records"], expected_documents
        )
        self.assertGreaterEqual(before_finalize["counters"].get("revisits", 0), 31)
        session.finalize()
        snapshot = ledger.snapshot()
        self.assertEqual(
            snapshot["cumulative"]["snapshot_revalidation_records"],
            expected_documents,
        )
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_detects_same_size_same_mtime_tamper(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "same size tamper"
        )
        session = namespace["SealedCommitValidationSession"](
            objects_root, ledger
        )
        session.validate_commit(head)
        path = objects_root / "commits" / f"{head}.json"
        before_stat = path.stat()
        raw = path.read_bytes()
        marker = raw.index(b"  ")
        tampered = raw[:marker] + b" \t" + raw[marker + 2 :]
        self.assertEqual(len(tampered), len(raw))
        path.write_bytes(tampered)
        os.utime(path, ns=(before_stat.st_atime_ns, before_stat.st_mtime_ns))
        with self.assertRaises(namespace["SealedValidationSubjectChanged"]) as raised:
            session.finalize()
        self.assertEqual(raised.exception.change_kind, "digest_changed")
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["terminal_reason"], "subject_changed")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_snapshot_limit_minus_exact_plus_one(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        baseline_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "snapshot baseline"
        )
        baseline = namespace["SealedCommitValidationSession"](
            objects_root, baseline_ledger
        )
        baseline.validate_commit(head)
        expected = baseline_ledger.snapshot()["cumulative"]["object_records"]
        baseline.finalize()

        for limit, succeeds in [(expected - 1, False), (expected, True), (expected + 1, True)]:
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(
                    {"snapshot_revalidation_records": limit}
                ),
                "snapshot boundary",
            )
            session = namespace["SealedCommitValidationSession"](
                objects_root, ledger
            )
            session.validate_commit(head)
            if succeeds:
                session.finalize()
            else:
                with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                    session.finalize()
                self.assertEqual(
                    raised.exception.diagnostic()["dimension"],
                    "snapshot_revalidation_records",
                )
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_rejects_metachar_huge_and_symlink_records(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"

        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "invalid object id"
        )
        session = namespace["SealedCommitValidationSession"](objects_root, ledger)
        with self.assertRaisesRegex(namespace["CFError"], "invalid object id"):
            session.validate_commit("CF-COMMIT-[0-9]*")
        session.close(None)
        self.assertEqual(ledger.snapshot()["current"].get("retained_bytes", 0), 0)

        huge_id = "CF-COMMIT-eeeeeeeeeeee"
        huge_path = objects_root / "commits" / f"{huge_id}.json"
        with huge_path.open("wb") as handle:
            handle.truncate(namespace["MAX_OBJECT_RECORD_BYTES"] + 1)
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "huge object"
        )
        session = namespace["SealedCommitValidationSession"](objects_root, ledger)
        with self.assertRaisesRegex(namespace["CFError"], "exceeds the"):
            session.validate_commit(huge_id)
        session.close(None)
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)
        self.assertNotIn("object_records", snapshot["cumulative"])

        symlink_id = "CF-COMMIT-ffffffffffff"
        symlink_path = objects_root / "commits" / f"{symlink_id}.json"
        symlink_path.symlink_to(huge_path.name)
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "symlink object"
        )
        session = namespace["SealedCommitValidationSession"](objects_root, ledger)
        with self.assertRaises(namespace["CFError"]):
            session.validate_commit(symlink_id)
        session.close(None)
        self.assertEqual(ledger.snapshot()["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_final_revalidation_rejects_huge_and_symlink_swap(self):
        for swap in ["huge", "symlink"]:
            repo = self.tmp / swap
            self.run_cf("init", str(repo))
            namespace = runpy.run_path(str(CODEFIRE))
            objects_root = repo / ".codefire/objects"
            head = json.loads(
                (repo / ".codefire/branches/main.json").read_text(encoding="utf-8")
            )["head"]
            head_path = objects_root / "commits" / f"{head}.json"
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(), f"{swap} swap"
            )
            session = namespace["SealedCommitValidationSession"](
                objects_root, ledger
            )
            session.validate_commit(head)
            head_path.unlink()
            if swap == "huge":
                with head_path.open("wb") as handle:
                    handle.truncate(namespace["MAX_OBJECT_RECORD_BYTES"] + 1)
            else:
                target = next((objects_root / "policies").glob("*.json"))
                head_path.symlink_to(target)
            with self.assertRaises(namespace["SealedValidationSubjectChanged"]):
                session.finalize()
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["terminal_reason"], "subject_changed")
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_rejects_ambient_store_replacement(self):
        for replacement in ["root", "commits"]:
            repo = self.tmp / replacement
            self.run_cf("init", str(repo))
            namespace = runpy.run_path(str(CODEFIRE))
            objects_root = repo / ".codefire/objects"
            head = json.loads(
                (repo / ".codefire/branches/main.json").read_text(encoding="utf-8")
            )["head"]
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(),
                f"ambient {replacement} replacement",
            )
            session = namespace["SealedCommitValidationSession"](
                objects_root, ledger
            )
            session.validate_commit(head)

            target = objects_root if replacement == "root" else objects_root / "commits"
            displaced = target.with_name(f"{target.name}-displaced")
            target.rename(displaced)
            shutil.copytree(displaced, target)

            with self.assertRaises(
                namespace["SealedValidationSubjectChanged"]
            ) as raised:
                session.finalize()
            self.assertEqual(raised.exception.change_kind, "store_identity_changed")
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["terminal_reason"], "subject_changed")
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_closing_swap_is_subject_changed_terminal(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "closing store swap"
        )
        session = namespace["SealedCommitValidationSession"](
            objects_root, ledger
        )
        session.validate_commit(head)
        expected_reads = len(session.documents)
        original_read = session._read_exact_bounded
        displaced = objects_root.with_name("objects-displaced")

        def swap_after_last_read(*args, **kwargs):
            result = original_read(*args, **kwargs)
            if kwargs.get("revalidation"):
                swap_after_last_read.calls += 1
                if swap_after_last_read.calls == expected_reads:
                    objects_root.rename(displaced)
                    shutil.copytree(displaced, objects_root)
            return result

        swap_after_last_read.calls = 0
        session._read_exact_bounded = swap_after_last_read
        with self.assertRaises(
            namespace["SealedValidationSubjectChanged"]
        ) as raised:
            session.finalize()
        self.assertEqual(swap_after_last_read.calls, expected_reads)
        self.assertEqual(raised.exception.change_kind, "store_identity_changed")
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["terminal_reason"], "subject_changed")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_rejects_hardlinked_record_initial_and_final(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        record_path = objects_root / "commits" / f"{head}.json"
        alias_path = record_path.with_name(f".{head}.hardlink")

        os.link(record_path, alias_path)
        initial_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "initial hardlink"
        )
        initial_session = namespace["SealedCommitValidationSession"](
            objects_root, initial_ledger
        )
        with self.assertRaisesRegex(namespace["CFError"], "identity is invalid"):
            initial_session.validate_commit(head)
        initial_session.close(None)
        self.assertEqual(
            initial_ledger.snapshot()["current"].get("retained_bytes", 0), 0
        )
        alias_path.unlink()

        final_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "final hardlink"
        )
        final_session = namespace["SealedCommitValidationSession"](
            objects_root, final_ledger
        )
        final_session.validate_commit(head)
        os.link(record_path, alias_path)
        with self.assertRaises(
            namespace["SealedValidationSubjectChanged"]
        ) as raised:
            final_session.finalize()
        self.assertEqual(raised.exception.change_kind, "identity_changed")
        snapshot = final_ledger.snapshot()
        self.assertEqual(snapshot["terminal_reason"], "subject_changed")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_rejects_stat_open_size_divergence_initial_and_final(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        record_path = objects_root / "commits" / f"{head}.json"
        real_open = namespace["os"].open

        def append_once_before_open(path, flags, *args, **kwargs):
            if (
                path == record_path.name
                and kwargs.get("dir_fd") is not None
                and not append_once_before_open.done
            ):
                append_once_before_open.done = True
                append_fd = real_open(
                    record_path,
                    os.O_WRONLY | os.O_APPEND,
                )
                try:
                    os.write(append_fd, b" ")
                finally:
                    os.close(append_fd)
            return real_open(path, flags, *args, **kwargs)

        initial_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "initial stat-open size"
        )
        initial_session = namespace["SealedCommitValidationSession"](
            objects_root, initial_ledger
        )
        append_once_before_open.done = False
        namespace["os"].open = append_once_before_open
        try:
            with self.assertRaisesRegex(namespace["CFError"], "identity is invalid"):
                initial_session.validate_commit(head)
        finally:
            namespace["os"].open = real_open
            initial_session.close(None)
        self.assertTrue(append_once_before_open.done)
        initial_snapshot = initial_ledger.snapshot()
        self.assertEqual(initial_snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(initial_snapshot["current"].get("temporary_bytes", 0), 0)

        final_ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "final stat-open size"
        )
        final_session = namespace["SealedCommitValidationSession"](
            objects_root, final_ledger
        )
        final_session.validate_commit(head)
        append_once_before_open.done = False
        namespace["os"].open = append_once_before_open
        try:
            with self.assertRaises(
                namespace["SealedValidationSubjectChanged"]
            ) as raised:
                final_session.finalize()
        finally:
            namespace["os"].open = real_open
        self.assertTrue(append_once_before_open.done)
        self.assertEqual(raised.exception.change_kind, "identity_changed")
        final_snapshot = final_ledger.snapshot()
        self.assertEqual(final_snapshot["terminal_reason"], "subject_changed")
        self.assertEqual(final_snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(final_snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_rejects_post_open_regular_file_replacement(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(),
            "post-open regular replacement",
        )
        session = namespace["SealedCommitValidationSession"](
            objects_root, ledger
        )
        session.validate_commit(head)
        first_id = sorted(session.documents)[0]
        target = session.documents[first_id].path
        self.assertIsNotNone(target)
        displaced = target.with_name(f"{target.name}.displaced")
        real_read = namespace["os"].read
        swapped = False

        def replace_name_after_open(fd, amount):
            nonlocal swapped
            if not swapped:
                swapped = True
                target.rename(displaced)
                shutil.copyfile(displaced, target)
            return real_read(fd, amount)

        namespace["os"].read = replace_name_after_open
        try:
            with self.assertRaises(namespace["SealedValidationSubjectChanged"]):
                session.finalize()
        finally:
            namespace["os"].read = real_read
        self.assertTrue(swapped)
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["terminal_reason"], "subject_changed")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_final_read_accounts_two_live_buffers(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        physical_bytes = (objects_root / "commits" / f"{head}.json").stat().st_size

        for limit, succeeds in [(physical_bytes * 2 - 1, False), (physical_bytes * 2, True)]:
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(),
                "final read live buffers",
            )
            session = namespace["SealedCommitValidationSession"](
                objects_root, ledger
            )
            session.load_payload(head)
            ledger._profile = namespace["DecisionWorkProfile"].local_v1(
                {"temporary_bytes": ("concurrent", limit)}
            )
            if succeeds:
                session.finalize()
            else:
                with self.assertRaises(namespace["ResourceBudgetExceeded"]) as raised:
                    session.finalize()
                self.assertEqual(
                    raised.exception.diagnostic()["dimension"], "temporary_bytes"
                )
                self.assertEqual(
                    raised.exception.diagnostic()["observed"], physical_bytes * 2
                )
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
            self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_sealed_session_continues_after_corrupt_candidate(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        bad_id = "CF-COMMIT-cccccccccccc"
        bad_path = objects_root / "commits" / f"{bad_id}.json"
        bad_path.write_text(
            json.dumps(
                {
                    "object_id": bad_id,
                    "type": "commit",
                    "hash": "wrong",
                    "payload": {"type": "commit", "parents": []},
                }
            ),
            encoding="utf-8",
        )
        ledger = namespace["DecisionWorkLedger"](
            namespace["DecisionWorkProfile"].local_v1(), "recoverable candidate"
        )
        session = namespace["SealedCommitValidationSession"](
            objects_root, ledger
        )
        session.validate_commit(head)
        cached_before = set(session.documents)
        with self.assertRaises(namespace["CFError"]):
            session.validate_commit(bad_id)
        self.assertEqual(set(session.documents), cached_before)
        session.validate_commit(head)
        self.assertGreaterEqual(ledger.snapshot()["counters"].get("revisits", 0), 1)
        session.finalize()
        snapshot = ledger.snapshot()
        self.assertEqual(snapshot["terminal_reason"], "complete")
        self.assertEqual(snapshot["current"].get("retained_bytes", 0), 0)
        self.assertEqual(snapshot["current"].get("temporary_bytes", 0), 0)

    def test_python_trace_index_1x_3x_10x_has_linear_counters(self):
        namespace = runpy.run_path(str(CODEFIRE))
        for scale in [1, 3, 10]:
            atoms = []
            links = []
            for index in range(scale):
                requirement = f"REQ-{index}"
                design = f"DES-{index}"
                atoms.extend(
                    [
                        {"atom_id": requirement, "kind": "requirement"},
                        {"atom_id": design, "kind": "design"},
                    ]
                )
                links.append(
                    {"from": requirement, "to": design, "type": "refined_by"}
                )
            atom_index = {"atoms": atoms}
            trace_graph = {"links": links}
            ledger = namespace["DecisionWorkLedger"](
                namespace["DecisionWorkProfile"].local_v1(),
                f"trace scale {scale}",
            )
            index = namespace["compile_trace_evaluation_index"](
                atom_index, trace_graph, ledger
            )
            missing = namespace["required_link_missing"](
                atom_index,
                trace_graph,
                {
                    "requirement": [
                        {
                            "type": "refined_by",
                            "target_kind": "design",
                            "min": 1,
                        }
                    ]
                },
                ledger,
                index,
            )
            self.assertEqual(missing, [])
            snapshot = ledger.snapshot()
            self.assertEqual(snapshot["cumulative"]["graph_nodes"], 2 * scale)
            self.assertEqual(snapshot["cumulative"]["graph_edges"], scale)
            self.assertEqual(snapshot["counters"]["atoms_indexed"], 2 * scale)
            self.assertEqual(snapshot["counters"]["trace_links_indexed"], scale)
            self.assertEqual(snapshot["counters"]["rules_evaluated"], scale)
            index.close()
            self.assertEqual(
                ledger.snapshot()["current"].get("retained_bytes", 0), 0
            )

    def test_python_common_ancestor_uses_public_id_as_final_tie_break(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        candidates = []
        for label in ["candidate-a", "candidate-b"]:
            payload = {
                "type": "commit",
                "version": 1,
                "parents": [],
                "generation": 7,
                "label": label,
            }
            candidates.append(self.write_commit_object(payload))
        expected = min(candidates)
        left = self.write_commit_object(
            {
                "type": "commit",
                "version": 1,
                "parents": candidates,
                "generation": 8,
                "label": "left-id-tie",
            }
        )
        right = self.write_commit_object(
            {
                "type": "commit",
                "version": 1,
                "parents": list(reversed(candidates)),
                "generation": 8,
                "label": "right-id-tie",
            }
        )

        self.assertEqual(namespace["common_ancestor"](self.tmp, left, right), expected)

    def test_python_ancestry_queue_preserves_breadth_first_parent_order(self):
        namespace = runpy.run_path(str(CODEFIRE))
        graph = {
            "head": {"parents": ["left", "right"]},
            "left": {"parents": ["left-parent"]},
            "right": {"parents": ["right-parent"]},
            "left-parent": {"parents": []},
            "right-parent": {"parents": []},
        }
        calls = []
        function = namespace["ancestor_distances_from_objects"]
        original = function.__globals__["load_object_from_root"]

        def load_graph(_objects_root, commit_id):
            calls.append(commit_id)
            return graph[commit_id]

        function.__globals__["load_object_from_root"] = load_graph
        try:
            distances = function(self.tmp, "head")
        finally:
            function.__globals__["load_object_from_root"] = original

        self.assertEqual(calls, ["head", "left", "right", "left-parent", "right-parent"])
        self.assertEqual(
            distances,
            {"head": 0, "left": 1, "right": 1, "left-parent": 2, "right-parent": 2},
        )

    def test_python_signature_history_is_iterative_for_deep_chain(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        objects_root = project_root / "objects"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "commit_signatures": {
                        "required": True,
                        "require_history": True,
                        "keys": {"alice": "signing-secret"},
                    }
                }
            ),
            encoding="utf-8",
        )
        parent = None
        for index in range(1100):
            payload = {
                "type": "commit",
                "version": 1,
                "parents": [] if parent is None else [parent],
                "label": f"signed-history-{index}",
            }
            signed = namespace["sign_commit_payload"](
                payload, "alice", "signing-secret", "alice"
            )
            parent = self.write_commit_object_to(objects_root, signed)

        namespace["enforce_remote_commit_signature_policy"](
            project_root, objects_root, parent
        )

    def test_python_signature_history_preserves_parent_error_order(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        objects_root = project_root / "objects"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "commit_signatures": {
                        "required": True,
                        "require_history": True,
                        "keys": {"alice": "signing-secret"},
                    }
                }
            ),
            encoding="utf-8",
        )
        invalid_parent = namespace["sign_commit_payload"](
            {"type": "commit", "version": 1, "parents": [], "label": "invalid"},
            "alice",
            "signing-secret",
            "alice",
        )
        invalid_parent["signature"]["signature"] = "0" * 64
        invalid_parent_id = self.write_commit_object_to(objects_root, invalid_parent)
        missing_parent_id = "CF-COMMIT-missing-parent"

        def signed_head(parents, label):
            payload = {
                "type": "commit",
                "version": 1,
                "parents": parents,
                "label": label,
            }
            return self.write_commit_object_to(
                objects_root,
                namespace["sign_commit_payload"](
                    payload, "alice", "signing-secret", "alice"
                ),
            )

        invalid_first = signed_head(
            [invalid_parent_id, missing_parent_id], "invalid-parent-first"
        )
        with self.assertRaisesRegex(namespace["CFError"], "invalid signature"):
            namespace["enforce_remote_commit_signature_policy"](
                project_root, objects_root, invalid_first
            )

        missing_first = signed_head(
            [missing_parent_id, invalid_parent_id], "missing-parent-first"
        )
        with self.assertRaisesRegex(namespace["CFError"], "object not found"):
            namespace["enforce_remote_commit_signature_policy"](
                project_root, objects_root, missing_first
            )

    def test_python_sealed_validation_is_iterative_for_deep_chain(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        objects_root = self.tmp / ".codefire/objects"
        branch = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )
        parent = branch["head"]
        template = json.loads(
            (objects_root / "commits" / f"{parent}.json").read_text(encoding="utf-8")
        )["payload"]
        for index in range(1100):
            payload = json.loads(json.dumps(template))
            payload["parents"] = [parent]
            payload["message"] = f"deep-sealed-history-{index}"
            parent = self.write_commit_object_to(objects_root, payload)

        namespace["validate_sealed_commit"](objects_root, parent)

    def test_python_sealed_validation_rejects_parent_cycles(self):
        namespace = runpy.run_path(str(CODEFIRE))
        first = "CF-COMMIT-aaaaaaaaaaaa"
        second = "CF-COMMIT-bbbbbbbbbbbb"
        commits = {
            first: {"type": "commit", "parents": [second]},
            second: {"type": "commit", "parents": [first]},
        }
        namespace["validate_sealed_commit"].__globals__["load_object_from_root"] = (
            lambda _root, oid: commits[oid]
        )

        with self.assertRaisesRegex(namespace["CFError"], "parent cycle detected"):
            namespace["validate_sealed_commit"](self.tmp, first)

    def test_python_http_upload_stages_only_reachable_existing_closure(self):
        namespace = runpy.run_path(str(CODEFIRE))
        existing_objects = self.tmp / "existing"
        staging_objects = self.tmp / "staging"

        def record(type_tag, payload, *, extensions=None):
            object_id = namespace["object_id"](type_tag, payload)
            value = {
                "object_id": object_id,
                "type": type_tag,
                "hash": namespace["object_digest"](type_tag, payload),
                "payload": payload,
            }
            if extensions is not None:
                value["extensions"] = extensions
            return value

        reachable = record(
            "blob",
            {"type": "blob", "version": 1, "encoding": "base64", "content": "eA=="},
        )
        sentinels = [
            record(
                "blob",
                {
                    "type": "blob",
                    "version": 1,
                    "encoding": "base64",
                    "content": f"sentinel-{index}",
                },
            )
            for index in range(512)
        ]
        namespace["write_object_records"](
            existing_objects, [reachable, *sentinels]
        )
        root = record(
            "policy",
            {"type": "policy", "version": 1, "policy": {"portable": True}},
            extensions=[
                {
                    "namespace": "example.test",
                    "name": "reachable-existing",
                    "version": 1,
                    "critical": False,
                    "references": [reachable["object_id"]],
                }
            ],
        )

        closure = namespace["plan_http_upload_closure"](
            existing_objects, root["object_id"], [root]
        )
        self.assertEqual(closure, ([0], [reachable["object_id"]]))
        namespace["stage_http_upload_closure"](
            existing_objects, staging_objects, [root], closure
        )

        staged_ids = {
            json.loads(path.read_text(encoding="utf-8"))["object_id"]
            for path in staging_objects.glob("*/*.json")
        }
        self.assertEqual(
            staged_ids, {root["object_id"], reachable["object_id"]}
        )
        self.assertTrue(
            all(sentinel["object_id"] not in staged_ids for sentinel in sentinels)
        )

    def test_python_http_upload_closure_rejects_missing_and_corrupt_records_without_staging(self):
        namespace = runpy.run_path(str(CODEFIRE))
        existing_objects = self.tmp / "existing"
        staging_objects = self.tmp / "staging"

        payload = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "roots": {},
        }
        root_id = namespace["object_id"]("commit", payload)
        missing_id = "CF-BLOB-missing-extension-reference"
        root = {
            "object_id": root_id,
            "type": "commit",
            "hash": namespace["object_digest"]("commit", payload),
            "payload": payload,
            "extensions": [
                {
                    "namespace": "example.test",
                    "name": "missing-existing",
                    "version": 1,
                    "critical": False,
                    "references": [missing_id],
                }
            ],
        }
        with self.assertRaisesRegex(namespace["CFError"], "object not found"):
            namespace["plan_http_upload_closure"](
                existing_objects, root_id, [root]
            )
        self.assertFalse(staging_objects.exists())
        storage_root = self.tmp / "missing-http-storage"
        with self.assertRaisesRegex(namespace["CFError"], "object not found"):
            namespace["handle_http_upload"](
                storage_root,
                "org",
                "app",
                "main",
                {"branch": "main", "head": root_id, "objects": [root]},
            )
        self.assertFalse(
            namespace["http_project_root"](
                storage_root, "org", "app"
            ).exists()
        )

        blob_payload = {
            "type": "blob",
            "version": 1,
            "encoding": "base64",
            "content": "corrupt-me",
        }
        blob_id = namespace["object_id"]("blob", blob_payload)
        blob = {
            "object_id": blob_id,
            "type": "blob",
            "hash": namespace["object_digest"]("blob", blob_payload),
            "payload": blob_payload,
        }
        namespace["write_object_records"](existing_objects, [blob])
        blob_path = namespace["object_record_path"](existing_objects, blob_id)
        corrupt = json.loads(blob_path.read_text(encoding="utf-8"))
        corrupt["hash"] = "0" * 64
        blob_path.write_text(json.dumps(corrupt), encoding="utf-8")
        root["extensions"][0]["references"] = [blob_id]

        with self.assertRaisesRegex(namespace["CFError"], "object record hash mismatch"):
            namespace["plan_http_upload_closure"](
                existing_objects, root_id, [root]
            )
        self.assertFalse(staging_objects.exists())

    def test_python_http_upload_closure_preserves_fast_forward_and_cleans_rejected_stage(self):
        self.run_cf("init")
        namespace = runpy.run_path(str(CODEFIRE))
        branch = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )
        objects = self.tmp / ".codefire/objects"
        head = branch["head"]
        records = self.reachable_object_records(objects, head)
        storage_root = self.tmp / "http-storage"
        request = {"branch": "main", "head": head, "objects": records}

        status, _ = namespace["handle_http_upload"](
            storage_root, "org", "app", "main", request
        )
        self.assertEqual(status, 200)
        project_root = namespace["http_project_root"](
            storage_root, "org", "app"
        )
        remote_branch = project_root / "branches/main.json"
        before_branch = remote_branch.read_bytes()
        before_objects = {
            path.relative_to(project_root).as_posix(): path.read_bytes()
            for path in (project_root / "objects").glob("*/*.json")
        }

        head_record = next(
            record for record in records if record["object_id"] == head
        )
        unrelated_payload = json.loads(json.dumps(head_record["payload"]))
        unrelated_payload["parents"] = []
        unrelated_payload["message"] = "unrelated closure-only upload"
        unrelated_head = namespace["object_id"]("commit", unrelated_payload)
        unrelated_record = {
            "object_id": unrelated_head,
            "type": "commit",
            "hash": namespace["object_digest"]("commit", unrelated_payload),
            "payload": unrelated_payload,
        }
        unrelated_records = [
            record for record in records if record["object_id"] != head
        ] + [unrelated_record]
        status, response = namespace["handle_http_upload"](
            storage_root,
            "org",
            "app",
            "main",
            {
                "branch": "other",
                "head": unrelated_head,
                "objects": unrelated_records,
            },
        )

        self.assertEqual(status, 409)
        self.assertIn("not an ancestor", response["error"])
        self.assertEqual(remote_branch.read_bytes(), before_branch)
        self.assertEqual(
            {
                path.relative_to(project_root).as_posix(): path.read_bytes()
                for path in (project_root / "objects").glob("*/*.json")
            },
            before_objects,
        )
        tmp_root = project_root / "tmp"
        self.assertTrue(not tmp_root.exists() or not any(tmp_root.iterdir()))

    def test_init_creates_main_and_object_store(self):
        self.run_cf("init")
        self.assertTrue((self.tmp / ".codefire/repo.json").exists())
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text())
        self.assertEqual(branch["name"], "main")
        self.assertTrue(branch["head"].startswith("CF-COMMIT-"))
        self.run_cf("doctor")

    def test_doctor_detects_missing_object_references(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        manifest_record = json.loads((self.tmp / ".codefire/objects/content_manifests" / f"{manifest_id}.json").read_text(encoding="utf-8"))
        blob_id = manifest_record["payload"]["entries"][0]["blob"]

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        (self.tmp / ".codefire/objects/blobs" / f"{blob_id}.json").unlink()
        local_doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(local_doctor.returncode, 0)
        self.assertIn("missing object references: 1", local_doctor.stdout)
        self.assertIn(f"missing object: {blob_id}", local_doctor.stdout)

        remote_blob = server / ".codefire-server/projects/org/app/objects/blobs" / f"{blob_id}.json"
        remote_blob.unlink()
        remote_doctor = self.run_cf("doctor", project_url, check=False)
        self.assertNotEqual(remote_doctor.returncode, 0)
        self.assertIn("missing object references: 1", remote_doctor.stdout)
        self.assertIn(f"missing object: {blob_id}", remote_doctor.stdout)

    def test_remote_doctor_reports_missing_project_without_traceback(self):
        project_url = f"cf://{self.tmp / 'missing-server'}/org/app"

        doctor = self.run_cf("doctor", project_url, check=False)

        self.assertEqual(doctor.returncode, 1)
        self.assertIn("CodeFire remote doctor", doctor.stdout)
        self.assertIn("project: missing", doctor.stdout)
        self.assertNotIn("Traceback", doctor.stderr)

    def test_checked_conformance_manifest_matches_python_support_decisions(self):
        root = ROOT / "schemas/conformance"
        manifest = json.loads((root / "manifest-v1.json").read_text(encoding="utf-8"))
        consumer = runpy.run_path(str(root / "minimal_consumer.py"))
        runtime = runpy.run_path(str(CODEFIRE))

        for vector in manifest["vectors"]:
            raw = (root / vector["path"]).read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), vector["sha256"])
            value = json.loads(raw)
            self.assertEqual(
                consumer["vector_decision"](vector["boundary"], value),
                vector["expected"],
            )
            if vector["boundary"] == "remote_branch":
                if vector["expected"] == "accept":
                    runtime["validate_remote_branch_record"](value)
                else:
                    with self.assertRaises(Exception):
                        runtime["validate_remote_branch_record"](value)
            elif vector["boundary"] == "object_record":
                validate = runtime["validate_object_record"]
                if vector["expected"] == "accept":
                    validate(value)
                    if value.get("extensions"):
                        value["extensions"][0]["references"] = ["CF-BLOB-extension-ref"]
                        self.assertIn(
                            "CF-BLOB-extension-ref",
                            runtime["object_record_references"](value),
                        )
                else:
                    with self.assertRaises(Exception):
                        validate(value)

    def test_doctor_detects_object_record_integrity_errors(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_path = self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json"
        commit_record = json.loads(commit_path.read_text(encoding="utf-8"))
        commit_record["hash"] = "bad-hash"
        commit_path.write_text(json.dumps(commit_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("object hash errors: 1", doctor.stdout)
        self.assertIn("bad object:", doctor.stdout)

        self.run_cf("init", str(self.tmp / "rename-repo"))
        renamed = next((self.tmp / "rename-repo/.codefire/objects/commits").glob("CF-COMMIT-*.json"))
        renamed.rename(renamed.with_name("CF-COMMIT-wrongname.json"))
        doctor_renamed = self.run_cf("doctor", cwd=self.tmp / "rename-repo", check=False)
        self.assertNotEqual(doctor_renamed.returncode, 0)
        self.assertIn("object hash errors: 1", doctor_renamed.stdout)

    def test_doctor_detects_invalid_local_sealed_commit_references(self):
        self.run_cf("init")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        branch["head"] = manifest_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 1", doctor.stdout)
        self.assertIn(f"invalid sealed ref: branch main: {manifest_id}: not a commit object", doctor.stdout)

    def test_open_and_clone_reject_invalid_local_branch_head(self):
        self.run_cf("init")
        _, bad_payload = self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        open_result = self.run_cf("open", "main", str(self.tmp / "main"), check=False)
        self.assertNotEqual(open_result.returncode, 0)
        self.assertIn("certificate is not consistent", open_result.stderr)

        clone_result = self.run_cf("clone", "main", "copy", check=False)
        self.assertNotEqual(clone_result.returncode, 0)
        self.assertIn("certificate is not consistent", clone_result.stderr)
        self.assertEqual(bad_payload["certificate"]["result"], "inconsistent")

    def test_open_directory_commands_reject_invalid_current_base_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        branch_path = self.tmp / ".codefire/branches/main.json"
        original_branch = json.loads(branch_path.read_text(encoding="utf-8"))
        bad_commit_id, _ = self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        branch_path.write_text(json.dumps(original_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        registry_path = self.tmp / ".codefire/opened/main.json"
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        registry["open"]["current_base_commit"] = bad_commit_id
        registry_path.write_text(json.dumps(registry, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        status = self.run_cf("status", cwd=main, check=False)
        self.assertNotEqual(status.returncode, 0)
        self.assertIn("open registry current_base_commit is invalid", status.stderr)
        self.assertIn("certificate is not consistent", status.stderr)

    def test_open_directory_commands_reject_invalid_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))

        status = self.run_cf("status", cwd=main, check=False)
        self.assertNotEqual(status.returncode, 0)
        self.assertIn("open branch head is invalid", status.stderr)
        self.assertIn("certificate is not consistent", status.stderr)

    def test_branch_list_rejects_invalid_local_branch_head(self):
        self.run_cf("init")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        listing = self.run_cf("branch", "list", check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("certificate is not consistent", listing.stderr)

    def test_object_graph_ignores_non_reference_cf_strings(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "verification:\n"
            "  required:\n"
            "    - id: cf-string-output\n"
            "      command: \"echo CF-NOT-A-REAL-OBJECT\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf("commit", "-m", "Message mentions CF-NOT-A-REAL-OBJECT", cwd=main)
        doctor = self.run_cf("doctor")
        self.assertIn("missing object references: 0", doctor.stdout)
        self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main")

    def test_demo_scan_fire_extinguish_verify_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        first_verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(first_verify.returncode, 0)
        self.assertIn("Open fires:", first_verify.stdout)
        scan = self.run_cf("scan", cwd=main)
        fire_ids = [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]
        self.assertGreaterEqual(len(fire_ids), 1)
        for fire_id in fire_ids:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        commit = self.run_cf("commit", "-m", "Initial consistent auth sample", cwd=main)
        self.assertIn("Sealed commit created.", commit.stdout)
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text())
        self.assertTrue(branch["head"].startswith("CF-COMMIT-"))

    def test_verify_runs_multiple_policy_commands(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "verification:\n"
            "  required:\n"
            "    - id: marker-one\n"
            "      command: \"printf ok > check_one.txt\"\n"
            "      cwd: \".\"\n"
            "    - id: marker-two\n"
            "      command: \"printf ok > check_two.txt\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.assertEqual((main / "check_one.txt").read_text(encoding="utf-8"), "ok")
        self.assertEqual((main / "check_two.txt").read_text(encoding="utf-8"), "ok")

    def test_commit_policy_can_allow_failed_verification_commands(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_verification_success: false\n"
            "verification:\n"
            "  required:\n"
            "    - id: allowed-failure\n"
            "      command: \"echo allowed failure && exit 9\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        verify = self.run_cf("verify", cwd=main)
        self.assertIn("Verification passed.", verify.stdout)
        commit = self.run_cf("commit", "-m", "Commit with allowed verification failure", cwd=main)
        commit_id = next(line.split(":", 1)[1].strip() for line in commit.stdout.splitlines() if line.startswith("Commit:"))
        record = json.loads((self.tmp / ".codefire/objects/commits" / f"{commit_id}.json").read_text(encoding="utf-8"))
        self.assertEqual(record["payload"]["certificate"]["failed_checks"], 1)
        self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main")

    def test_commit_can_store_hmac_signature(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed import", cwd=main)
        self.run_cf("verify", cwd=main)
        commit = self.run_cf(
            "commit",
            "-m",
            "Signed commit",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        commit_id = next(line.split(":", 1)[1].strip() for line in commit.stdout.splitlines() if line.startswith("Commit:"))
        record = json.loads((self.tmp / ".codefire/objects/commits" / f"{commit_id}.json").read_text(encoding="utf-8"))
        signature = record["payload"]["signature"]
        self.assertEqual(signature["algorithm"], "hmac-sha256")
        self.assertEqual(signature["signer"], "alice")
        self.assertEqual(signature["key_id"], "alice")
        self.assertEqual(len(signature["signature"]), 64)
        show = self.run_cf("show", "main")
        self.assertIn("Signature: alice (hmac-sha256)", show.stdout)

    def test_remote_requires_and_verifies_commit_signature(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial unsigned sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)

        self.write_demo_files(main, 45)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed session update", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Signed session update",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        self.run_cf("upload", "main", main_url)
        remote_show = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", remote_show.stdout)

        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        signed_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(signed_record["payload"]))
        bad_payload["signature"]["signature"] = "0" * 64
        bad_commit_id = self.write_commit_object(bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        bad_server = self.tmp / "bad-server"
        bad_project_root = bad_server / ".codefire-server/projects/org/app"
        bad_project_root.mkdir(parents=True)
        (bad_project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        invalid = self.run_cf("upload", "main", f"cf://{bad_server}/org/app/main", check=False)
        self.assertNotEqual(invalid.returncode, 0)
        self.assertIn("invalid signature", invalid.stderr)

        remote_branch_path = project_root / "branches/main.json"
        remote_branch = json.loads(remote_branch_path.read_text(encoding="utf-8"))
        remote_branch["head"] = signed_record["payload"]["parents"][0]
        remote_branch_path.write_text(json.dumps(remote_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        doctor = self.run_cf("doctor", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid commit signature references: 1", doctor.stdout)
        gc = self.run_cf("gc", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("invalid commit signatures", gc.stderr)

    def test_remote_commit_signature_key_rotation_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "old signed import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Old signed import",
            "--signer",
            "alice",
            "--key-id",
            "alice-2026-01",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "old-secret"},
        )

        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        policy = {
            "commit_signatures": {
                "required": True,
                "require_history": True,
                "keys": {
                    "alice-2026-01": {"secret": "old-secret", "signers": ["alice"], "not_after": "2099-01-01T00:00:00Z"},
                    "alice-2026-06": {"secret": "new-secret", "signers": ["alice"], "not_before": "2000-01-01T00:00:00Z"},
                },
            }
        }
        (project_root / "server_policy.json").write_text(json.dumps(policy), encoding="utf-8")
        main_url = f"cf://{server}/org/app/main"
        self.run_cf("upload", "main", main_url)
        show_old = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", show_old.stdout)

        self.write_demo_files(main, 55)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "rotated signing key", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Rotated signing key",
            "--signer",
            "alice",
            "--key-id",
            "alice-2026-06",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "new-secret"},
        )
        self.run_cf("upload", "main", main_url)

        policy["commit_signatures"]["keys"]["alice-2026-01"]["status"] = "revoked"
        (project_root / "server_policy.json").write_text(json.dumps(policy), encoding="utf-8")
        doctor = self.run_cf("doctor", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("key is revoked", doctor.stdout)
        gc = self.run_cf("gc", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("invalid commit signatures", gc.stderr)

    def test_http_remote_requires_commit_signature_on_upload(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial unsigned sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        main_url = f"{endpoint}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)

        self.write_demo_files(main, 50)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed HTTP update", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Signed HTTP update",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        self.run_cf("upload", "main", main_url)
        show = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", show.stdout)

    def test_obsolete_auto_fire_after_revert(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 15\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("FIRE-", scan.stdout)
        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 30\n",
            encoding="utf-8",
        )
        reverted = self.run_cf("scan", cwd=main)
        self.assertNotIn("FIRE-", reverted.stdout)

    def test_manual_fire_survives_scan_until_extinguished(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        created = self.run_cf("fire", "CODE-SessionPolicy", "--reason", "manual-review", cwd=main)
        self.assertIn("created FIRE-", created.stdout)
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("manual-review", self.run_cf("status", cwd=main).stdout)
        self.assertIn("Open fires:", scan.stdout)
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.run_cf("extinguish", "FIRE-001", "--resolution", "verified", "--evidence", "manual review complete", cwd=main)
        self.run_cf("verify", cwd=main)

    def test_stale_resolution_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 15\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        self.run_cf(
            "extinguish",
            fire_id,
            "--resolution",
            "no-change-required",
            "--rationale",
            "design still intentionally describes the public contract",
            cwd=main,
        )
        (main / "docs/design/auth.md").write_text(
            "## DES-AUTH-001: Session policy\n\n"
            "SessionPolicy now needs another review.\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Blocking checks:", verify.stdout)
        self.assertIn("stale resolutions", verify.stdout)
        self.assertIn("Stale resolutions: 1", verify.stdout)
        refreshed_fire = self.run_cf(
            "extinguish",
            fire_id,
            "--resolution",
            "no-change-required",
            "--rationale",
            "design change was reviewed after stale resolution detection",
            "--refresh",
            cwd=main,
        )
        self.assertIn(f"refreshed {fire_id}", refreshed_fire.stdout)
        rescan = self.run_cf("scan", cwd=main)
        for line in rescan.stdout.splitlines():
            stripped = line.strip()
            if stripped.startswith("FIRE-"):
                new_fire_id = stripped.split()[0]
                self.run_cf(
                    "extinguish",
                    new_fire_id,
                    "--resolution",
                    "changed",
                    "--evidence",
                    "reviewed newly propagated design change",
                    cwd=main,
                )
        refreshed = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(refreshed.returncode, 0, refreshed.stdout + refreshed.stderr)

    def test_duplicate_atom_id_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "docs/spec/duplicate.md").write_text(
            "## REQ-AUTH-001: Duplicate requirement\n\n"
            "This duplicates an Atom ID.\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", "--details", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Blocking checks:", verify.stdout)
        self.assertIn("duplicate atom ids", verify.stdout)
        self.assertIn("Duplicate atom ids: 1", verify.stdout)
        self.assertIn("Duplicate Atom ID details:", verify.stdout)
        self.assertIn("REQ-AUTH-001", verify.stdout)

    def test_python_methods_include_class_owner_in_derived_atom_id(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "src").mkdir(parents=True)
        (main / "src/planners.py").write_text(
            "class StaticPlanner:\n"
            "    def propose(self):\n"
            "        return []\n\n"
            "class CodexPlanner:\n"
            "    def propose(self):\n"
            "        return []\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("CODE:src/planners.py::method:StaticPlanner.propose", scan.stdout)
        self.assertIn("CODE:src/planners.py::method:CodexPlanner.propose", scan.stdout)
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stdout + verify.stderr)

    def test_policy_required_links_can_be_customized(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/spec").mkdir(parents=True)
        (main / "docs/design").mkdir(parents=True)
        (main / "docs/spec/adr.md").write_text(
            "## ADR-CUSTOM-001: Custom decision\n\n"
            "This decision must point to design.\n",
            encoding="utf-8",
        )
        (main / "docs/design/decision.md").write_text(
            "## DES-CUSTOM-001: Custom design\n\n"
            "Design linked from the decision.\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - path: docs/spec/**/*.md\n"
            "      kind: requirement_document\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "required_links:\n"
            "  adr:\n"
            "    - type: refined_by\n"
            "      target_kind: design\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Missing required links: 1", verify.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: ADR-CUSTOM-001\n"
            "    to: DES-CUSTOM-001\n"
            "    type: refined_by\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "custom policy link added", cwd=main)
        self.run_cf("verify", cwd=main)

    def test_adr_and_ops_markdown_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "adr").mkdir(parents=True)
        (main / "docs/ops").mkdir(parents=True)
        (main / "adr/platform.md").write_text(
            "## ADR-PLATFORM-001: Runtime choice\n\n"
            "The service runs on the managed runtime.\n",
            encoding="utf-8",
        )
        (main / "docs/ops/deploy.md").write_text(
            "## OPS-DEPLOY-001: Runtime deployment\n\n"
            "Deploy the managed runtime release.\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  adrs:\n"
            "    - path: adr/**/*.md\n"
            "      kind: adr_document\n"
            "  ops:\n"
            "    - path: docs/ops/**/*.md\n"
            "      kind: ops_document\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  adr:\n"
            "    - type: operated_by\n"
            "      target_kind: ops\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("ADR-PLATFORM-001", scan.stdout)
        self.assertIn("OPS-DEPLOY-001", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: ADR-PLATFORM-001\n"
            "    to: OPS-DEPLOY-001\n"
            "    type: operated_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_verify_details_reports_missing_required_links(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.links.yaml").write_text("version: 1\nlinks:\n", encoding="utf-8")

        verify = self.run_cf("verify", "--details", cwd=main, check=False)

        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Missing required link details:", verify.stdout)
        self.assertIn("REQ-AUTH-001 requires refined_by -> design min 1", verify.stdout)

    def test_openapi_json_operations_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/api").mkdir(parents=True)
        (main / "tests").mkdir(parents=True)
        (main / "docs/api/openapi.json").write_text(
            json.dumps(
                {
                    "openapi": "3.0.0",
                    "paths": {
                        "/sessions": {
                            "post": {
                                "operationId": "createSession",
                                "x-codefire-atom-id": "API-AUTH-CREATE-SESSION",
                                "summary": "Create a session",
                            }
                        }
                    },
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        (main / "tests/test_api.py").write_text(
            "# cf-atom: TEST-api-create-session\n"
            "def test_api_create_session_contract():\n"
            "    assert True\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  apis:\n"
            "    - path: docs/api/**/*.json\n"
            "      kind: openapi_json\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  api:\n"
            "    - type: verified_by\n"
            "      target_kind: test\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("API-AUTH-CREATE-SESSION", scan.stdout)
        self.assertIn("TEST-api-create-session", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: API-AUTH-CREATE-SESSION\n"
            "    to: TEST-api-create-session\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_openapi_yaml_operations_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/api").mkdir(parents=True)
        (main / "tests").mkdir(parents=True)
        (main / "docs/api/openapi.yaml").write_text(
            "openapi: 3.0.0\n"
            "paths:\n"
            "  /sessions:\n"
            "    post:\n"
            "      operationId: createSession\n"
            "      x-codefire-atom-id: API-AUTH-CREATE-SESSION-YAML\n"
            "      summary: Create a session\n"
            "    get:\n"
            "      operationId: listSessions\n",
            encoding="utf-8",
        )
        (main / "tests/test_api.py").write_text(
            "# cf-atom: TEST-api-create-session-yaml\n"
            "def test_api_create_session_contract():\n"
            "    assert True\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  apis:\n"
            "    - path: docs/api/**/*.yaml\n"
            "      kind: openapi_yaml\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  api:\n"
            "    - type: verified_by\n"
            "      target_kind: test\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("API-AUTH-CREATE-SESSION-YAML", scan.stdout)
        self.assertIn("API-listSessions", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 2", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: API-AUTH-CREATE-SESSION-YAML\n"
            "    to: TEST-api-create-session-yaml\n"
            "    type: verified_by\n"
            "  - from: API-listSessions\n"
            "    to: TEST-api-create-session-yaml\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_db_schema_tables_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "db").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "db/schema.sql").write_text(
            "CREATE TABLE IF NOT EXISTS sessions (\n"
            "  id TEXT PRIMARY KEY,\n"
            "  user_id TEXT NOT NULL,\n"
            "  expires_at TIMESTAMP,\n"
            "  CONSTRAINT sessions_user_id_fk FOREIGN KEY (user_id) REFERENCES users(id)\n"
            ");\n"
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);\n"
            "CREATE VIEW active_sessions AS\n"
            "  SELECT id, user_id FROM sessions WHERE expires_at > CURRENT_TIMESTAMP;\n"
            "CREATE TRIGGER sessions_touch AFTER UPDATE ON sessions BEGIN\n"
            "  SELECT NEW.id;\n"
            "END;\n"
            "CREATE FUNCTION calculate_session_count() RETURNS INTEGER AS $$\n"
            "BEGIN\n"
            "  RETURN 1;\n"
            "END;\n"
            "$$ LANGUAGE plpgsql;\n"
            "CREATE PROCEDURE prune_sessions() AS $$\n"
            "BEGIN\n"
            "  DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP;\n"
            "END;\n"
            "$$ LANGUAGE SQL;\n"
            "CREATE SEQUENCE IF NOT EXISTS session_events_seq START WITH 1;\n"
            "CREATE MATERIALIZED VIEW session_daily_counts AS\n"
            "  SELECT user_id, count(*) AS total FROM sessions GROUP BY user_id;\n"
            "CREATE TYPE session_status AS ENUM ('active', 'expired');\n",
            encoding="utf-8",
        )
        (main / "src/session_repo.py").write_text(
            "# cf-atom: CODE-SessionRepository\n"
            "class SessionRepository:\n"
            "    pass\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  db:\n"
            "    - path: db/**/*.sql\n"
            "      kind: sql_schema\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  db:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DB-sessions", scan.stdout)
        self.assertIn("DB-sessions.id", scan.stdout)
        self.assertIn("DB-sessions.user_id", scan.stdout)
        self.assertIn("DB-sessions.expires_at", scan.stdout)
        self.assertIn("DB-idx_sessions_user_id", scan.stdout)
        self.assertIn("DB-active_sessions", scan.stdout)
        self.assertIn("DB-sessions_touch", scan.stdout)
        self.assertIn("DB-calculate_session_count", scan.stdout)
        self.assertIn("DB-prune_sessions", scan.stdout)
        self.assertIn("DB-session_events_seq", scan.stdout)
        self.assertIn("DB-session_daily_counts", scan.stdout)
        self.assertIn("DB-session_status", scan.stdout)
        self.assertNotIn("DB-sessions.sessions_user_id_fk", scan.stdout)
        self.assertIn("CODE-SessionRepository", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 12", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DB-sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.user_id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.expires_at\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-idx_sessions_user_id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-active_sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions_touch\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-calculate_session_count\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-prune_sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_events_seq\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_daily_counts\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_status\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_typescript_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/frontend.md").write_text(
            "## DES-FRONTEND-001: Session frontend\n\n"
            "The frontend shows the current session status.\n",
            encoding="utf-8",
        )
        (main / "src/session.ts").write_text(
            "// cf-atom: CODE-SessionViewModel\n"
            "export class SessionViewModel {\n"
            "  status(): string {\n"
            "    return 'active';\n"
            "  }\n"
            "}\n\n"
            "export const formatSessionStatus = (status: string): string => {\n"
            "  return status.toUpperCase();\n"
            "};\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.ts\n"
            "      kind: typescript_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-FRONTEND-001", scan.stdout)
        self.assertIn("CODE-SessionViewModel", scan.stdout)
        self.assertIn("CODE:src/session.ts::function:formatSessionStatus", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-FRONTEND-001\n"
            "    to: CODE-SessionViewModel\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_go_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-BACKEND-001: Session backend\n\n"
            "The backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session.go").write_text(
            "package session\n\n"
            "// cf-atom: CODE-GoSessionStore\n"
            "type Store struct {\n"
            "  sessions map[string]string\n"
            "}\n\n"
            "func NewStore() *Store {\n"
            "  return &Store{sessions: map[string]string{}}\n"
            "}\n\n"
            "func (s *Store) Find(id string) string {\n"
            "  return s.sessions[id]\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.go\n"
            "      kind: go_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-BACKEND-001", scan.stdout)
        self.assertIn("CODE-GoSessionStore", scan.stdout)
        self.assertIn("CODE:src/session.go::function:NewStore", scan.stdout)
        self.assertIn("CODE:src/session.go::method:Store.Find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-BACKEND-001\n"
            "    to: CODE-GoSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_java_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-JAVA-BACKEND-001: Java session backend\n\n"
            "The Java backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.java").write_text(
            "package session;\n\n"
            "import java.util.Map;\n\n"
            "// cf-atom: CODE-JavaSessionStore\n"
            "public class SessionStore {\n"
            "  private final Map<String, String> sessions;\n\n"
            "  public SessionStore(Map<String, String> sessions) {\n"
            "    this.sessions = sessions;\n"
            "  }\n\n"
            "  public String find(String id) {\n"
            "    return sessions.get(id);\n"
            "  }\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.java\n"
            "      kind: java_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-JAVA-BACKEND-001", scan.stdout)
        self.assertIn("CODE-JavaSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.java::method:SessionStore.find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-JAVA-BACKEND-001\n"
            "    to: CODE-JavaSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_csharp_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-CSHARP-BACKEND-001: CSharp session backend\n\n"
            "The CSharp backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.cs").write_text(
            "namespace Session;\n\n"
            "// cf-atom: CODE-CSharpSessionStore\n"
            "public class SessionStore\n"
            "{\n"
            "    private readonly Dictionary<string, string> sessions;\n\n"
            "    public SessionStore(Dictionary<string, string> sessions)\n"
            "    {\n"
            "        this.sessions = sessions;\n"
            "    }\n\n"
            "    public string? Find(string id)\n"
            "    {\n"
            "        return sessions.GetValueOrDefault(id);\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.cs\n"
            "      kind: csharp_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-CSHARP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-CSharpSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.cs::method:SessionStore.Find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-CSHARP-BACKEND-001\n"
            "    to: CODE-CSharpSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_rust_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-RUST-BACKEND-001: Rust session backend\n\n"
            "The Rust backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session.rs").write_text(
            "use std::collections::HashMap;\n\n"
            "// cf-atom: CODE-RustSessionStore\n"
            "pub struct SessionStore {\n"
            "    sessions: HashMap<String, String>,\n"
            "}\n\n"
            "impl SessionStore {\n"
            "    pub fn new() -> Self {\n"
            "        Self { sessions: HashMap::new() }\n"
            "    }\n\n"
            "    pub fn find(&self, id: &str) -> Option<&String> {\n"
            "        self.sessions.get(id)\n"
            "    }\n"
            "}\n\n"
            "pub fn normalize_session_id(id: &str) -> String {\n"
            "    id.trim().to_string()\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.rs\n"
            "      kind: rust_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-RUST-BACKEND-001", scan.stdout)
        self.assertIn("CODE-RustSessionStore", scan.stdout)
        self.assertIn("CODE:src/session.rs::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session.rs::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-RUST-BACKEND-001\n"
            "    to: CODE-RustSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_kotlin_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-KOTLIN-BACKEND-001: Kotlin session backend\n\n"
            "The Kotlin backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.kt").write_text(
            "package session\n\n"
            "// cf-atom: CODE-KotlinSessionStore\n"
            "class SessionStore(private val sessions: Map<String, String>) {\n"
            "    fun find(id: String): String? {\n"
            "        return sessions[id]\n"
            "    }\n"
            "}\n\n"
            "fun normalizeSessionId(id: String): String {\n"
            "    return id.trim()\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.kt\n"
            "      kind: kotlin_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-KOTLIN-BACKEND-001", scan.stdout)
        self.assertIn("CODE-KotlinSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.kt::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.kt::function:normalizeSessionId", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-KOTLIN-BACKEND-001\n"
            "    to: CODE-KotlinSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_php_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-PHP-BACKEND-001: PHP session backend\n\n"
            "The PHP backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.php").write_text(
            "<?php\n\n"
            "namespace Session;\n\n"
            "// cf-atom: CODE-PhpSessionStore\n"
            "class SessionStore {\n"
            "    private array $sessions;\n\n"
            "    public function __construct(array $sessions) {\n"
            "        $this->sessions = $sessions;\n"
            "    }\n\n"
            "    public function find(string $id): ?string {\n"
            "        return $this->sessions[$id] ?? null;\n"
            "    }\n"
            "}\n\n"
            "function normalize_session_id(string $id): string {\n"
            "    return trim($id);\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.php\n"
            "      kind: php_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-PHP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-PhpSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.php::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.php::function:normalize_session_id", scan.stdout)
        self.assertNotIn("CODE:src/SessionStore.php::method:SessionStore.__construct", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-PHP-BACKEND-001\n"
            "    to: CODE-PhpSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_ruby_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-RUBY-BACKEND-001: Ruby session backend\n\n"
            "The Ruby backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session_store.rb").write_text(
            "module Session\n"
            "  # cf-atom: CODE-RubySessionStore\n"
            "  class SessionStore\n"
            "    def initialize(sessions)\n"
            "      @sessions = sessions\n"
            "    end\n\n"
            "    def find(id)\n"
            "      if id.nil?\n"
            "        nil\n"
            "      else\n"
            "        @sessions[id]\n"
            "      end\n"
            "    end\n"
            "  end\n"
            "end\n\n"
            "def normalize_session_id(id)\n"
            "  id.strip\n"
            "end\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.rb\n"
            "      kind: ruby_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-RUBY-BACKEND-001", scan.stdout)
        self.assertIn("CODE-RubySessionStore", scan.stdout)
        self.assertIn("CODE:src/session_store.rb::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session_store.rb::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-RUBY-BACKEND-001\n"
            "    to: CODE-RubySessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_swift_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-SWIFT-BACKEND-001: Swift session backend\n\n"
            "The Swift backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.swift").write_text(
            "import Foundation\n\n"
            "// cf-atom: CODE-SwiftSessionStore\n"
            "struct SessionStore {\n"
            "    let sessions: [String: String]\n\n"
            "    func find(id: String) -> String? {\n"
            "        return sessions[id]\n"
            "    }\n"
            "}\n\n"
            "func normalizeSessionId(_ id: String) -> String {\n"
            "    return id.trimmingCharacters(in: .whitespaces)\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.swift\n"
            "      kind: swift_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-SWIFT-BACKEND-001", scan.stdout)
        self.assertIn("CODE-SwiftSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.swift::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.swift::function:normalizeSessionId", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-SWIFT-BACKEND-001\n"
            "    to: CODE-SwiftSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_cpp_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-CPP-BACKEND-001: C++ session backend\n\n"
            "The C++ backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session_store.cpp").write_text(
            "#include <string>\n"
            "#include <unordered_map>\n\n"
            "// cf-atom: CODE-CppSessionStore\n"
            "class SessionStore {\n"
            "public:\n"
            "    std::string find(const std::string& id) const {\n"
            "        return sessions.at(id);\n"
            "    }\n\n"
            "private:\n"
            "    std::unordered_map<std::string, std::string> sessions;\n"
            "};\n\n"
            "std::string normalize_session_id(const std::string& id) {\n"
            "    return id;\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.cpp\n"
            "      kind: cpp_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-CPP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-CppSessionStore", scan.stdout)
        self.assertIn("CODE:src/session_store.cpp::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session_store.cpp::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-CPP-BACKEND-001\n"
            "    to: CODE-CppSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_invalid_codefire_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - kind: requirement_document\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.yaml", verify.stderr)
        self.assertIn("must start with 'path:'", verify.stderr)

    def test_codefire_yaml_ignores_future_top_level_sections(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        with (main / "codefire.yaml").open("a", encoding="utf-8") as f:
            f.write(
                "\n"
                "atom_id:\n"
                "  require_explicit_ids_for:\n"
                "    - requirement\n"
                "    - design\n"
            )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("FIRE-", scan.stdout)

    def test_invalid_links_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: REQ-AUTH-001\n"
            "    to: DES-AUTH-001\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.links.yaml", verify.stderr)
        self.assertIn("missing type", verify.stderr)

    def test_invalid_policy_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "required_links:\n"
            "  requirement:\n"
            "    - type: refined_by\n"
            "      target_kind: design\n"
            "      min: many\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.policy.yaml", verify.stderr)
        self.assertIn("min must be an integer", verify.stderr)

    def test_repo_lock_blocks_structural_changes(self):
        self.run_cf("init")
        lock = self.tmp / ".codefire/locks/repo.lock"
        lock.write_text("held by test\n", encoding="utf-8")
        proc = self.run_cf("clone", "main", "blocked", check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("lock is already held", proc.stderr)

    def test_close_and_close_discard(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.run_cf("close", "main")
        self.assertFalse(main.exists())

        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        self.run_cf("scan", cwd=main)
        failed = self.run_cf("close", "main", check=False)
        self.assertNotEqual(failed.returncode, 0)
        self.run_cf("close", "main", "--discard")
        self.assertFalse(main.exists())

    def test_discard_alias_closes_burning_branch(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        self.run_cf("scan", cwd=main)
        self.run_cf("discard", "main")
        self.assertFalse(main.exists())

    def test_show_and_diff_local_branches(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")

        show = self.run_cf("show", "feature-session")
        self.assertIn("Object: feature-session@CF-COMMIT-", show.stdout)
        self.assertIn("Certificate: consistent", show.stdout)

        diff = self.run_cf("diff", "main", "feature-session")
        self.assertIn("--- main@CF-COMMIT-", diff.stdout)
        self.assertIn("+++ feature-session@CF-COMMIT-", diff.stdout)
        self.assertIn("-        return 30", diff.stdout)
        self.assertIn("+        return 15", diff.stdout)

    def test_show_and_diff_reject_invalid_local_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))

        show = self.run_cf("show", "main", check=False)
        self.assertNotEqual(show.returncode, 0)
        self.assertIn("certificate is not consistent", show.stderr)
        diff = self.run_cf("diff", "main", "main", check=False)
        self.assertNotEqual(diff.returncode, 0)
        self.assertIn("certificate is not consistent", diff.stderr)

    def test_clone_and_merge_reject_open_burning_source(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.write_demo_files(feature, 15)
        self.run_cf("scan", cwd=feature)

        clone = self.run_cf("clone", "feature-session", "copy-of-burning", check=False)
        self.assertNotEqual(clone.returncode, 0)
        self.assertIn("cannot clone from branch 'feature-session' while it is open-burning", clone.stderr)

        merge = self.run_cf("merge", "feature-session", "--into", "main", check=False)
        self.assertNotEqual(merge.returncode, 0)
        self.assertIn("cannot merge from branch 'feature-session' while it is open-burning", merge.stderr)

    def test_merge_rejects_invalid_local_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        merge = self.run_cf("merge", "feature-session", "--into", "main", check=False)
        self.assertNotEqual(merge.returncode, 0)
        self.assertIn("certificate is not consistent", merge.stderr)

    def test_branch_names_with_slashes_do_not_collide(self):
        self.run_cf("init")
        self.run_cf("clone", "main", "feature/session")
        self.run_cf("clone", "main", "feature__session")
        listing = self.run_cf("branch", "list")
        self.assertIn("feature/session\t", listing.stdout)
        self.assertIn("feature__session\t", listing.stdout)
        self.assertTrue((self.tmp / ".codefire/branches/feature%2Fsession.json").exists())
        self.assertTrue((self.tmp / ".codefire/branches/feature__session.json").exists())

        feature = self.tmp / "feature-session-open"
        self.run_cf("open", "feature/session", str(feature))
        self.assertTrue((self.tmp / ".codefire/opened/feature%2Fsession.json").exists())
        status = self.run_cf("status", cwd=feature)
        self.assertIn("Branch: feature/session", status.stdout)
        scan = self.run_cf("scan", cwd=feature)
        self.assertIn("Branch state:", scan.stdout)
        self.run_cf("close", "feature/session")
        self.assertFalse(feature.exists())

    def test_branch_lock_names_with_slashes_do_not_collide(self):
        self.run_cf("init")
        stale_collision_lock = self.tmp / ".codefire/locks/branch-feature__slash.lock"
        stale_collision_lock.write_text("unrelated stale lock\n", encoding="utf-8")
        clone = self.run_cf("clone", "main", "feature/slash")
        self.assertIn("cloned main -> feature/slash", clone.stdout)
        self.assertTrue((self.tmp / ".codefire/branches/feature%2Fslash.json").exists())

    def test_remote_components_reject_encoded_separators_before_writes(self):
        namespace = runpy.run_path(str(CODEFIRE))
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        encoded_url = f"{project_url}/feature%2Fsession"
        with self.assertRaisesRegex(namespace["CFError"], "invalid remote branch component"):
            namespace["parse_cf_url"](encoded_url)
        with self.assertRaisesRegex(namespace["CFError"], "invalid remote organization component"):
            namespace["parse_cf_url"](f"cf://{server}/%2E%2E/app/main")
        self.assertFalse((server / ".codefire-server").exists())

    def test_python_remote_component_contract_matches_shared_fixture(self):
        namespace = runpy.run_path(str(CODEFIRE))
        fixture = json.loads(
            (ROOT / "schemas/fixtures/remote-components-v1.json").read_text(
                encoding="utf-8"
            )
        )

        def expand(case):
            if "value" in case:
                return case["value"]
            return case["repeat"] * case["count"]

        for case in fixture["valid"]:
            namespace["validate_remote_component"](expand(case), "fixture")
        for case in fixture["invalid"]:
            with self.assertRaises(namespace["CFError"]):
                namespace["validate_remote_component"](expand(case), "fixture")

    def test_python_remote_lock_uses_persistent_kernel_lock(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        lock_path = project_root / "locks/project-gate.lock"
        with namespace["stable_remote_lock"](lock_path):
            with self.assertRaisesRegex(namespace["CFError"], "lock is already held"):
                with namespace["stable_remote_lock"](lock_path):
                    self.fail("exclusive remote lock unexpectedly re-entered")
        self.assertTrue(lock_path.is_file())
        with namespace["stable_remote_lock"](lock_path):
            pass

    @unittest.skipUnless(hasattr(os, "symlink"), "requires symlinks")
    def test_python_remote_bootstrap_rejects_intermediate_symlink_without_outside_write(self):
        namespace = runpy.run_path(str(CODEFIRE))
        root = self.tmp / "bootstrap-link"
        outside = self.tmp / "outside"
        root.mkdir()
        outside.mkdir()
        os.symlink(outside, root / "redirect", target_is_directory=True)
        project_root = root / "redirect/projects/org/app"
        with self.assertRaises(OSError):
            namespace["ensure_remote_layout"](project_root)
        self.assertFalse((outside / "projects").exists())

    def test_python_project_gate_detects_descriptor_replacement(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        descriptor = project_root / "remote_project.json"
        with self.assertRaisesRegex(namespace["CFError"], "descriptor changed"):
            with namespace["remote_project_gate"](project_root):
                descriptor.rename(project_root / "remote_project.old.json")
                descriptor.write_text("{}\n", encoding="utf-8")

    def test_python_bootstrap_accepts_maximum_length_application_component(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org" / ("a" * 255)
        namespace["ensure_remote_layout"](project_root)
        self.assertTrue((project_root / "remote_project.json").is_file())

    @unittest.skipUnless(hasattr(os, "link"), "requires hard links")
    def test_python_rejects_hardlinked_project_descriptor(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        descriptor = project_root / "remote_project.json"
        os.link(descriptor, self.tmp / "descriptor-copy.json")
        with self.assertRaises(namespace["CFError"]):
            namespace["remote_project_stable_id"](project_root)

    def test_python_maximum_length_authorities_use_bounded_roundtrip_names(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        maximum = "a" * 255
        paths = [
            namespace["remote_branch_path"](project_root, maximum),
            namespace["remote_branch_path"](project_root, "界" * 80),
            namespace["remote_merge_request_path"](project_root, maximum),
            project_root
            / "gc"
            / namespace["remote_authority_file_name"]("gc-audit", maximum),
        ]
        for path in paths:
            self.assertLessEqual(len(path.name.encode("utf-8")), 255)
            namespace["write_json"](path, {"version": 1})
            self.assertEqual(namespace["read_json"](path), {"version": 1})

    def test_python_hashed_and_direct_authority_names_cannot_alias(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        maximum = "a" * 255
        for domain, directory, percent_encode in [
            ("branch", project_root / "branches", True),
            ("merge-request", project_root / "merge_requests", False),
            ("gc-audit", project_root / "gc", False),
        ]:
            long_name = namespace["remote_authority_file_name"](
                domain, maximum, percent_encode=percent_encode
            )
            digest = long_name.removeprefix("CF-INTERNAL-HASH-").removesuffix(".json")
            direct = f"h-{digest}"
            direct_name = namespace["remote_authority_file_name"](
                domain, direct, percent_encode=percent_encode
            )
            self.assertNotEqual(long_name, direct_name)
            namespace["write_json"](directory / long_name, {"authority": "long"})
            namespace["write_json"](directory / direct_name, {"authority": "direct"})
            self.assertEqual(namespace["read_json"](directory / long_name)["authority"], "long")
            self.assertEqual(
                namespace["read_json"](directory / direct_name)["authority"], "direct"
            )

    def test_upload_list_remote_clone_and_request_merge(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)
        remote_doctor = self.run_cf("doctor", project_url)
        self.assertIn("CodeFire remote doctor", remote_doctor.stdout)
        self.assertIn("object hash errors: 0", remote_doctor.stdout)
        listing = self.run_cf("list", project_url)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)

        clone_repo = self.tmp / "clone-repo"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-remote", cwd=clone_repo)
        self.assertIn("main-from-remote", clone.stdout)
        clone_show = self.run_cf("show", "main-from-remote", cwd=clone_repo)
        self.assertIn("Certificate: consistent", clone_show.stdout)

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")
        feature_url = f"{project_url}/feature-session"
        self.run_cf("upload", "feature-session", feature_url)

        remote_show = self.run_cf("show", feature_url)
        self.assertIn(f"Object: {feature_url}@CF-COMMIT-", remote_show.stdout)
        outside = self.tmp / "outside"
        outside.mkdir()
        remote_show_outside = self.run_cf("show", feature_url, cwd=outside)
        self.assertIn("Certificate: consistent", remote_show_outside.stdout)
        remote_diff = self.run_cf("diff", main_url, feature_url, cwd=outside)
        self.assertIn(f"--- {main_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn(f"+++ {feature_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn("+        return 15", remote_diff.stdout)

        mr = self.run_cf("request-merge", feature_url, main_url)
        self.assertIn("created MR-", mr.stdout)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        requests = self.run_cf("request-list", project_url)
        self.assertIn("open", requests.stdout)
        self.assertIn(feature_url, requests.stdout)
        review = self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
            "--comment",
            "sealed source is ready",
        )
        self.assertIn(f"approved {mr_id} by alice", review.stdout)
        approved_requests = self.run_cf("request-list", project_url)
        self.assertIn("approved", approved_requests.stdout)
        approved_source = self.run_cf("show", feature_url)
        approved_source_commit = next(
            line for line in approved_source.stdout.splitlines() if line.startswith("Commit:")
        )
        self.commit_demo_version(feature, 10, "Advance source after approval")
        self.run_cf("upload", "feature-session", feature_url)
        applied = self.run_cf("request-apply", project_url, mr_id)
        self.assertIn(f"applied {mr_id}", applied.stdout)
        target_after_apply = self.run_cf("show", main_url)
        feature_after_apply = self.run_cf("show", feature_url)
        self.assertEqual(
            next(line for line in target_after_apply.stdout.splitlines() if line.startswith("Commit:")),
            approved_source_commit,
        )
        self.assertNotEqual(
            approved_source_commit,
            next(line for line in feature_after_apply.stdout.splitlines() if line.startswith("Commit:")),
        )

        # Advancing the target branch makes the existing merge request stale.
        self.run_cf("clone", main_url, "target-advance")
        target_advance = self.tmp / "target-advance"
        self.run_cf("open", "target-advance", str(target_advance))
        self.commit_demo_version(target_advance, 45, "Advance target branch")
        self.run_cf("upload", "target-advance", main_url)
        stale_mr = self.run_cf("request-merge", feature_url, main_url)
        stale_mr_id = next(line.split()[1] for line in stale_mr.stdout.splitlines() if line.startswith("created MR-"))
        self.run_cf(
            "request-review",
            project_url,
            stale_mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
        )
        stale_requests = self.run_cf("request-list", project_url)
        self.assertIn("approved", stale_requests.stdout)
        # Move target again after approval; apply must mark the MR stale.
        self.run_cf("clone", main_url, "target-advance-again")
        target_advance_again = self.tmp / "target-advance-again"
        self.run_cf("open", "target-advance-again", str(target_advance_again))
        self.commit_demo_version(target_advance_again, 60, "Advance target branch again")
        self.run_cf("upload", "target-advance-again", main_url)
        stale_apply = self.run_cf("request-apply", project_url, stale_mr_id, check=False)
        self.assertNotEqual(stale_apply.returncode, 0)
        self.assertIn("merge request is stale", stale_apply.stderr)
        stale_review = self.run_cf(
            "request-review",
            project_url,
            stale_mr_id,
            "--reviewer",
            "bob",
            "--decision",
            "approve",
            check=False,
        )
        self.assertNotEqual(stale_review.returncode, 0)
        self.assertIn("merge request is stale", stale_review.stderr)

    def test_merge_request_subject_escrow_survives_source_removal_and_rejects_tamper(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Cross-project escrow base")

        server = self.tmp / "server"
        target_project = f"cf://{server}/org/target"
        source_project = f"cf://{server}/org/source"
        main_url = f"{target_project}/main"
        self.run_cf("upload", "main", main_url)

        self.run_cf("clone", "main", "escrow-feature")
        feature = self.tmp / "escrow-feature"
        self.run_cf("open", "escrow-feature", str(feature))
        self.commit_demo_version(feature, 15, "Cross-project escrow feature")
        feature_url = f"{source_project}/feature"
        self.run_cf("upload", "escrow-feature", feature_url)
        source_show = self.run_cf("show", feature_url)
        source_commit = next(line for line in source_show.stdout.splitlines() if line.startswith("Commit:"))

        request = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in request.stdout.splitlines() if line.startswith("created MR-"))
        mr_path = (
            server
            / ".codefire-server"
            / "projects"
            / "org"
            / "target"
            / "merge_requests"
            / f"{mr_id}.json"
        )
        legacy = json.loads(mr_path.read_bytes())
        del legacy["subject"]
        del legacy["subject_digest"]
        legacy["status"] = "approved"
        legacy["reviews"] = [
            {
                "reviewer": "legacy",
                "decision": "approve",
                "comment": "unbound legacy approval",
                "reviewed_at": "2026-07-18T00:00:00Z",
            }
        ]
        mr_path.write_text(json.dumps(legacy, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n")
        legacy_listing = self.run_cf("request-list", target_project, check=False)
        self.assertNotEqual(legacy_listing.returncode, 0)
        self.assertIn("legacy cross-project merge request", legacy_listing.stderr)
        self.run_cf(
            "request-review",
            target_project,
            mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
            "--comment",
            "review target-local escrow",
        )
        approved_bytes = mr_path.read_bytes()
        self.assertIn("subject", json.loads(approved_bytes))
        self.assertIn("legacy_review_history", json.loads(approved_bytes))
        for malformed_subject in (None, "not-an-object"):
            malformed = json.loads(approved_bytes)
            malformed["subject"] = malformed_subject
            mr_path.write_text(
                json.dumps(
                    malformed, ensure_ascii=False, sort_keys=True, separators=(",", ":")
                )
                + "\n"
            )
            malformed_bytes = mr_path.read_bytes()
            rejected = self.run_cf(
                "request-review",
                target_project,
                mr_id,
                "--reviewer",
                "mallory",
                "--decision",
                "approve",
                check=False,
            )
            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("merge request subject", rejected.stderr)
            self.assertEqual(mr_path.read_bytes(), malformed_bytes)
        mr_path.write_bytes(approved_bytes)
        escrow_target_url = f"{target_project}/escrow-main"
        self.run_cf("upload", "main", escrow_target_url)
        escrow_request = self.run_cf("request-merge", feature_url, escrow_target_url)
        escrow_mr_id = next(
            line.split()[1] for line in escrow_request.stdout.splitlines() if line.startswith("created MR-")
        )
        shutil.rmtree(server / ".codefire-server" / "projects" / "org" / "source")
        self.run_cf(
            "request-review",
            target_project,
            escrow_mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
            "--comment",
            "source project already removed",
        )
        self.run_cf("request-apply", target_project, escrow_mr_id)
        tampered = json.loads(approved_bytes)
        tampered["subject"]["source_branch"] = "different"
        mr_path.write_text(json.dumps(tampered, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n")
        subject_tamper = self.run_cf("request-apply", target_project, mr_id, check=False)
        self.assertNotEqual(subject_tamper.returncode, 0)
        self.assertIn("merge request subject", subject_tamper.stderr)

        mr_path.write_bytes(approved_bytes)
        stale_approval = json.loads(approved_bytes)
        stale_approval["reviews"].append(
            {
                "reviewer": "bob",
                "decision": "reject",
                "comment": "latest decision rejects",
                "subject_digest": stale_approval["subject_digest"],
                "reviewed_at": "2026-07-19T00:00:00Z",
            }
        )
        stale_approval["status"] = "approved"
        mr_path.write_text(
            json.dumps(stale_approval, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        )
        old_approval = self.run_cf("request-apply", target_project, mr_id, check=False)
        self.assertNotEqual(old_approval.returncode, 0)
        self.assertIn("effective review", old_approval.stderr)

        mr_path.write_bytes(approved_bytes)
        self.run_cf("request-apply", target_project, mr_id)
        target_show = self.run_cf("show", main_url)
        self.assertEqual(
            next(line for line in target_show.stdout.splitlines() if line.startswith("Commit:")),
            source_commit,
        )

    def test_merge_request_subject_matches_cross_runtime_golden_vector(self):
        namespace = runpy.run_path(str(CODEFIRE))
        fixture = json.loads((ROOT / "schemas/fixtures/remote-merge-subject-v1.json").read_text())
        self.assertEqual(
            namespace["merge_request_subject_digest"](fixture["subject"]),
            fixture["expected"]["subject_digest"],
        )
        self.assertEqual(
            namespace["merge_request_object_set_digest"](fixture["source_object_records"]),
            fixture["expected"]["source_object_set_digest"],
        )
        self.assertEqual(
            fixture["subject"]["source_object_set_digest"],
            fixture["expected"]["source_object_set_digest"],
        )
        reversed_records = list(reversed(fixture["source_object_records"]))
        self.assertEqual(
            namespace["merge_request_object_set_digest"](reversed_records),
            fixture["expected"]["source_object_set_digest"],
        )
        with self.assertRaisesRegex(namespace["CFError"], "duplicate object ID"):
            namespace["merge_request_object_set_digest"](
                reversed_records + [reversed_records[0]]
            )

    def test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = (
            self.tmp
            / "server"
            / ".codefire-server"
            / "projects"
            / "org"
            / "app"
        )
        namespace["ensure_remote_layout"](project_root)
        generation_path = project_root / "gc_state.json"
        branch_path = namespace["remote_branch_path"](project_root, "main")
        writes = [
            (
                {"kind": "generation"},
                None,
                {"current_generation": 1},
            ),
            (
                {"kind": "branch", "name": "main"},
                None,
                {"version": 1, "name": "main", "head": "CF-COMMIT-fixture"},
            ),
        ]
        execute = namespace["execute_remote_json_publication"]
        real_write_json = execute.__globals__["write_json"]
        real_durable_write = execute.__globals__[
            "write_remote_publication_json_durable"
        ]

        def crash_after_generation(path, value):
            real_durable_write(path, value)
            if Path(path) == generation_path:
                raise RuntimeError("simulated write-before-progress crash")

        execute.__globals__[
            "write_remote_publication_json_durable"
        ] = crash_after_generation
        try:
            with self.assertRaisesRegex(RuntimeError, "write-before-progress"):
                execute(
                    project_root,
                    "fixture_publish",
                    "fixture-actor",
                    {"subject": "same"},
                    writes,
                    {"result": "original"},
                    idempotency_key="fixture-key",
                )
        finally:
            execute.__globals__[
                "write_remote_publication_json_durable"
            ] = real_durable_write

        recovered = execute(
            project_root,
            "fixture_publish",
            "fixture-actor",
            {"subject": "same"},
            [
                ({"kind": "generation"}, {"different": True}, {"different": "after"}),
                (
                    {"kind": "branch", "name": "different-caller-value"},
                    None,
                    {"different": "after"},
                ),
            ],
            {"result": "different-caller-value"},
            idempotency_key="fixture-key",
        )
        self.assertEqual(recovered, {"result": "original"})
        self.assertEqual(json.loads(generation_path.read_bytes()), {"current_generation": 1})
        self.assertEqual(
            json.loads(branch_path.read_bytes())["head"], "CF-COMMIT-fixture"
        )
        self.assertFalse((project_root / "pending_transaction.json").exists())

        real_write_json(
            branch_path,
            {"version": 1, "name": "main", "head": "CF-COMMIT-later"},
        )
        replayed = execute(
            project_root,
            "fixture_publish",
            "fixture-actor",
            {"subject": "same"},
            writes,
            {"result": "ignored"},
            idempotency_key="fixture-key",
        )
        self.assertEqual(replayed, {"result": "original"})

    def test_python_completed_pending_quarantine_preserves_missing_receipt(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "server/.codefire-server/projects/org/app"
        namespace["ensure_remote_layout"](project_root)
        execute = namespace["execute_remote_json_publication"]
        writes = [
            ({"kind": "generation"}, None, {"current_generation": 1}),
            (
                {"kind": "branch", "name": "main"},
                None,
                {"version": 1, "name": "main", "head": "CF-COMMIT-fixture"},
            ),
        ]
        real_remove = execute.__globals__["remove_remote_transaction_stage"]

        def crash_before_cleanup(*_args):
            raise RuntimeError("simulated completed pending release gap")

        execute.__globals__["remove_remote_transaction_stage"] = crash_before_cleanup
        try:
            with self.assertRaisesRegex(RuntimeError, "pending release gap"):
                execute(
                    project_root,
                    "fixture_publish",
                    "fixture-actor",
                    {"subject": "quarantine"},
                    writes,
                    {"result": "original"},
                    idempotency_key="quarantine-key",
                )
        finally:
            execute.__globals__["remove_remote_transaction_stage"] = real_remove
        pending_path = project_root / "pending_transaction.json"
        pending = json.loads(pending_path.read_text())
        transaction_id = pending["transaction_id"]
        shutil.rmtree(project_root / "transactions" / transaction_id)
        quarantine = project_root / (
            f".pending_transaction.removing-{transaction_id}.json"
        )
        pending_path.rename(quarantine)
        quarantine_original = json.loads(quarantine.read_text())
        malformed_hash = json.loads(json.dumps(quarantine_original))
        malformed_hash["writes"][0]["before_absent"] = False
        malformed_hash["writes"][0]["before_hash"] = "not-a-sha256"
        quarantine.write_text(json.dumps(malformed_hash, sort_keys=True) + "\n")
        with self.assertRaisesRegex(
            namespace["CFError"], "quarantine write is malformed"
        ):
            execute(
                project_root,
                "fixture_publish",
                "fixture-actor",
                {"subject": "quarantine"},
                writes,
                {"result": "ignored"},
                idempotency_key="quarantine-key",
            )
        duplicated = json.loads(json.dumps(quarantine_original))
        duplicate_write = dict(duplicated["writes"][0])
        duplicate_write["postimage_index"] = len(duplicated["writes"])
        duplicated["writes"].append(duplicate_write)
        duplicated["published_count"] = len(duplicated["writes"])
        quarantine.write_text(json.dumps(duplicated, sort_keys=True) + "\n")
        with self.assertRaisesRegex(
            namespace["CFError"], "quarantine authorities are duplicated"
        ):
            execute(
                project_root,
                "fixture_publish",
                "fixture-actor",
                {"subject": "quarantine"},
                writes,
                {"result": "ignored"},
                idempotency_key="quarantine-key",
            )
        quarantine.write_text(json.dumps(quarantine_original, sort_keys=True) + "\n")
        receipt = namespace["remote_publication_receipt_path"](
            project_root,
            "fixture_publish",
            "fixture-actor",
            "quarantine-key",
        )
        receipt.unlink()

        with self.assertRaises(namespace["CFError"]):
            execute(
                project_root,
                "fixture_publish",
                "fixture-actor",
                {"subject": "quarantine"},
                writes,
                {"result": "ignored"},
                idempotency_key="quarantine-key",
            )
        self.assertTrue(quarantine.exists())
        self.assertFalse(pending_path.exists())

    def test_http_remote_upload_list_and_clone(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP remote sample")

        base_url = self.start_http_remote()
        project_url = f"{base_url}/org/app"
        main_url = f"{project_url}/main"
        upload = self.run_cf("upload", "main", main_url)
        self.assertIn("uploaded main@CF-COMMIT-", upload.stdout)

        listing = self.run_cf("list", project_url)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)
        remote_doctor = self.run_cf("doctor", project_url)
        self.assertIn("CodeFire remote doctor", remote_doctor.stdout)
        self.assertIn("object hash errors: 0", remote_doctor.stdout)

        clone_repo = self.tmp / "http-clone"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-http", cwd=clone_repo)
        self.assertIn("main-from-http", clone.stdout)
        show = self.run_cf("show", "main-from-http", cwd=clone_repo)
        self.assertIn("Initial HTTP remote sample", show.stdout)

        self.run_cf("clone", "main", "feature-http")
        feature = self.tmp / "feature-http"
        self.run_cf("open", "feature-http", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration over HTTP")
        feature_url = f"{project_url}/feature-http"
        self.run_cf("upload", "feature-http", feature_url)

        outside = self.tmp / "http-outside"
        outside.mkdir()
        remote_show = self.run_cf("show", feature_url, cwd=outside)
        self.assertIn(f"Object: {feature_url}@CF-COMMIT-", remote_show.stdout)
        self.assertIn("Change session expiration over HTTP", remote_show.stdout)
        remote_diff = self.run_cf("diff", main_url, feature_url, cwd=outside)
        self.assertIn(f"--- {main_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn(f"+++ {feature_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn("+        return 15", remote_diff.stdout)

        mr = self.run_cf("request-merge", feature_url, main_url)
        self.assertIn("created MR-", mr.stdout)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        requests = self.run_cf("request-list", project_url)
        self.assertIn("open", requests.stdout)
        self.assertIn(feature_url, requests.stdout)
        review = self.run_cf("request-review", project_url, mr_id, "--reviewer", "alice", "--decision", "approve")
        self.assertIn(f"approved {mr_id} by alice", review.stdout)
        approved_source = self.run_cf("show", feature_url)
        approved_source_commit = next(
            line for line in approved_source.stdout.splitlines() if line.startswith("Commit:")
        )
        self.commit_demo_version(feature, 10, "Advance HTTP source after approval")
        self.run_cf("upload", "feature-http", feature_url)
        applied = self.run_cf("request-apply", project_url, mr_id)
        self.assertIn(f"applied {mr_id}", applied.stdout)
        target_after_apply = self.run_cf("show", main_url)
        feature_after_apply = self.run_cf("show", feature_url)
        self.assertEqual(
            next(line for line in target_after_apply.stdout.splitlines() if line.startswith("Commit:")),
            approved_source_commit,
        )
        self.assertNotEqual(
            approved_source_commit,
            next(line for line in feature_after_apply.stdout.splitlines() if line.startswith("Commit:")),
        )

        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"orphan": "http"}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        dry_run = self.run_cf("gc", project_url, "--dry-run")
        self.assertIn(f"would remove {orphan_id}", dry_run.stdout)
        removed = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 1", removed.stdout)
        self.assertFalse(orphan_path.exists())
        self.assertTrue((project_root / "audit/gc.jsonl").exists())

        post_apply_repo = self.tmp / "http-post-apply"
        post_apply_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=post_apply_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.run_cf("clone", main_url, "main-after-http-apply", cwd=post_apply_repo)
        post_apply_show = self.run_cf("show", "main-after-http-apply", cwd=post_apply_repo)
        self.assertIn("Change session expiration over HTTP", post_apply_show.stdout)

    def test_remote_upload_receipt_replays_across_file_and_http_transports(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Cross-transport receipt base")
        storage = self.tmp / "shared-transport-storage"
        file_project = f"cf://{storage}/org/app"
        file_main = f"{file_project}/main"
        self.run_cf(
            "upload",
            "main",
            file_main,
            "--actor",
            "cross-transport",
            "--idempotency-key",
            "shared-upload-key",
        )
        project_root = storage / ".codefire-server/projects/org/app"
        generation_path = project_root / "gc_state.json"
        branch_path = project_root / "branches/main.json"
        generation_before = generation_path.read_bytes()
        branch_before = branch_path.read_bytes()

        endpoint = self.start_http_remote(storage_root=storage)
        http_main = f"{endpoint}/org/app/main"
        replay = self.run_cf(
            "upload",
            "main",
            http_main,
            "--actor",
            "cross-transport",
            "--idempotency-key",
            "shared-upload-key",
        )
        self.assertIn("uploaded main@CF-COMMIT-", replay.stdout)
        self.assertEqual(generation_path.read_bytes(), generation_before)
        self.assertEqual(branch_path.read_bytes(), branch_before)

        self.commit_demo_version(main, 45, "Conflicting cross-transport upload")
        conflict = self.run_cf(
            "upload",
            "main",
            http_main,
            "--actor",
            "cross-transport",
            "--idempotency-key",
            "shared-upload-key",
            check=False,
        )
        self.assertNotEqual(conflict.returncode, 0)
        self.assertIn("idempotency", conflict.stderr)
        self.assertEqual(generation_path.read_bytes(), generation_before)
        self.assertEqual(branch_path.read_bytes(), branch_before)

    def test_http_merge_request_escrow_survives_source_project_removal(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "HTTP escrow base")
        storage = self.tmp / "http-escrow-storage"
        endpoint = self.start_http_remote(storage_root=storage)
        target_project = f"{endpoint}/org/target"
        source_project = f"{endpoint}/org/source"
        target_main = f"{target_project}/main"
        self.run_cf("upload", "main", target_main)

        self.run_cf("clone", "main", "http-escrow-feature")
        feature = self.tmp / "http-escrow-feature"
        self.run_cf("open", "http-escrow-feature", str(feature))
        self.commit_demo_version(feature, 15, "HTTP escrow feature")
        source_feature = f"{source_project}/feature"
        self.run_cf("upload", "http-escrow-feature", source_feature)
        request = self.run_cf("request-merge", source_feature, target_main)
        mr_id = next(
            line.split()[1]
            for line in request.stdout.splitlines()
            if line.startswith("created MR-")
        )
        shutil.rmtree(storage / ".codefire-server/projects/org/source")

        review = self.run_cf(
            "request-review",
            target_project,
            mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
        )
        self.assertIn(f"approved {mr_id}", review.stdout)
        applied = self.run_cf("request-apply", target_project, mr_id)
        self.assertIn(f"applied {mr_id}", applied.stdout)
        shown = self.run_cf("show", target_main)
        self.assertIn("HTTP escrow feature", shown.stdout)

    def test_https_remote_upload_list_clone_and_show(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTPS remote sample")

        base_url = self.start_http_remote(tls=True)
        self.assertTrue(base_url.startswith("cf+https://"))
        project_url = f"{base_url}/org/app"
        main_url = f"{project_url}/main"
        tls_env = {"CODEFIRE_TLS_INSECURE": "1"}
        upload = self.run_cf("upload", "main", main_url, env=tls_env)
        self.assertIn("uploaded main@CF-COMMIT-", upload.stdout)
        listing = self.run_cf("list", project_url, env=tls_env)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)
        remote_show = self.run_cf("show", main_url, env=tls_env)
        self.assertIn("Initial HTTPS remote sample", remote_show.stdout)

        clone_repo = self.tmp / "https-clone"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-https", cwd=clone_repo, env=tls_env)
        self.assertIn("main-from-https", clone.stdout)

    def test_show_and_diff_reject_invalid_remote_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)

        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        bad_payload["certificate"]["result"] = "inconsistent"
        bad_commit_id = self.write_commit_object_to(project_root / "objects", bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        show = self.run_cf("show", main_url, check=False)
        self.assertNotEqual(show.returncode, 0)
        self.assertIn("certificate is not consistent", show.stderr)
        diff = self.run_cf("diff", main_url, main_url, check=False)
        self.assertNotEqual(diff.returncode, 0)
        self.assertIn("certificate is not consistent", diff.stderr)

    def test_file_remote_show_and_diff_block_on_pending_publication(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Pending reader barrier base")
        server = self.tmp / "server"
        main_url = f"cf://{server}/org/app/main"
        self.run_cf("upload", "main", main_url)
        project_root = server / ".codefire-server/projects/org/app"
        (project_root / "pending_transaction.json").write_text(
            json.dumps(
                {
                    "type": "remote_publication_transaction",
                    "version": 1,
                    "transaction_id": "TX-reader-barrier",
                },
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        shown = self.run_cf("show", main_url, check=False)
        self.assertNotEqual(shown.returncode, 0)
        self.assertIn("pending transaction", shown.stderr)
        diffed = self.run_cf("diff", main_url, main_url, check=False)
        self.assertNotEqual(diffed.returncode, 0)
        self.assertIn("pending transaction", diffed.stderr)

    def test_remote_list_rejects_invalid_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)

        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        bad_payload["certificate"]["result"] = "inconsistent"
        bad_commit_id = self.write_commit_object_to(project_root / "objects", bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        listing = self.run_cf("list", project_url, check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("certificate is not consistent", listing.stderr)

    def test_request_list_and_review_reject_invalid_merge_request_refs(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        feature_url = f"{project_url}/feature-session"
        self.run_cf("upload", "main", main_url)
        self.run_cf("upload", "feature-session", feature_url)
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))

        project_root = server / ".codefire-server/projects/org/app"
        feature_branch = json.loads((project_root / "branches/feature-session.json").read_text(encoding="utf-8"))
        feature_commit = json.loads((project_root / "objects/commits" / f"{feature_branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = feature_commit["payload"]["roots"]["content_manifest"]
        mr_path = project_root / "merge_requests" / f"{mr_id}.json"
        mr_record = json.loads(mr_path.read_text(encoding="utf-8"))
        mr_record["source_head"] = manifest_id
        mr_path.write_text(json.dumps(mr_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        listing = self.run_cf("request-list", project_url, check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("merge request validation failed: source_head: not a commit object", listing.stderr)
        review = self.run_cf("request-review", project_url, mr_id, "--reviewer", "alice", "--decision", "approve", check=False)
        self.assertNotEqual(review.returncode, 0)
        self.assertIn("merge request validation failed: source_head: not a commit object", review.stderr)

    def test_upload_rejects_non_fast_forward(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        main_url = f"cf://{server}/org/app/main"
        self.run_cf("upload", "main", main_url)

        # Advance the remote branch through another local branch.
        self.run_cf("clone", "main", "remote-advance")
        remote_advance = self.tmp / "remote-advance"
        self.run_cf("open", "remote-advance", str(remote_advance))
        self.commit_demo_version(remote_advance, 15, "Remote branch advances")
        self.run_cf("upload", "remote-advance", main_url)

        # Try uploading the older main head again.
        rejected = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("remote branch is not an ancestor", rejected.stderr)

    def test_upload_rejects_inconsistent_sealed_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_path = self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json"
        record = json.loads(commit_path.read_text(encoding="utf-8"))
        record["payload"]["certificate"]["result"] = "inconsistent"
        commit_path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("object hash mismatch", rejected.stderr)

    def test_upload_rejects_sealed_commit_with_wrong_root_type(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = dict(commit_record["payload"])
        bad_payload["roots"] = dict(commit_record["payload"]["roots"])
        bad_payload["roots"]["content_manifest"] = bad_payload["roots"]["verification"]
        bad_commit_id = self.remote_object_id("commit", bad_payload)
        bad_record = {
            "object_id": bad_commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", bad_payload),
            "payload": bad_payload,
        }
        bad_path = self.tmp / ".codefire/objects/commits" / f"{bad_commit_id}.json"
        bad_path.write_text(json.dumps(bad_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("root content_manifest has type verification, expected content_manifest", rejected.stderr)
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 1", doctor.stdout)
        self.assertIn("root content_manifest has type verification, expected content_manifest", doctor.stdout)

    def test_upload_rejects_sealed_commit_with_malformed_payload_fields(self):
        cases = [
            ("parents", lambda payload: payload.__setitem__("parents", "not-a-list"), "parents must be a list"),
            ("roots", lambda payload: payload.__setitem__("roots", []), "roots must be an object"),
            ("certificate", lambda payload: payload.__setitem__("certificate", []), "certificate must be an object"),
        ]
        for name, mutate, message in cases:
            with self.subTest(name=name):
                repo = self.tmp / name
                repo.mkdir()
                subprocess.run([str(CODEFIRE), "init"], cwd=repo, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                old_tmp = self.tmp
                self.tmp = repo
                try:
                    main = repo / "main"
                    self.run_cf("open", "main", str(main), cwd=repo)
                    self.commit_demo_version(main, 30, f"Initial sample for {name}")
                    self.replace_main_head_with_mutated_commit(mutate)
                    rejected = self.run_cf("upload", "main", f"cf://{repo / 'server'}/org/app/main", cwd=repo, check=False)
                    self.assertNotEqual(rejected.returncode, 0)
                    self.assertIn(message, rejected.stderr)
                    doctor = self.run_cf("doctor", cwd=repo, check=False)
                    self.assertNotEqual(doctor.returncode, 0)
                    self.assertIn(message, doctor.stdout)
                finally:
                    self.tmp = old_tmp

    def test_upload_rejects_sealed_commit_with_invalid_parent_history(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        head_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        parent_id = head_record["payload"]["parents"][0]
        parent_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{parent_id}.json").read_text(encoding="utf-8"))

        bad_parent_payload = json.loads(json.dumps(parent_record["payload"]))
        bad_parent_payload["certificate"]["result"] = "inconsistent"
        bad_parent_id = self.write_commit_object(bad_parent_payload)

        bad_head_payload = json.loads(json.dumps(head_record["payload"]))
        bad_head_payload["parents"] = [bad_parent_id]
        bad_head_id = self.write_commit_object(bad_head_payload)
        branch["head"] = bad_head_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("certificate is not consistent", rejected.stderr)
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("certificate is not consistent", doctor.stdout)

    def test_upload_runs_server_side_verification_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "commit-env-present",
                                "command": "test -n \"$CODEFIRE_REMOTE_COMMIT\"",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("upload", "main", f"cf://{server}/org/app/main")

        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "reject-all",
                                "command": "echo server rejected && exit 7",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "server-verify-fail")
        failing = self.tmp / "server-verify-fail"
        self.run_cf("open", "server-verify-fail", str(failing))
        self.commit_demo_version(failing, 15, "Commit rejected by server policy")
        rejected = self.run_cf("upload", "server-verify-fail", f"cf://{server}/org/app/server-verify-fail", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("server verification failed: reject-all", rejected.stderr)

    def test_server_side_verification_isolates_cwd_env_and_timeout(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial server isolation sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        checks_dir = project_root / "checks"
        checks_dir.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "isolated-env-cwd",
                                "cwd": "checks",
                                "timeout_seconds": 5,
                                "env": {"CUSTOM_ALLOWED": "yes"},
                                "command": "test \"$CUSTOM_ALLOWED\" = yes && test -z \"$SECRET_PARENT\" && pwd > marker.txt",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("upload", "main", f"cf://{server}/org/app/main", env={"SECRET_PARENT": "should-not-leak"})
        self.assertTrue((checks_dir / "marker.txt").exists())

        (project_root / "server_policy.json").write_text(
            json.dumps({"verification": {"required": [{"id": "bad-cwd", "cwd": "..", "command": "true"}]}}),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "bad-cwd")
        bad_cwd = self.tmp / "bad-cwd"
        self.run_cf("open", "bad-cwd", str(bad_cwd))
        self.commit_demo_version(bad_cwd, 31, "Server verification cwd escape")
        rejected_cwd = self.run_cf("upload", "bad-cwd", f"cf://{server}/org/app/bad-cwd", check=False)
        self.assertNotEqual(rejected_cwd.returncode, 0)
        self.assertIn("cwd escapes remote project", rejected_cwd.stderr)

        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "timeout-check",
                                "timeout_seconds": 0.1,
                                "command": "python3 -c 'import time; time.sleep(2)'",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "timeout-check")
        timeout_branch = self.tmp / "timeout-check"
        self.run_cf("open", "timeout-check", str(timeout_branch))
        self.commit_demo_version(timeout_branch, 32, "Server verification timeout")
        rejected_timeout = self.run_cf("upload", "timeout-check", f"cf://{server}/org/app/timeout-check", check=False)
        self.assertNotEqual(rejected_timeout.returncode, 0)
        self.assertIn("timed out after", rejected_timeout.stderr)

    def test_remote_permissions_gate_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"permissions": {"upload": ["alice"], "gc": ["admin"]}}),
            encoding="utf-8",
        )
        denied = self.run_cf("upload", "main", f"cf://{server}/org/app/main", "--actor", "bob", check=False)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn("permission denied", denied.stderr)
        self.run_cf("upload", "main", f"cf://{server}/org/app/main", "--actor", "alice")
        denied_gc = self.run_cf("gc", f"cf://{server}/org/app", "--actor", "alice", check=False)
        self.assertNotEqual(denied_gc.returncode, 0)
        self.assertIn("permission denied", denied_gc.stderr)

    def test_remote_branch_protection_gates_upload_and_apply(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "branch_protection": {
                        "main": {
                            "upload": ["alice"],
                            "apply": ["admin"],
                        }
                    }
                }
            ),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        denied_upload = self.run_cf("upload", "main", main_url, "--actor", "bob", check=False)
        self.assertNotEqual(denied_upload.returncode, 0)
        self.assertIn("branch protection denied", denied_upload.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice")

        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")
        feature_url = f"cf://{server}/org/app/feature"
        self.run_cf("upload", "feature", feature_url, "--actor", "bob")
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        self.run_cf("request-review", f"cf://{server}/org/app", mr_id, "--reviewer", "reviewer", "--decision", "approve")

        denied_apply = self.run_cf("request-apply", f"cf://{server}/org/app", mr_id, "--actor", "alice", check=False)
        self.assertNotEqual(denied_apply.returncode, 0)
        self.assertIn("branch protection denied", denied_apply.stderr)
        applied = self.run_cf("request-apply", f"cf://{server}/org/app", mr_id, "--actor", "admin")
        self.assertIn(f"applied {mr_id}", applied.stdout)

    def test_remote_token_auth_required_for_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "permissions": {"upload": ["alice"]},
                    "auth": {"required": True, "tokens": {"alice": "secret"}},
                }
            ),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        missing = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("token is required", missing.stderr)
        wrong = self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "wrong", check=False)
        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("invalid token", wrong.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "secret")

    def test_remote_token_auth_accepts_hashed_tokens(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "salted")
        salted = self.tmp / "salted"
        self.run_cf("open", "salted", str(salted))
        self.commit_demo_version(salted, 45, "Salted hashed token branch")

        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        alice_hash = json.loads(self.run_cf("token-hash", "secret").stdout)
        bob_hash = json.loads(self.run_cf("token-hash", "sensitive", "--salt", "pepper").stdout)
        self.assertTrue(alice_hash.startswith("sha256:"))
        self.assertEqual(bob_hash["algorithm"], "sha256")
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "permissions": {"upload": ["alice", "bob"]},
                    "auth": {"required": True, "tokens": {"alice": alice_hash, "bob": bob_hash}},
                }
            ),
            encoding="utf-8",
        )

        main_url = f"cf://{server}/org/app/main"
        wrong = self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "wrong", check=False)
        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("invalid token", wrong.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "secret")
        salted_url = f"cf://{server}/org/app/salted"
        self.run_cf("upload", "salted", salted_url, "--actor", "bob", "--token", "sensitive")

    def test_remote_request_signatures_gate_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial request signature sample")
        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 45, "Signed request feature")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "max_skew_seconds": 300,
                        "keys": {
                            "alice-req": {"secret": "alice-request-secret", "signers": ["alice"]},
                            "reviewer-req": {"secret": "review-request-secret", "signers": ["reviewer"]},
                            "admin-req": {"secret": "admin-request-secret", "signers": ["admin"]},
                        },
                    }
                }
            ),
            encoding="utf-8",
        )

        main_url = f"{project_url}/main"
        unsigned = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)
        self.run_cf(
            "upload",
            "main",
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        feature_url = f"{project_url}/feature"
        self.run_cf(
            "upload",
            "feature",
            feature_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        mr = self.run_cf(
            "request-merge",
            feature_url,
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        unsigned_review = self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "reviewer",
            "--decision",
            "approve",
            "--actor",
            "reviewer",
            check=False,
        )
        self.assertNotEqual(unsigned_review.returncode, 0)
        self.assertIn("signature is required", unsigned_review.stderr)
        self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "reviewer",
            "--decision",
            "approve",
            "--actor",
            "reviewer",
            "--request-key-id",
            "reviewer-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "review-request-secret"},
        )
        self.run_cf(
            "request-apply",
            project_url,
            mr_id,
            "--actor",
            "admin",
            "--request-key-id",
            "admin-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "admin-request-secret"},
        )
        gc = self.run_cf(
            "gc",
            project_url,
            "--dry-run",
            "--actor",
            "admin",
            "--request-key-id",
            "admin-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "admin-request-secret"},
        )
        self.assertIn("reachable_objects:", gc.stdout)

    def test_remote_request_signature_matches_shared_golden_vector(self):
        fixture = json.loads(
            (ROOT / "schemas/fixtures/remote-mutation-subject-v1.json").read_text(encoding="utf-8")
        )
        namespace = runpy.run_path(str(CODEFIRE))
        signature = namespace["sign_remote_request"](
            fixture["operation"],
            fixture["actor"],
            fixture["target"],
            fixture["payload"],
            fixture["key_id"],
            fixture["key"],
            timestamp=fixture["timestamp"],
            nonce=fixture["nonce"],
        )
        self.assertEqual(signature["payload_digest"], fixture["expected"]["payload_digest"])
        self.assertEqual(signature["subject_digest"], fixture["expected"]["subject_digest"])
        self.assertEqual(signature["signature"], fixture["expected"]["signature"])

    def test_python_remote_request_signature_rejects_payload_field_tamper_and_key_windows(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "python-remote-signatures"
        namespace["ensure_remote_layout"](project_root)
        policy_path = project_root / "server_policy.json"
        base_key = {
            "secret": "alice-request-key",
            "signers": ["alice"],
        }
        policy_path.write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "nonce_cache": False,
                        "keys": {"alice-request": base_key},
                    }
                }
            ),
            encoding="utf-8",
        )
        target = "org/app/branches/main/upload"
        payload = {
            "actor": "alice",
            "branch": "main",
            "head": "CF-COMMIT-original",
            "idempotency_key": "request-1",
            "lock": {"wait": True, "timeout_ms": 1000},
            "objects": [
                {
                    "id": "CF-BLOB-original",
                    "payload": {"type": "blob", "content": "eA=="},
                }
            ],
        }
        signature = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            payload,
            "alice-request",
            "alice-request-key",
            nonce="field-table",
        )
        mutations = []
        for label, mutate in [
            ("actor", lambda value: value.__setitem__("actor", "mallory")),
            ("branch", lambda value: value.__setitem__("branch", "other")),
            ("head", lambda value: value.__setitem__("head", "CF-COMMIT-tampered")),
            ("idempotency", lambda value: value.__setitem__("idempotency_key", "request-2")),
            ("lock", lambda value: value["lock"].__setitem__("timeout_ms", 2000)),
            ("object-id", lambda value: value["objects"][0].__setitem__("id", "CF-BLOB-tampered")),
            (
                "object-payload",
                lambda value: value["objects"][0]["payload"].__setitem__("content", "eQ=="),
            ),
        ]:
            tampered = json.loads(json.dumps(payload))
            mutate(tampered)
            mutations.append((label, tampered))
        for label, tampered in mutations:
            with self.assertRaisesRegex(namespace["CFError"], "payload digest mismatch", msg=label):
                namespace["verify_remote_request_signature"](
                    project_root,
                    "upload",
                    "alice",
                    target,
                    tampered,
                    signature,
                )
        self.assertFalse((project_root / "request_nonce_cache.json").exists())

        for field, boundary, expected in [
            ("not_before", "2100-01-01T00:00:00Z", "key is not valid yet"),
            ("not_after", "1970-01-01T00:00:00Z", "key is expired"),
        ]:
            key = dict(base_key)
            key[field] = boundary
            policy_path.write_text(
                json.dumps(
                    {
                        "request_signatures": {
                            "required": True,
                            "keys": {"alice-request": key},
                        }
                    }
                ),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(namespace["CFError"], expected):
                namespace["verify_remote_request_signature"](
                    project_root,
                    "upload",
                    "alice",
                    target,
                    payload,
                    signature,
                )
            self.assertFalse((project_root / "request_nonce_cache.json").exists())

        for malformed_policy, expected in [
            ([], "server policy must be an object"),
            ({"request_signatures": "not-an-object"}, "policy must be an object"),
            ({"request_signatures": {"required": "true"}}, "required must be a boolean"),
        ]:
            policy_path.write_text(json.dumps(malformed_policy), encoding="utf-8")
            with self.assertRaisesRegex(namespace["CFError"], expected):
                namespace["verify_remote_request_signature"](
                    project_root,
                    "upload",
                    "alice",
                    target,
                    payload,
                    None,
                )

        invalid_key = dict(base_key)
        invalid_key["not_before"] = 1
        policy_path.write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "keys": {"alice-request": invalid_key},
                    }
                }
            ),
            encoding="utf-8",
        )
        with self.assertRaisesRegex(namespace["CFError"], "not_before must be UTC seconds ending in Z"):
            namespace["verify_remote_request_signature"](
                project_root,
                "upload",
                "alice",
                target,
                payload,
                signature,
            )
        self.assertFalse((project_root / "request_nonce_cache.json").exists())

    @unittest.skipUnless(hasattr(time, "tzset"), "timezone regression requires time.tzset")
    def test_python_remote_request_signature_enforces_default_skew_in_utc(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "python-remote-signature-time"
        namespace["ensure_remote_layout"](project_root)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "nonce_cache": False,
                        "keys": {
                            "alice-request": {
                                "secret": "alice-request-key",
                                "signers": ["alice"],
                            }
                        },
                    }
                }
            ),
            encoding="utf-8",
        )
        target = "org/app/branches/main/upload"
        payload = {"branch": "main", "head": "CF-COMMIT-original"}
        stale = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            payload,
            "alice-request",
            "alice-request-key",
            timestamp="1970-01-01T00:00:00Z",
            nonce="stale-default-skew",
        )
        with self.assertRaisesRegex(namespace["CFError"], "timestamp is outside allowed skew"):
            namespace["verify_remote_request_signature"](
                project_root,
                "upload",
                "alice",
                target,
                payload,
                stale,
            )

        prior_tz = os.environ.get("TZ")
        try:
            os.environ["TZ"] = "PST8PDT"
            time.tzset()
            current = namespace["sign_remote_request"](
                "upload",
                "alice",
                target,
                payload,
                "alice-request",
                "alice-request-key",
                nonce="utc-not-local-time",
            )
            namespace["verify_remote_request_signature"](
                project_root,
                "upload",
                "alice",
                target,
                payload,
                current,
            )
        finally:
            if prior_tz is None:
                os.environ.pop("TZ", None)
            else:
                os.environ["TZ"] = prior_tz
            time.tzset()

        malformed_policy = json.loads((project_root / "server_policy.json").read_text(encoding="utf-8"))
        malformed_policy["request_signatures"]["max_skew_seconds"] = None
        (project_root / "server_policy.json").write_text(
            json.dumps(malformed_policy),
            encoding="utf-8",
        )
        with self.assertRaisesRegex(namespace["CFError"], "max_skew_seconds must be a number"):
            namespace["verify_remote_request_signature"](
                project_root,
                "upload",
                "alice",
                target,
                payload,
                current,
            )

    def test_python_remote_nonce_capacity_preserves_live_replay_memory(self):
        namespace = runpy.run_path(str(CODEFIRE))
        project_root = self.tmp / "python-remote-nonce"
        namespace["ensure_remote_layout"](project_root)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "max_skew_seconds": 300,
                        "nonce_ttl_seconds": 300,
                        "max_nonce_cache_entries": 1,
                        "keys": {
                            "alice-request": {
                                "secret": "alice-request-key",
                                "signers": ["alice"],
                            }
                        },
                    }
                }
            ),
            encoding="utf-8",
        )
        target = "org/app/branches/main/upload"
        payload = {"branch": "main", "head": "CF-COMMIT-original"}
        first = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            payload,
            "alice-request",
            "alice-request-key",
            nonce="nonce-1",
        )
        second = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            payload,
            "alice-request",
            "alice-request-key",
            nonce="nonce-2",
        )
        verify = namespace["verify_remote_request_signature"]
        verify(project_root, "upload", "alice", target, payload, first)
        changed_payload = {"branch": "main", "head": "CF-COMMIT-changed"}
        reused_nonce = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            changed_payload,
            "alice-request",
            "alice-request-key",
            timestamp=first["timestamp"],
            nonce="nonce-1",
        )
        with self.assertRaisesRegex(namespace["CFError"], "nonce has already been used"):
            verify(project_root, "upload", "alice", target, changed_payload, reused_nonce)
        with self.assertRaisesRegex(namespace["CFError"], "nonce cache capacity reached"):
            verify(project_root, "upload", "alice", target, payload, second)
        with self.assertRaisesRegex(namespace["CFError"], "nonce has already been used"):
            verify(project_root, "upload", "alice", target, payload, first)
        cache = json.loads((project_root / "request_nonce_cache.json").read_text(encoding="utf-8"))
        self.assertEqual(cache["entry_count"], 1)
        self.assertEqual(cache["max_entries"], 1)

        cache_path = project_root / "request_nonce_cache.json"
        cache_path.write_text(
            json.dumps(
                {
                    "version": 1,
                    "entries": {
                        "legacy-entry": {
                            "actor": "alice",
                            "nonce": "legacy-nonce",
                            "seen_at": "invalid",
                        }
                    },
                }
            ),
            encoding="utf-8",
        )
        before = cache_path.read_bytes()
        malformed_probe = namespace["sign_remote_request"](
            "upload",
            "alice",
            target,
            payload,
            "alice-request",
            "alice-request-key",
            nonce="nonce-3",
        )
        with self.assertRaisesRegex(namespace["CFError"], "nonce cache entry is malformed"):
            verify(project_root, "upload", "alice", target, payload, malformed_probe)
        self.assertEqual(cache_path.read_bytes(), before)

    def test_http_remote_request_signature_required_on_upload(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP request signature sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "keys": {"alice-req": {"secret": "alice-request-secret", "signers": ["alice"]}},
                    }
                }
            ),
            encoding="utf-8",
        )
        main_url = f"{endpoint}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)
        signed = self.run_cf(
            "upload",
            "main",
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        self.assertIn("uploaded main@CF-COMMIT-", signed.stdout)

    def test_http_remote_request_signature_rejects_replayed_nonce(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP replay signature sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "max_skew_seconds": 300,
                        "nonce_ttl_seconds": 300,
                        "keys": {"alice-req": {"secret": "alice-request-secret", "signers": ["alice"]}},
                    }
                }
            ),
            encoding="utf-8",
        )
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        objects_root = self.tmp / ".codefire/objects"
        payload = {
            "branch": "main",
            "head": branch["head"],
            "objects": self.reachable_object_records(objects_root, branch["head"]),
            "actor": "alice",
        }
        payload["request_signature"] = self.remote_request_signature(
            "alice",
            "upload",
            "org/app/branches/main/upload",
            payload,
            "alice-req",
            "alice-request-secret",
            nonce="replay-nonce",
        )
        upload_url = endpoint.replace("cf+http://", "http://") + "/v1/projects/org/app/branches/main/upload"
        request = urllib.request.Request(
            upload_url,
            data=json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8"),
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(request, timeout=10) as response:
            self.assertEqual(response.status, 200)
        replay = urllib.request.Request(
            upload_url,
            data=json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8"),
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        with self.assertRaises(urllib.error.HTTPError) as ctx:
            urllib.request.urlopen(replay, timeout=10)
        body = ctx.exception.read().decode("utf-8")
        self.assertIn("nonce has already been used", body)
        cache = json.loads((project_root / "request_nonce_cache.json").read_text(encoding="utf-8"))
        self.assertEqual(len(cache["entries"]), 1)

    def test_http_remote_payload_tamper_writes_no_layout_or_nonce(self):
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "keys": {
                            "alice-req": {
                                "secret": "alice-request-secret",
                                "signers": ["alice"],
                            }
                        },
                    }
                }
            ),
            encoding="utf-8",
        )
        original = {
            "branch": "main",
            "head": "CF-COMMIT-original",
            "objects": [],
            "actor": "alice",
            "idempotency_key": "request-1",
            "lock": {},
        }
        signature = self.remote_request_signature(
            "alice",
            "upload",
            "org/app/branches/main/upload",
            original,
            "alice-req",
            "alice-request-secret",
            nonce="tamper-nonce",
        )
        tampered = dict(original)
        tampered["head"] = "CF-COMMIT-tampered"
        tampered["request_signature"] = signature
        upload_url = endpoint.replace("cf+http://", "http://") + "/v1/projects/org/app/branches/main/upload"
        request = urllib.request.Request(
            upload_url,
            data=json.dumps(tampered, ensure_ascii=False, sort_keys=True).encode("utf-8"),
            method="POST",
            headers={"Content-Type": "application/json"},
        )

        with self.assertRaises(urllib.error.HTTPError) as ctx:
            urllib.request.urlopen(request, timeout=10)

        self.assertIn("payload digest mismatch", ctx.exception.read().decode("utf-8"))
        self.assertFalse((project_root / "request_nonce_cache.json").exists())
        self.assertFalse((project_root / "objects").exists())
        self.assertFalse((project_root / "branches").exists())
        self.assertFalse((project_root / "locks").exists())

    def test_remote_doctor_detects_invalid_sealed_commit_references(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        feature_url = f"{project_url}/feature"
        self.run_cf("upload", "main", main_url)
        self.run_cf("upload", "feature", feature_url)
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))

        project_root = server / ".codefire-server/projects/org/app"
        feature_branch_path = project_root / "branches/feature.json"
        feature_branch = json.loads(feature_branch_path.read_text(encoding="utf-8"))
        feature_commit_record = json.loads((project_root / "objects/commits" / f"{feature_branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = feature_commit_record["payload"]["roots"]["content_manifest"]

        main_branch_path = project_root / "branches/main.json"
        main_branch = json.loads(main_branch_path.read_text(encoding="utf-8"))
        main_branch["head"] = manifest_id
        main_branch_path.write_text(json.dumps(main_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        mr_path = project_root / "merge_requests" / f"{mr_id}.json"
        mr_record = json.loads(mr_path.read_text(encoding="utf-8"))
        mr_record["source_head"] = manifest_id
        mr_path.write_text(json.dumps(mr_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        doctor = self.run_cf("doctor", project_url, check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 2", doctor.stdout)
        self.assertIn(f"invalid sealed ref: branch main: {manifest_id}: not a commit object", doctor.stdout)
        self.assertIn(f"invalid sealed ref: merge_request {mr_id} source_head: {manifest_id}: not a commit object", doctor.stdout)

    def test_remote_gc_removes_unreachable_objects(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        dry = self.run_cf("gc", project_url, "--dry-run")
        self.assertIn(f"would remove {orphan_id}", dry.stdout)
        self.assertTrue(orphan_path.exists())
        gc = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 1", gc.stdout)
        self.assertFalse(orphan_path.exists())

    def test_remote_gc_writes_audit_log(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"audited_orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        dry = self.run_cf("gc", project_url, "--dry-run", "--actor", "auditor")
        self.assertIn("audit_log: audit/gc.jsonl", dry.stdout)
        self.assertTrue(orphan_path.exists())
        removed = self.run_cf("gc", project_url, "--actor", "auditor")
        self.assertIn("audit_log: audit/gc.jsonl", removed.stdout)
        self.assertFalse(orphan_path.exists())

        audit_path = project_root / "audit/gc.jsonl"
        entries = [json.loads(line) for line in audit_path.read_text(encoding="utf-8").splitlines()]
        self.assertEqual(len(entries), 2)
        self.assertTrue(entries[0]["dry_run"])
        self.assertFalse(entries[1]["dry_run"])
        self.assertEqual(entries[0]["actor"], "auditor")
        self.assertEqual(entries[1]["actor"], "auditor")
        self.assertEqual(entries[0]["removed_objects"][0]["object_id"], orphan_id)
        self.assertEqual(entries[1]["removed_objects"][0]["object_id"], orphan_id)
        self.assertEqual(entries[1]["objects_removed"], 1)

    def test_remote_gc_recovers_delete_before_progress_and_exact_audit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {
            "type": "policy",
            "version": 1,
            "policy": {"gc_crash_gap": True},
        }
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.write_text(
            json.dumps(orphan_record, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        namespace = runpy.run_path(str(CODEFIRE))
        gc_payload = namespace["remote_gc_payload"]
        real_delete = gc_payload.__globals__["_remote_gc_delete_candidate"]

        def crash_after_delete(root, plan_id, item):
            real_delete(root, plan_id, item)
            raise RuntimeError("simulated delete-before-progress crash")

        gc_payload.__globals__["_remote_gc_delete_candidate"] = crash_after_delete
        try:
            with self.assertRaisesRegex(RuntimeError, "delete-before-progress"):
                gc_payload(
                    project_root,
                    {"org": "org", "app": "app", "url": project_url},
                    "gc-recovery",
                    False,
                )
        finally:
            gc_payload.__globals__["_remote_gc_delete_candidate"] = real_delete

        self.assertFalse(orphan_path.exists())
        self.assertTrue((project_root / "pending_transaction.json").exists())
        recovered = gc_payload(
            project_root,
            {"org": "org", "app": "app", "url": project_url},
            "gc-recovery",
            False,
        )
        self.assertEqual(recovered["objects_removed"], 1)
        self.assertFalse((project_root / "pending_transaction.json").exists())
        exact_audits = list((project_root / "gc").glob("GC-*.json"))
        self.assertEqual(len(exact_audits), 1)
        audit_lines = (project_root / "audit/gc.jsonl").read_text(
            encoding="utf-8"
        ).splitlines()
        self.assertEqual(len(audit_lines), 1)
        self.assertEqual(json.loads(audit_lines[0])["plan_id"], exact_audits[0].stem)

    def test_remote_gc_dry_run_recovers_completed_cleanup_without_retention_write(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Dry-run GC recovery base")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"dry_gc": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.write_text(
            json.dumps(
                {
                    "object_id": orphan_id,
                    "type": "policy",
                    "hash": self.object_digest("policy", orphan_payload),
                    "payload": orphan_payload,
                },
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        namespace = runpy.run_path(str(CODEFIRE))
        gc_payload = namespace["remote_gc_payload"]
        real_remove = gc_payload.__globals__["remove_remote_transaction_stage"]
        crashed = False

        def crash_after_first_cleanup(*args):
            nonlocal crashed
            real_remove(*args)
            if not crashed:
                crashed = True
                raise RuntimeError("simulated dry-run completed cleanup crash")

        gc_payload.__globals__["remove_remote_transaction_stage"] = crash_after_first_cleanup
        try:
            with self.assertRaisesRegex(RuntimeError, "completed cleanup"):
                gc_payload(
                    project_root,
                    {"org": "org", "app": "app", "url": project_url},
                    "dry-recovery",
                    True,
                )
        finally:
            gc_payload.__globals__["remove_remote_transaction_stage"] = real_remove
        pending = json.loads((project_root / "pending_transaction.json").read_text())
        self.assertEqual((pending["status"], pending["deleted_count"]), ("completed", 0))
        recovered = gc_payload(
            project_root,
            {"org": "org", "app": "app", "url": project_url},
            "dry-recovery",
            True,
        )
        self.assertEqual(recovered["objects_removed"], 1)
        self.assertTrue(orphan_path.exists())
        self.assertFalse((project_root / "retention_index.json").exists())

    def test_remote_gc_rejects_self_consistent_tampered_decision_plan(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "GC plan authenticity base")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        namespace = runpy.run_path(str(CODEFIRE))
        with namespace["remote_project_gate"](project_root):
            plan = namespace["_remote_gc_prepare_plan"](
                project_root,
                {"org": "org", "app": "app", "url": project_url},
                "plan-tamper",
                False,
            )
            pending = namespace["_remote_gc_stage_plan"](
                project_root, "plan-tamper", False, plan
            )
        transaction_dir = project_root / "transactions" / pending["transaction_id"]
        manifest_stage = next(
            stage for stage in pending["stages"] if stage["name"].startswith("manifest-")
        )
        manifest_path = transaction_dir / manifest_stage["name"]
        manifest = json.loads(manifest_path.read_text())
        reachable = next(
            record for record in manifest["records"] if record["decision"] == "reachable"
        )
        reachable["decision"] = "delete"
        reachable["reason"] = "forged-delete"
        reachable["delete_index"] = 0
        raw = namespace["canonical_json"](manifest)
        manifest_path.write_bytes(raw)
        manifest_stage["hash"] = hashlib.sha256(raw).hexdigest()
        manifest_stage["size"] = len(raw)
        combined = []
        for stage in pending["stages"]:
            if stage["name"].startswith("manifest-"):
                combined.extend(
                    json.loads((transaction_dir / stage["name"]).read_text())["records"]
                )
        pending["inventory_fingerprint"] = namespace["_remote_json_hash"](combined)
        pending["candidate_count"] = 1
        namespace["write_remote_publication_json_durable"](
            project_root / "pending_transaction.json", pending
        )
        before_objects = sorted(
            path.relative_to(project_root).as_posix()
            for path in (project_root / "objects").glob("*/*.json")
        )
        with self.assertRaisesRegex(namespace["CFError"], "canonical planner"):
            namespace["remote_gc_payload"](
                project_root,
                {"org": "org", "app": "app", "url": project_url},
                "plan-tamper",
                False,
            )
        after_objects = sorted(
            path.relative_to(project_root).as_posix()
            for path in (project_root / "objects").glob("*/*.json")
        )
        self.assertEqual(after_objects, before_objects)
        self.assertTrue((project_root / "pending_transaction.json").exists())

    def test_remote_gc_rejects_future_planning_timestamp(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "GC timestamp authority base")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        namespace = runpy.run_path(str(CODEFIRE))
        with namespace["remote_project_gate"](project_root):
            plan = namespace["_remote_gc_prepare_plan"](
                project_root,
                {"org": "org", "app": "app", "url": project_url},
                "timestamp-tamper",
                False,
            )
            pending = namespace["_remote_gc_stage_plan"](
                project_root, "timestamp-tamper", False, plan
            )
        pending["planned_at_ns"] = time.time_ns() + 10**12
        namespace["write_remote_publication_json_durable"](
            project_root / "pending_transaction.json", pending
        )
        with self.assertRaisesRegex(namespace["CFError"], "identity is malformed"):
            namespace["remote_gc_payload"](
                project_root,
                {"org": "org", "app": "app", "url": project_url},
                "timestamp-tamper",
                False,
            )

    def test_remote_gc_recovers_candidate_quarantine_before_unlink(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "GC quarantine recovery base")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {
            "type": "policy",
            "version": 1,
            "policy": {"gc_quarantine": True},
        }
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.write_text(
            json.dumps(
                {
                    "object_id": orphan_id,
                    "type": "policy",
                    "hash": self.object_digest("policy", orphan_payload),
                    "payload": orphan_payload,
                },
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        namespace = runpy.run_path(str(CODEFIRE))
        gc_payload = namespace["remote_gc_payload"]
        real_rename = gc_payload.__globals__["_rename_directory_noreplace_at"]
        crashed = False

        def crash_after_gc_quarantine(directory_fd, source, target):
            nonlocal crashed
            real_rename(directory_fd, source, target)
            if ".gc-removing-" in target and not crashed:
                crashed = True
                raise RuntimeError("simulated GC quarantine-before-unlink crash")

        gc_payload.__globals__["_rename_directory_noreplace_at"] = (
            crash_after_gc_quarantine
        )
        try:
            with self.assertRaisesRegex(RuntimeError, "quarantine-before-unlink"):
                gc_payload(
                    project_root,
                    {"org": "org", "app": "app", "url": project_url},
                    "gc-quarantine",
                    False,
                )
        finally:
            gc_payload.__globals__["_rename_directory_noreplace_at"] = real_rename
        self.assertFalse(orphan_path.exists())
        quarantines = list(
            (project_root / "objects/policies").glob(
                f".{orphan_id}.json.gc-removing-GC-*"
            )
        )
        self.assertEqual(len(quarantines), 1)
        recovered = gc_payload(
            project_root,
            {"org": "org", "app": "app", "url": project_url},
            "gc-quarantine",
            False,
        )
        self.assertEqual(recovered["objects_removed"], 1)
        self.assertFalse(quarantines[0].exists())

    def test_remote_gc_inode_swap_is_quarantined_but_never_unlinked(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "GC inode swap base")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"victim": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_bytes = json.dumps(orphan_record, sort_keys=True).encode() + b"\n"
        orphan_path.write_bytes(orphan_bytes)
        replacement_payload = {
            "type": "policy",
            "version": 1,
            "policy": {"replacement": True},
        }
        replacement_id = self.remote_object_id("policy", replacement_payload)
        replacement_bytes = (
            json.dumps(
                {
                    "object_id": replacement_id,
                    "type": "policy",
                    "hash": self.object_digest("policy", replacement_payload),
                    "payload": replacement_payload,
                },
                sort_keys=True,
            ).encode()
            + b"\n"
        )
        namespace = runpy.run_path(str(CODEFIRE))
        gc_payload = namespace["remote_gc_payload"]
        real_rename = gc_payload.__globals__["_rename_directory_noreplace_at"]
        saved_name = f".{orphan_id}.json.race-original"
        raced = False

        def swap_before_quarantine(directory_fd, source, target):
            nonlocal raced
            if ".gc-removing-" in target and not raced:
                raced = True
                os.rename(
                    source,
                    saved_name,
                    src_dir_fd=directory_fd,
                    dst_dir_fd=directory_fd,
                )
                fd = os.open(
                    source,
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                    0o600,
                    dir_fd=directory_fd,
                )
                try:
                    os.write(fd, replacement_bytes)
                    os.fsync(fd)
                finally:
                    os.close(fd)
            real_rename(directory_fd, source, target)

        gc_payload.__globals__["_rename_directory_noreplace_at"] = swap_before_quarantine
        try:
            with self.assertRaisesRegex(
                namespace["CFError"], "quarantined candidate identity diverged"
            ):
                gc_payload(
                    project_root,
                    {"org": "org", "app": "app", "url": project_url},
                    "gc-inode-race",
                    False,
                )
        finally:
            gc_payload.__globals__["_rename_directory_noreplace_at"] = real_rename
        policies = project_root / "objects/policies"
        self.assertEqual((policies / saved_name).read_bytes(), orphan_bytes)
        quarantines = list(policies.glob(f".{orphan_id}.json.gc-removing-GC-*"))
        self.assertEqual(len(quarantines), 1)
        self.assertEqual(quarantines[0].read_bytes(), replacement_bytes)

    def test_remote_gc_preserves_future_pending_transaction(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        pending_path = project_root / "pending_transaction.json"
        future = {
            "type": "remote_gc_transaction",
            "version": 999,
            "status": "pending",
            "phase": "future",
            "transaction_id": "GC-TX-future",
        }
        pending_path.write_text(
            json.dumps(future, sort_keys=True) + "\n", encoding="utf-8"
        )

        blocked = self.run_cf("gc", project_url, check=False)
        self.assertNotEqual(blocked.returncode, 0)
        self.assertIn("future", blocked.stderr)
        self.assertEqual(json.loads(pending_path.read_text(encoding="utf-8")), future)

    def test_remote_gc_blocks_on_unhealthy_object_graph(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"

        orphan_payload = {"type": "policy", "version": 1, "policy": {"must_not_delete": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(
            json.dumps(
                {
                    "object_id": orphan_id,
                    "type": "policy",
                    "hash": self.object_digest("policy", orphan_payload),
                    "payload": orphan_payload,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

        branch = json.loads((project_root / "branches/main.json").read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        manifest_record = json.loads((project_root / "objects/content_manifests" / f"{manifest_id}.json").read_text(encoding="utf-8"))
        blob_id = manifest_record["payload"]["entries"][0]["blob"]
        (project_root / "objects/blobs" / f"{blob_id}.json").unlink()

        gc = self.run_cf("gc", project_url, check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("gc blocked: missing object references: 1", gc.stderr)
        self.assertTrue(orphan_path.exists())

    def test_remote_gc_blocks_on_invalid_sealed_refs(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        branch["head"] = manifest_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        gc = self.run_cf("gc", project_url, check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("gc blocked: invalid sealed commit references: 1", gc.stderr)

    def test_remote_gc_respects_retention_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        (project_root / "server_policy.json").write_text(
            json.dumps({"gc": {"retention_seconds": 3600}}),
            encoding="utf-8",
        )
        orphan_payload = {"type": "policy", "version": 1, "policy": {"retained_orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        retained = self.run_cf("gc", project_url)
        self.assertIn("objects_protected: 1", retained.stdout)
        self.assertIn("objects_removed: 0", retained.stdout)
        self.assertTrue(orphan_path.exists())

        old = time.time() - 7200
        os.utime(orphan_path, (old, old))
        removed = self.run_cf("gc", project_url)
        self.assertIn("objects_protected: 0", removed.stdout)
        self.assertIn("objects_removed: 1", removed.stdout)
        self.assertFalse(orphan_path.exists())

    def test_remote_gc_respects_generation_retention_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)
        project_root = server / ".codefire-server/projects/org/app"

        (project_root / "server_policy.json").write_text(
            json.dumps({"gc": {"retention_generations": 1}}),
            encoding="utf-8",
        )

        orphan_payload = {
            "type": "policy",
            "version": 1,
            "policy": {"generation_retained_orphan": True},
        }
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.write_text(
            json.dumps(orphan_record, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        protected = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 0", protected.stdout)
        protected_count = int(next(line.split(": ")[1] for line in protected.stdout.splitlines() if line.startswith("objects_protected:")))
        self.assertEqual(protected_count, 1)
        self.assertTrue(orphan_path.exists())
        self.run_cf("clone", "main", "advance-one")
        advance_one = self.tmp / "advance-one"
        self.run_cf("open", "advance-one", str(advance_one))
        self.commit_demo_version(advance_one, 31, "Advance generation one")
        self.run_cf("upload", "advance-one", main_url)

        self.run_cf("clone", "advance-one", "advance-two")
        advance_two = self.tmp / "advance-two"
        self.run_cf("open", "advance-two", str(advance_two))
        self.commit_demo_version(advance_two, 32, "Advance generation two")
        self.run_cf("upload", "advance-two", main_url)

        removed = self.run_cf("gc", project_url)
        removed_count = int(next(line.split(": ")[1] for line in removed.stdout.splitlines() if line.startswith("objects_removed:")))
        self.assertEqual(removed_count, 1)
        self.assertFalse(orphan_path.exists())

    def test_install_script_installs_codefire(self):
        prefix = self.tmp / "prefix"
        proc = subprocess.run(
            [str(INSTALL), "--prefix", str(prefix)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        installed = prefix / "bin/codefire"
        self.assertTrue(installed.exists())
        help_proc = subprocess.run(
            [str(installed), "--help"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(help_proc.returncode, 0, help_proc.stderr)
        self.assertIn("usage: codefire", help_proc.stdout)

    def test_install_script_dry_run_bad_binary_and_prefix_safety(self):
        prefix = self.tmp / "dry-prefix"
        completion_dir = self.tmp / "dry-completions"
        dry = subprocess.run(
            [
                str(INSTALL),
                "--prefix",
                str(prefix),
                "--completion",
                "bash",
                "--completion-dir",
                str(completion_dir),
                "--dry-run",
            ],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(dry.returncode, 0, dry.stderr)
        self.assertIn("dry_run: true", dry.stdout)
        self.assertIn(f"install_codefire: {prefix / 'bin/codefire'}", dry.stdout)
        self.assertFalse(prefix.exists())
        self.assertFalse(completion_dir.exists())

        bad_binary = self.tmp / "not-codefire"
        bad_binary.write_text("#!/usr/bin/env sh\necho nope\n", encoding="utf-8")
        bad_binary.chmod(0o755)
        bad = subprocess.run(
            [str(INSTALL), "--prefix", str(self.tmp / "bad-prefix"), "--binary", str(bad_binary)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertNotEqual(bad.returncode, 0)
        self.assertIn("does not look like CodeFire", bad.stderr)
        self.assertFalse((self.tmp / "bad-prefix/bin/codefire").exists())

        real_prefix = self.tmp / "real-prefix"
        real_prefix.mkdir()
        linked_prefix = self.tmp / "linked-prefix"
        linked_prefix.symlink_to(real_prefix, target_is_directory=True)
        symlink = subprocess.run(
            [str(INSTALL), "--prefix", str(linked_prefix), "--dry-run"],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertNotEqual(symlink.returncode, 0)
        self.assertIn("must not be a symlink", symlink.stderr)

    def test_install_script_completion_failure_does_not_install_binary(self):
        fake = self.tmp / "fake-codefire"
        fake.write_text(
            """#!/usr/bin/env sh
case "$1" in
  --help) echo "usage: codefire <command> [args]"; exit 0 ;;
  --version) echo "codefire foundation 1"; exit 0 ;;
  completion) echo "completion failed" >&2; exit 9 ;;
  *) exit 0 ;;
esac
""",
            encoding="utf-8",
        )
        fake.chmod(0o755)
        prefix = self.tmp / "partial-prefix"
        completion_dir = self.tmp / "partial-completions"
        proc = subprocess.run(
            [
                str(INSTALL),
                "--prefix",
                str(prefix),
                "--binary",
                str(fake),
                "--completion",
                "bash",
                "--completion-dir",
                str(completion_dir),
            ],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("completion failed", proc.stderr)
        self.assertFalse((prefix / "bin/codefire").exists())
        self.assertFalse((prefix / "bin/codefire-py").exists())

    def test_pyproject_installs_codefire_script(self):
        venv = self.tmp / "venv"
        subprocess.run(
            [sys.executable, "-m", "venv", "--system-site-packages", str(venv)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        pip = venv / "bin/pip"
        installed = venv / "bin/codefire"
        proc = subprocess.run(
            [str(pip), "install", "--no-build-isolation", "--no-deps", str(ROOT)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertTrue(installed.exists())
        help_proc = subprocess.run(
            [str(installed), "--help"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(help_proc.returncode, 0, help_proc.stderr)
        self.assertIn("usage: codefire", help_proc.stdout)

    def test_completion_command_and_install_completion(self):
        bash = self.run_cf("completion", "bash")
        self.assertIn("_codefire_complete", bash.stdout)
        self.assertIn("complete -F _codefire_complete codefire", bash.stdout)
        self.assertIn("request-apply", bash.stdout)
        self.assertIn("serve", bash.stdout)
        self.assertIn("--details", bash.stdout)

        zsh = self.run_cf("completion", "zsh")
        self.assertIn("#compdef codefire", zsh.stdout)
        self.assertIn("_codefire", zsh.stdout)
        self.assertIn("completion\\:completion", zsh.stdout)
        self.assertIn("--details[show failed verification diagnostic details]", zsh.stdout)

        prefix = self.tmp / "prefix"
        completion_dir = self.tmp / "completions"
        proc = subprocess.run(
            [str(INSTALL), "--prefix", str(prefix), "--completion", "bash", "--completion-dir", str(completion_dir)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        installed_completion = completion_dir / "codefire"
        self.assertTrue(installed_completion.exists())
        self.assertIn("_codefire_complete", installed_completion.read_text(encoding="utf-8"))

    def test_no_change_required_requires_rationale(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale", proc.stderr)

    def test_extinguish_policy_can_allow_no_change_required_evidence_only(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "extinguish_policy:\n"
            "  no-change-required:\n"
            "    requires_rationale: false\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale or --evidence", proc.stderr)
        self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", "--evidence", "reviewed linked artifacts", cwd=main)

    def test_extinguish_requires_rationale_or_evidence(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "changed", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale or --evidence", proc.stderr)
        self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "updated linked artifact", cwd=main)

    def test_clone_open_merge_and_merge_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")

        self.run_cf("merge", "feature-session", "--into", "main")
        status = self.run_cf("status", cwd=main)
        self.assertIn("State: open-burning", status.stdout)
        merge_scan = self.run_cf("scan", cwd=main)
        self.assertIn("merge_changed", merge_scan.stdout)
        for fire_id in [line.strip().split()[0] for line in merge_scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "merged feature-session", cwd=main)
        self.run_cf("verify", cwd=main)
        merge_commit = self.run_cf("commit", "-m", "Merge feature-session", cwd=main)
        merge_id = next(line.split(":", 1)[1].strip() for line in merge_commit.stdout.splitlines() if line.startswith("Commit:"))
        commit_file = next((self.tmp / ".codefire/objects/commits").glob(f"{merge_id}.json"))
        payload = json.loads(commit_file.read_text())["payload"]
        self.assertEqual(len(payload["parents"]), 2)

    def test_merge_conflict_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Feature changes session expiration to 15")

        self.commit_demo_version(main, 45, "Main changes session expiration to 45")

        merge = self.run_cf("merge", "feature-session", "--into", "main")
        self.assertIn("with conflicts", merge.stdout)
        self.assertIn("<<<<<<< target", (main / "src/auth.py").read_text(encoding="utf-8"))
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Failed checks:", verify.stdout)


if __name__ == "__main__":
    unittest.main()
