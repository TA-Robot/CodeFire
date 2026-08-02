"""CFR-168/CFB-136 production-path coverage for ``GET /branches``.

Every test drives the live ``make_http_handler`` route over a real socket so the
request-owned ``DecisionWorkLedger`` and ``HTTPRequestDeadline`` reaching branch
authority capture, sealed validation, projection, and response publication are
the handler's own objects.  No wall-clock sleep is used: the deadline is driven
by an injected monotonic clock and every budget bound is derived from a measured
baseline observation.
"""

import contextlib
import json
import os
import runpy
import shutil
import socket
import subprocess
import tempfile
import threading
import unittest
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"

BRANCH_LIST_REQUEST = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
SHARED_DEPTH = 12
HEAD_COUNT = 26
FIXED_UPDATED_AT = "2026-07-30T00:00:00Z"


class _Clock:
    """One injectable monotonic clock that never advances on its own."""

    def __init__(self):
        self.now = 0.0
        self.reads = 0

    def __call__(self):
        self.reads += 1
        return self.now


class _ExpiringClock(_Clock):
    """Expire the request deadline exactly at the ``expire_read``-th poll."""

    def __init__(self, expire_read):
        super().__init__()
        self.expire_read = expire_read

    def __call__(self):
        value = 0.011 if self.reads >= self.expire_read else 0.0
        self.reads += 1
        return value


class PythonHTTPBranchDecisionWorkTests(unittest.TestCase):
    """Exercise the live branch-list route's request-owned decision work."""

    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.template = Path(tempfile.mkdtemp(prefix="codefire-branch-template-"))
        try:
            cls._build_template()
        except BaseException:
            shutil.rmtree(cls.template, ignore_errors=True)
            raise

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.template, ignore_errors=True)

    @classmethod
    def _build_template(cls):
        """Seed one remote project whose heads share a deep commit ancestry."""
        namespace = cls.namespace
        local = cls.template / "local"
        local.mkdir()
        subprocess.run(
            [str(CODEFIRE), "init"],
            cwd=local,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        local_objects = local / ".codefire/objects"
        root_head = json.loads(
            (local / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]

        def record_for(object_id):
            path = namespace["object_record_path"](local_objects, object_id)
            return json.loads(path.read_text(encoding="utf-8"))

        records = []
        seen = set()
        stack = [root_head]
        while stack:
            object_id = stack.pop()
            if object_id in seen:
                continue
            seen.add(object_id)
            record = record_for(object_id)
            records.append(record)
            payload = record["payload"]
            if payload.get("type") == "commit":
                stack.extend(
                    parent
                    for parent in payload.get("parents", [])
                    if isinstance(parent, str)
                )
                stack.extend(
                    value
                    for value in payload.get("roots", {}).values()
                    if isinstance(value, str)
                )
            elif payload.get("type") == "content_manifest":
                stack.extend(
                    entry["blob"]
                    for entry in payload.get("entries", [])
                    if isinstance(entry, dict) and entry.get("blob")
                )
        records.sort(key=lambda record: record["object_id"])
        root_commit = record_for(root_head)

        def child_commit(parent_id, message):
            payload = dict(root_commit["payload"])
            payload["parents"] = [parent_id]
            payload["message"] = message
            return {
                "object_id": namespace["object_id"]("commit", payload),
                "type": "commit",
                "hash": namespace["object_digest"]("commit", payload),
                "payload": payload,
            }

        spine = []
        tip = root_head
        for index in range(SHARED_DEPTH):
            commit = child_commit(tip, f"shared depth {index:02d}")
            spine.append(commit)
            tip = commit["object_id"]
        heads = []
        for index in range(HEAD_COUNT):
            commit = child_commit(tip, f"branch head {index:02d}")
            spine.append(commit)
            heads.append(commit["object_id"])

        cls.all_records = records + spine
        cls.heads = heads
        cls.shared_tip = tip
        cls.root_head = root_head
        cls.branch_names = [f"b{index:02d}" for index in range(HEAD_COUNT)]
        cls.commit_count = 1 + SHARED_DEPTH + HEAD_COUNT

        storage = cls.template / "storage"
        status, payload = namespace["handle_http_upload"](
            storage,
            "org",
            "app",
            cls.branch_names[0],
            {
                "branch": cls.branch_names[0],
                "head": heads[0],
                "objects": cls.all_records,
            },
        )
        if status != 200:
            raise AssertionError(f"template upload failed: {status} {payload}")
        project_root = namespace["http_project_root"](storage, "org", "app")
        for name, head in zip(cls.branch_names[1:], heads[1:]):
            namespace["write_json"](
                namespace["remote_branch_path"](project_root, name),
                {
                    "version": 1,
                    "name": name,
                    "head": head,
                    "generation": 1,
                    "transport": "http",
                    "updated_at": FIXED_UPDATED_AT,
                    "uploaded_by": "anonymous",
                    "uploaded_from": name,
                },
            )
        shutil.rmtree(local)

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="codefire-branch-route-"))
        self.storage = self.tmp / "storage"
        shutil.copytree(self.template / "storage", self.storage)
        self.project_root = self.namespace["http_project_root"](
            self.storage, "org", "app"
        )
        self.dirs = self.namespace["remote_dirs"](self.project_root)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    # ------------------------------------------------------------------
    # harness
    # ------------------------------------------------------------------

    def profile(self, overrides=None, *, deadline_ms=None):
        """Build one HTTP profile, optionally with an injectable deadline."""
        base = self.namespace["DecisionWorkProfile"].http_v1(overrides or {})
        if deadline_ms is None:
            return base
        return self.namespace["DecisionWorkProfile"](
            dict(base.limits),
            profile_id=base.profile_id,
            request_deadline_ms=deadline_ms,
            max_active_connections=base.max_active_connections,
            max_queued_connections=base.max_queued_connections,
            applicable_commands=tuple(self.namespace["DECISION_WORK_COMMANDS"]),
            applicable_phases=tuple(self.namespace["DECISION_WORK_PHASES"]),
        )

    def exchange(
        self,
        profile=None,
        *,
        storage=None,
        clock=None,
        configure_handler=None,
        request=BRANCH_LIST_REQUEST,
    ):
        """Drive one real request through the live handler and observe its work."""
        observations = []
        options = {"_decision_work_observer": observations.append}
        if clock is not None:
            options["_monotonic_clock"] = clock
        handler = self.namespace["make_http_handler"](
            storage or self.storage, profile or self.profile(), **options
        )
        if configure_handler is not None:
            configure_handler(handler)
        server = self.namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        thread = threading.Thread(target=server.handle_request, daemon=True)
        thread.start()
        try:
            with socket.create_connection(server.server_address, timeout=30) as client:
                client.settimeout(30)
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
            thread.join(timeout=30)
        self.assertFalse(thread.is_alive(), "branch-list handler did not terminate")
        self.assertEqual(len(observations), 1)
        return b"".join(chunks), observations[0]

    @contextlib.contextmanager
    def patched(self, container, name, value):
        """Install one production-namespace override and always restore it."""
        original = container[name] if isinstance(container, dict) else getattr(
            container, name
        )
        if isinstance(container, dict):
            container[name] = value
        else:
            setattr(container, name, value)
        try:
            yield original
        finally:
            if isinstance(container, dict):
                container[name] = original
            else:
                setattr(container, name, original)

    @property
    def route_globals(self):
        return self.namespace["collect_http_branch_list_payload"].__globals__

    @contextlib.contextmanager
    def tracked_sessions(self):
        """Collect every sealed-validation session opened inside the block."""
        Session = self.route_globals["SealedCommitValidationSession"]
        original_init = Session.__init__
        sessions = []

        def recording_init(owner, objects_root, ledger, **kwargs):
            sessions.append(owner)
            return original_init(owner, objects_root, ledger, **kwargs)

        with self.patched(Session, "__init__", recording_init):
            yield sessions

    @contextlib.contextmanager
    def tracked_ledgers(self):
        """Collect every decision-work ledger constructed inside the block."""
        Ledger = self.route_globals["DecisionWorkLedger"]
        original_init = Ledger.__init__
        ledgers = []

        def recording_init(owner, *args, **kwargs):
            ledgers.append(owner)
            return original_init(owner, *args, **kwargs)

        with self.patched(Ledger, "__init__", recording_init):
            yield ledgers

    def assert_sessions_closed(self, sessions):
        """No sealed-validation session may survive the request."""
        self.assertEqual(len(sessions), 1, "route opened more than one session")
        for session in sessions:
            self.assertTrue(session.closed, "a sealed-validation session stayed open")
            self.assertFalse(session.documents)
            self.assertEqual(session._frame_bytes, 0)
            self.assertEqual(session._pending_temporary_bytes, 0)
            self.assertEqual(session._pending_document_retained_bytes, 0)

    def body_of(self, wire):
        self.assertIn(b"\r\n\r\n", wire)
        return json.loads(wire.split(b"\r\n\r\n", 1)[1])

    def status_line(self, wire):
        return wire.split(b"\r\n", 1)[0]

    def stable_response(self, wire):
        """Split one response into its date-independent status/header/body parts."""
        head, _, body = wire.partition(b"\r\n\r\n")
        lines = [
            line
            for line in head.split(b"\r\n")
            if not line.startswith(b"Date: ")
        ]
        return tuple(lines), body

    def assert_same_response(self, first, second):
        """Two responses agree on everything but the volatile Date header."""
        self.assertEqual(self.stable_response(first), self.stable_response(second))

    def assert_ok(self, wire):
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 200 OK")
        return self.body_of(wire)

    def assert_no_ownership_debt(self, observation):
        """No retained/temporary debt, no open permit, no second ledger."""
        snapshot = observation["snapshot"]
        self.assertTrue(
            all(value == 0 for value in snapshot["current"].values()),
            f"concurrent ownership debt remained: {snapshot['current']}",
        )
        self.assertEqual(observation["open_permit_count"], 0)

    def assert_no_partial_response(self, wire, observation):
        """A failed request publishes a diagnostic envelope, never a projection."""
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        self.assertNotIn(b'"branches"', wire)
        self.assert_no_ownership_debt(observation)

    def record_deadline_phases(self, profile):
        """Return the exact ordered deadline-poll phases of one baseline run."""
        deadline = self.route_globals["HTTPRequestDeadline"]
        original_check = deadline.check
        phases = []

        def record_check(owner, phase):
            phases.append(phase)
            return original_check(owner, phase)

        with self.patched(deadline, "check", record_check):
            wire, observation = self.exchange(profile, clock=_Clock())
        self.assert_ok(wire)
        self.assertEqual(observation["deadline"]["poll_count"], len(phases))
        self.assertIsNone(observation["deadline"]["expired_phase"])
        return phases, observation

    def only_branch(self, name):
        """Reduce the copied authority snapshot to a single branch record."""
        keep = self.namespace["remote_branch_path"](self.project_root, name)
        for path in sorted(self.dirs["branches"].glob("*.json")):
            if path != keep:
                path.unlink()

    def clear_branches(self):
        for path in sorted(self.dirs["branches"].glob("*.json")):
            path.unlink()

    def write_branch_record(self, name, record):
        path = self.namespace["remote_branch_path"](self.project_root, name)
        path.write_text(
            json.dumps(record, ensure_ascii=False, sort_keys=True), encoding="utf-8"
        )

    def object_path(self, object_id):
        return self.namespace["object_record_path"](self.dirs["objects"], object_id)

    # ------------------------------------------------------------------
    # request-owned ledger and deadline identity
    # ------------------------------------------------------------------

    def test_route_threads_the_exact_request_ledger_and_deadline(self):
        """Every branch-list stage runs on the handler's own ledger/deadline."""
        globals_ = self.route_globals
        seen = {}
        sessions = []
        projections = []
        list_args = []
        collect_args = []
        created_ledgers = []

        Ledger = globals_["DecisionWorkLedger"]
        original_ledger_init = Ledger.__init__

        def counting_init(owner, *args, **kwargs):
            created_ledgers.append(owner)
            return original_ledger_init(owner, *args, **kwargs)

        Session = globals_["SealedCommitValidationSession"]
        original_session_init = Session.__init__

        def recording_session_init(owner, objects_root, ledger, **kwargs):
            sessions.append(owner)
            return original_session_init(owner, objects_root, ledger, **kwargs)

        Projection = globals_["AccountedHTTPBranchList"]
        original_projection_init = Projection.__init__

        def recording_projection_init(owner, ledger, branches, retained_bytes):
            projections.append((owner, ledger))
            return original_projection_init(owner, ledger, branches, retained_bytes)

        original_list = globals_["handle_http_list"]

        def recording_list(storage_root, org, app, ledger, deadline=None):
            list_args.append((ledger, deadline))
            return original_list(storage_root, org, app, ledger, deadline)

        original_collect = globals_["collect_http_branch_list_payload"]

        def recording_collect(branches_root, objects_root, ledger, deadline=None):
            collect_args.append((ledger, deadline))
            return original_collect(branches_root, objects_root, ledger, deadline)

        def forbidden_validate(*args, **kwargs):
            raise AssertionError(
                "live branch-list route used an independent sealed-validation cache"
            )

        def configure(handler):
            original_do_get = handler.do_GET

            def do_GET(owner):
                seen["ledger"] = owner.decision_work
                seen["deadline"] = owner.request_deadline
                return original_do_get(owner)

            handler.do_GET = do_GET

        profile = self.profile(deadline_ms=30_000)
        with contextlib.ExitStack() as stack:
            stack.enter_context(self.patched(Ledger, "__init__", counting_init))
            stack.enter_context(
                self.patched(Session, "__init__", recording_session_init)
            )
            stack.enter_context(
                self.patched(Projection, "__init__", recording_projection_init)
            )
            stack.enter_context(self.patched(globals_, "handle_http_list", recording_list))
            stack.enter_context(
                self.patched(
                    globals_, "collect_http_branch_list_payload", recording_collect
                )
            )
            stack.enter_context(
                self.patched(globals_, "validate_sealed_commit", forbidden_validate)
            )
            wire, observation = self.exchange(
                profile, clock=_Clock(), configure_handler=configure
            )

        body = self.assert_ok(wire)
        self.assertEqual(len(body["branches"]), HEAD_COUNT)

        ledger = seen["ledger"]
        deadline = seen["deadline"]
        self.assertIsNotNone(ledger)
        self.assertIsNotNone(deadline)

        # Exactly one decision-work ledger exists for the whole request: the
        # route owns no hidden second ledger and no nested local ledger.
        self.assertEqual(len(created_ledgers), 1)
        self.assertIs(created_ledgers[0], ledger)

        self.assertEqual(len(list_args), 1)
        self.assertEqual(len(collect_args), 1)
        self.assertIs(list_args[0][0], ledger)
        self.assertIs(list_args[0][1], deadline)
        self.assertIs(collect_args[0][0], ledger)
        self.assertIs(collect_args[0][1], deadline)

        # One request-scoped sealed-validation session, on the same ledger.
        self.assertEqual(len(sessions), 1)
        self.assertIs(sessions[0].ledger, ledger)
        self.assertTrue(sessions[0].closed)
        self.assertTrue(sessions[0].finalized)

        # One result projection, owned by the same ledger.
        self.assertEqual(len(projections), 1)
        self.assertIs(projections[0][1], ledger)

        self.assertEqual(deadline.limit_ms, 30_000)
        self.assertGreater(deadline.poll_count, 0)
        self.assertEqual(
            observation["deadline"]["poll_count"], deadline.poll_count
        )
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_ownership_debt(observation)

    # ------------------------------------------------------------------
    # one shared session across every head
    # ------------------------------------------------------------------

    def test_deep_shared_ancestry_validates_once_under_one_session(self):
        """26 heads over a depth-12 shared spine share one traversal."""
        self.assertGreaterEqual(HEAD_COUNT, 24)
        with self.tracked_sessions() as sessions:
            wire, observation = self.exchange()
        body = self.assert_ok(wire)

        self.assert_sessions_closed(sessions)
        self.assertTrue(sessions[0].finalized)
        names = [entry["name"] for entry in body["branches"]]
        self.assertEqual(names, sorted(self.branch_names))
        self.assertEqual(
            [entry["head"] for entry in body["branches"]],
            [
                head
                for _, head in sorted(zip(self.branch_names, self.heads))
            ],
        )

        snapshot = observation["snapshot"]
        cumulative = snapshot["cumulative"]
        counters = snapshot["counters"]

        # Shared immutable ancestry is read and validated once per object, not
        # once per head: the unique-object and unique-commit totals are exact.
        self.assertEqual(cumulative["object_records"], len(self.all_records))
        self.assertEqual(counters["record_reads"], len(self.all_records))
        self.assertEqual(counters["memo_misses"], len(self.all_records))
        self.assertEqual(cumulative["graph_nodes"], self.commit_count)
        self.assertEqual(
            cumulative["snapshot_revalidation_records"], len(self.all_records)
        )
        self.assertEqual(snapshot["maximum"]["logical_depth"], SHARED_DEPTH + 1)
        self.assertGreater(counters["memo_hits"], HEAD_COUNT)

        # Measure the cold, one-session-per-head cost the defect produced.
        cold = self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(),
            "cold per-head probe",
            command_kind="http_request",
        )
        for head in self.heads:
            session = self.namespace["SealedCommitValidationSession"](
                self.dirs["objects"], cold
            )
            try:
                session.validate_commit(head)
                session.finalize(terminate=False)
            finally:
                session.close(None)
        cold_cumulative = cold.snapshot()["cumulative"]
        self.assertGreater(
            cold_cumulative["object_records"], cumulative["object_records"]
        )
        self.assertGreater(cold_cumulative["graph_nodes"], cumulative["graph_nodes"])
        self.assertGreater(
            cold_cumulative["snapshot_revalidation_records"],
            cumulative["snapshot_revalidation_records"],
        )
        # A per-head traversal re-reads the shared spine for every head.
        self.assertGreaterEqual(
            cold_cumulative["object_records"],
            HEAD_COUNT * (SHARED_DEPTH + 1),
        )
        self.assert_no_ownership_debt(observation)

    def test_cold_and_reused_session_results_are_identical(self):
        """A cold request and a warm one publish identical bytes and work."""
        first_wire, first_observation = self.exchange()
        second_wire, second_observation = self.exchange()
        self.assert_ok(first_wire)
        self.assert_same_response(first_wire, second_wire)
        first = first_observation["snapshot"]
        second = second_observation["snapshot"]
        for field in ("cumulative", "counters", "peak", "maximum", "phases"):
            self.assertEqual(first[field], second[field], field)

        # The shared-session projection matches an independent per-head
        # validation of the same authority snapshot, field for field and in the
        # same no-limit order.
        expected = []
        for path in sorted(self.dirs["branches"].glob("*.json")):
            record = self.namespace["validate_remote_branch_record"](
                json.loads(path.read_text(encoding="utf-8"))
            )
            self.namespace["validate_sealed_commit"](
                self.dirs["objects"], record["head"]
            )
            expected.append(
                {"version": 1, "name": record["name"], "head": record["head"]}
            )
        self.assertEqual(self.body_of(first_wire), {"branches": expected})

        # A failing store fails identically on a cold and a reused route.
        self.object_path(self.shared_tip).write_text("{broken", encoding="utf-8")
        cold_failure = self.exchange()
        warm_failure = self.exchange()
        self.assert_same_response(cold_failure[0], warm_failure[0])
        self.assertEqual(
            self.status_line(cold_failure[0]), b"HTTP/1.0 400 Bad Request"
        )
        for wire, observation in (cold_failure, warm_failure):
            self.assert_no_partial_response(wire, observation)

    # ------------------------------------------------------------------
    # the final snapshot/digest fence
    # ------------------------------------------------------------------

    def _mutate_before_final_fence(self, victim, mutate):
        """Run the live route, mutating ``victim`` just before the last fence."""
        globals_ = self.route_globals
        Session = globals_["SealedCommitValidationSession"]
        original_finalize = Session.finalize
        before = victim.stat()
        state = {"mutated": False, "after": None}

        def mutating_finalize(session, *args, **kwargs):
            if not state["mutated"]:
                state["mutated"] = True
                mutate(victim, before)
                state["after"] = victim.stat()
            return original_finalize(session, *args, **kwargs)

        with self.patched(Session, "finalize", mutating_finalize):
            wire, observation = self.exchange()
        self.assertTrue(state["mutated"], "the final fence never ran")
        return wire, observation, before, state["after"]

    def test_same_metadata_content_change_fails_closed_at_the_final_fence(self):
        """Same path, inode, and mtime never authorize reuse of stale content."""
        victim_id = min(
            record["object_id"]
            for record in self.all_records
            if record["type"] != "commit"
        )
        victim = self.object_path(victim_id)

        def same_length_swap(path, before):
            raw = bytearray(path.read_bytes())
            for index, byte in enumerate(raw):
                if byte in b"0123456789":
                    raw[index] = ord("0") if byte == ord("9") else byte + 1
                    break
            else:
                raise AssertionError("no digit byte available to swap")
            descriptor = os.open(path, os.O_WRONLY)
            try:
                os.pwrite(descriptor, bytes(raw), 0)
            finally:
                os.close(descriptor)
            os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))

        wire, observation, before, after = self._mutate_before_final_fence(
            victim, same_length_swap
        )
        # The mutation is invisible to path, inode, size, and mtime.
        self.assertEqual(after.st_ino, before.st_ino)
        self.assertEqual(after.st_size, before.st_size)
        self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertIn("sealed validation subject changed", message)
        self.assertIn("digest_changed", message)
        self.assertIn(victim_id, message)
        self.assertIn(str(victim), message)
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "subject_changed"
        )
        self.assert_no_partial_response(wire, observation)

    def test_extension_only_change_fails_closed_at_the_final_fence(self):
        """An extension-only append on the same inode is still rejected."""
        victim_id = min(
            record["object_id"]
            for record in self.all_records
            if record["type"] != "commit"
        )
        victim = self.object_path(victim_id)

        def append_extension(path, before):
            record = json.loads(path.read_bytes())
            record["extensions"] = [
                {
                    "namespace": "codefire.test",
                    "name": "branch-list-fence",
                    "version": 1,
                    "critical": False,
                }
            ]
            descriptor = os.open(path, os.O_WRONLY)
            try:
                os.pwrite(
                    descriptor,
                    json.dumps(record, ensure_ascii=False, sort_keys=True).encode(
                        "utf-8"
                    ),
                    0,
                )
            finally:
                os.close(descriptor)
            os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))

        wire, observation, before, after = self._mutate_before_final_fence(
            victim, append_extension
        )
        self.assertEqual(after.st_ino, before.st_ino)
        self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertIn("sealed validation subject changed", message)
        self.assertIn(victim_id, message)
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "subject_changed"
        )
        self.assert_no_partial_response(wire, observation)

    # ------------------------------------------------------------------
    # exact and exact-minus-one resource bounds
    # ------------------------------------------------------------------

    def test_exact_and_exact_minus_one_limits_bound_every_stage(self):
        """Each stage's measured cost passes at exactly its limit and no less."""
        baseline_wire, baseline = self.exchange()
        self.assert_ok(baseline_wire)
        snapshot = baseline["snapshot"]
        cumulative = snapshot["cumulative"]
        peak = snapshot["peak"]
        cases = [
            # authority record capture
            ("aggregate_bytes", cumulative["aggregate_bytes"], "record_read"),
            ("object_records", cumulative["object_records"], "record_read"),
            # shared object/commit work
            ("graph_nodes", cumulative["graph_nodes"], "semantic_validation"),
            ("graph_edges", cumulative["graph_edges"], "graph_discovery"),
            (
                "snapshot_revalidation_records",
                cumulative["snapshot_revalidation_records"],
                "snapshot_revalidation",
            ),
            # retained projection ownership
            ("retained_bytes", peak["retained_bytes"], None),
            ("temporary_bytes", peak["temporary_bytes"], None),
            # response encoding
            ("response_bytes", cumulative["response_bytes"], "response_body"),
        ]
        for dimension, exact, phase in cases:
            with self.subTest(dimension=dimension, bound="exact"):
                self.assertGreater(exact, 0)
                wire, observation = self.exchange(self.profile({dimension: exact}))
                self.assert_same_response(wire, baseline_wire)
                self.assertIsNone(observation["budget_diagnostic"])
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], "complete"
                )
                self.assert_no_ownership_debt(observation)
            with self.subTest(dimension=dimension, bound="exact-minus-one"):
                wire, observation = self.exchange(
                    self.profile({dimension: exact - 1})
                )
                self.assertEqual(
                    self.status_line(wire), b"HTTP/1.0 413 Request Entity Too Large"
                )
                diagnostic = observation["budget_diagnostic"]
                self.assertIsNotNone(diagnostic)
                self.assertEqual(diagnostic["kind"], "resource_budget_exceeded")
                self.assertEqual(diagnostic["dimension"], dimension)
                self.assertEqual(diagnostic["limit"], exact - 1)
                self.assertEqual(diagnostic["observed"], exact)
                self.assertFalse(diagnostic["continuation_supported"])
                if phase is not None:
                    self.assertEqual(diagnostic["phase"], phase)
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], "budget_exceeded"
                )
                self.assertIsNone(observation["deadline_diagnostic"])
                self.assert_no_partial_response(wire, observation)

    # ------------------------------------------------------------------
    # deterministic deadline expiry
    # ------------------------------------------------------------------

    def test_deadline_expires_deterministically_at_every_boundary(self):
        """Authority, validation, fence, and response phases all fail closed."""
        profile = self.profile(deadline_ms=10)
        phases, baseline = self.record_deadline_phases(profile)
        counts = Counter(phases)
        expected_phases = [
            "branch list authority",
            "branch list sealed validation",
            "branch list projection",
            "branch list snapshot revalidation",
            "branch list response encoding",
            "branch list response headers",
            "branch list response publication",
        ]
        for phase in ("branch list planning", "branch list project gate"):
            self.assertGreaterEqual(counts[phase], 1, phase)
        for phase in expected_phases:
            self.assertGreaterEqual(counts[phase], 1, phase)
        self.assertGreater(counts["branch list authority"], HEAD_COUNT)
        self.assertGreater(counts["branch list sealed validation"], HEAD_COUNT)
        self.assertGreater(counts["branch list snapshot revalidation"], 0)

        for phase in expected_phases:
            polls = [
                index
                for index, observed in enumerate(phases, start=1)
                if observed == phase
            ]
            expire_read = polls[len(polls) // 2]
            with self.subTest(phase=phase, expire_read=expire_read):
                wire, observation = self.exchange(
                    profile, clock=_ExpiringClock(expire_read)
                )
                self.assertEqual(
                    self.status_line(wire), b"HTTP/1.0 503 Service Unavailable"
                )
                payload = self.body_of(wire)
                diagnostic = payload["diagnostics"][0]
                self.assertEqual(diagnostic["kind"], "request_deadline_exceeded")
                self.assertEqual(diagnostic["phase"], phase)
                self.assertEqual(diagnostic["limit_ms"], 10)
                self.assertEqual(observation["deadline"]["expired_phase"], phase)
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], "deadline_exceeded"
                )
                self.assertIsNone(observation["budget_diagnostic"])
                self.assert_no_partial_response(wire, observation)

        # A deadline that has already expired stops the route before it reads
        # any branch authority record at all.
        immediate_wire, immediate = self.exchange(
            profile, clock=_ExpiringClock(1)
        )
        self.assertEqual(
            self.status_line(immediate_wire), b"HTTP/1.0 503 Service Unavailable"
        )
        self.assertEqual(
            immediate["snapshot"]["cumulative"].get("object_records", 0), 0
        )
        self.assertEqual(immediate["snapshot"]["cumulative"].get("graph_nodes", 0), 0)
        self.assert_no_partial_response(immediate_wire, immediate)
        self.assertEqual(baseline["snapshot"]["terminal_reason"], "complete")

    # ------------------------------------------------------------------
    # authority cardinality and failure precedence
    # ------------------------------------------------------------------

    def test_absent_empty_one_head_and_many_head_authority(self):
        """Cardinality boundaries all publish the compatible domain envelope."""
        absent_wire, absent = self.exchange(storage=self.tmp / "absent")
        self.assertEqual(self.status_line(absent_wire), b"HTTP/1.0 404 Not Found")
        self.assertEqual(self.body_of(absent_wire), {"error": "remote project not found"})
        self.assert_no_ownership_debt(absent)

        many_wire, many = self.exchange()
        many_body = self.assert_ok(many_wire)
        self.assertEqual(len(many_body["branches"]), HEAD_COUNT)
        self.assert_no_ownership_debt(many)

        self.only_branch(self.branch_names[0])
        one_wire, one = self.exchange()
        one_body = self.assert_ok(one_wire)
        self.assertEqual(
            one_body,
            {
                "branches": [
                    {
                        "version": 1,
                        "name": self.branch_names[0],
                        "head": self.heads[0],
                    }
                ]
            },
        )
        self.assert_no_ownership_debt(one)

        self.clear_branches()
        empty_wire, empty = self.exchange()
        self.assertEqual(self.assert_ok(empty_wire), {"branches": []})
        self.assertEqual(
            empty["snapshot"]["cumulative"].get("graph_nodes", 0), 0
        )
        self.assertEqual(empty["snapshot"]["terminal_reason"], "complete")
        self.assert_no_ownership_debt(empty)

    def test_malformed_authority_precedes_sealed_head_failure(self):
        """Authority capture rejects before any head validation is attempted."""
        # An earlier-sorting branch carries an absent sealed head while a
        # later-sorting branch is structurally malformed.  Authority capture is
        # a whole-snapshot phase, so the malformed record wins regardless of
        # sort position.
        absent_head = "CF-COMMIT-" + "0" * 24
        self.write_branch_record(
            self.branch_names[1],
            {"version": 1, "name": self.branch_names[1], "head": absent_head},
        )
        self.write_branch_record(
            self.branch_names[-1],
            {"version": 1, "name": self.branch_names[-1], "head": "not-a-commit"},
        )
        wire, observation = self.exchange()
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assertEqual(
            self.body_of(wire),
            {"error": "remote branch head is not a commit object ID"},
        )
        self.assertEqual(
            observation["snapshot"]["cumulative"].get("graph_nodes", 0),
            0,
            "no sealed validation may run once authority capture fails",
        )
        self.assert_no_partial_response(wire, observation)

    def test_unparseable_authority_record_fails_closed(self):
        """A corrupt authority record is a typed 400, not a partial listing."""
        path = self.namespace["remote_branch_path"](
            self.project_root, self.branch_names[3]
        )
        path.write_text("{not json", encoding="utf-8")
        wire, observation = self.exchange()
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assertIn("invalid JSON", self.body_of(wire)["error"])
        self.assert_no_partial_response(wire, observation)

    def test_invalid_sealed_head_fails_closed(self):
        """A well-formed record naming an absent commit fails during validation."""
        absent_head = "CF-COMMIT-" + "0" * 24
        self.write_branch_record(
            self.branch_names[2],
            {"version": 1, "name": self.branch_names[2], "head": absent_head},
        )
        wire, observation = self.exchange()
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assertEqual(
            self.body_of(wire), {"error": f"object not found: {absent_head}"}
        )
        # Authority capture completed for every record before validation ran.
        self.assertGreater(
            observation["snapshot"]["counters"]["files_read"], HEAD_COUNT
        )
        self.assert_no_partial_response(wire, observation)

    def test_shared_parent_failure_reports_once_for_every_head(self):
        """One corrupt shared ancestor fails the whole request exactly once."""
        healthy_wire, healthy = self.exchange()
        self.assert_ok(healthy_wire)
        healthy_reads = healthy["snapshot"]["counters"]["record_reads"]

        self.object_path(self.shared_tip).write_text("{broken", encoding="utf-8")
        wire, observation = self.exchange()
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertIn("invalid object JSON", message)
        self.assertIn(self.shared_tip, message)
        counters = observation["snapshot"]["counters"]
        # The shared ancestor is reached once, not once per head.
        self.assertLess(counters["record_reads"], healthy_reads)
        self.assertLessEqual(
            counters["record_reads"], len(self.all_records)
        )
        self.assert_no_partial_response(wire, observation)


    # ------------------------------------------------------------------
    # every failure path releases the whole request's ownership
    # ------------------------------------------------------------------

    def test_failure_paths_leave_no_open_session_permit_or_second_ledger(self):
        """Budget, deadline, and subject-change failures all release everything."""
        baseline_wire, baseline = self.exchange()
        self.assert_ok(baseline_wire)
        exact_nodes = baseline["snapshot"]["cumulative"]["graph_nodes"]

        deadline_profile = self.profile(deadline_ms=10)
        phases, _ = self.record_deadline_phases(deadline_profile)
        validation_polls = [
            index
            for index, phase in enumerate(phases, start=1)
            if phase == "branch list sealed validation"
        ]

        def budget_case():
            return self.exchange(self.profile({"graph_nodes": exact_nodes - 1}))

        def deadline_case():
            return self.exchange(
                deadline_profile,
                clock=_ExpiringClock(validation_polls[len(validation_polls) // 2]),
            )

        def subject_case():
            victim_id = min(
                record["object_id"]
                for record in self.all_records
                if record["type"] != "commit"
            )
            victim = self.object_path(victim_id)

            def swap(path, before):
                raw = bytearray(path.read_bytes())
                for index, byte in enumerate(raw):
                    if byte in b"0123456789":
                        raw[index] = ord("0") if byte == ord("9") else byte + 1
                        break
                descriptor = os.open(path, os.O_WRONLY)
                try:
                    os.pwrite(descriptor, bytes(raw), 0)
                finally:
                    os.close(descriptor)
                os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))

            wire, observation, _, _ = self._mutate_before_final_fence(victim, swap)
            return wire, observation

        cases = [
            ("budget_exceeded", budget_case),
            ("deadline_exceeded", deadline_case),
            ("subject_changed", subject_case),
        ]
        for reason, run_case in cases:
            with self.subTest(terminal_reason=reason):
                with self.tracked_ledgers() as ledgers:
                    with self.tracked_sessions() as sessions:
                        wire, observation = run_case()
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], reason
                )
                self.assert_no_partial_response(wire, observation)
                # One request ledger; no hidden second or nested local ledger.
                self.assertEqual(len(ledgers), 1)
                self.assert_sessions_closed(sessions)


if __name__ == "__main__":
    unittest.main()
