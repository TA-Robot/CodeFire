"""CFR-168 production-path coverage for ``GET /merge-requests``.

Every test drives the live ``make_http_handler`` route over a real socket so the
request-owned ``DecisionWorkLedger`` and ``HTTPRequestDeadline`` reaching
merge-request authority capture, URL/subject validation, sealed-reference
validation, the escrow object-set digest, projection, the final snapshot fence,
and response publication are the handler's own objects.  No wall-clock sleep is
used: the deadline is driven by an injected monotonic clock and every budget
bound is derived from a measured baseline observation.

Reversal scope of the regressions added while closing the CFR-168 review chain.
Most of them fail on the rejected predecessor ``38da898`` for their own finding.
Three deliberately do **not**, and that is recorded here rather than left as a
silent gap:

* ``test_every_production_poll_site_is_individually_required`` passes on
  ``38da898`` because the rejected finding was that the *oracle* was too coarse,
  not that any production poll was redundant.  ``38da898``'s polls were already
  individually necessary, so a strictly finer oracle correctly passes there.  Its
  non-vacuity is carried by the in-test helper mutation, which silences more than
  one site and so distinguishes this oracle from the rejected per-helper one.
* ``test_stable_pre_existing_links_stay_compatible_authority`` passes on
  ``38da898`` because the removed ``if capture.get("nlink") != 1`` check applied
  only to the project descriptor, for which the shared gate's
  ``_read_regular_bounded_at`` already enforces the legacy one-link rule.  The
  removal is observationally inert; the test still pins the contract and compares
  the live route against the legacy baseline byte for byte.
* ``test_stable_project_id_reads_the_descriptor_once_from_charged_bytes`` passes
  on ``38da898`` because it is a rename of the pre-existing
  ``test_project_descriptor_is_read_once_from_the_bytes_it_charges``, narrowed in
  title to the accounted boundary it actually measures.  The route-wide claim it
  used to overstate now lives in
  ``test_descriptor_read_protocol_is_exact_and_route_wide``, which does fail on
  ``38da898``.

The regressions added for the final-publication closure all fail on the rejected
candidate ``4930d25`` for their named reason, and also fail or error on
``38da898``: four of them exercise symbols that revision does not contain
(``_HTTPReadResponseSlot``, ``os_error_origin``, ``_release_exact``), while the
unproved-response and disposal-precedence cases fail there for the same class of
defect through the legacy no-slot path.
"""

import contextlib
import errno
import hashlib
import json
import os
import runpy
import shutil
import socket
import stat as stat_module
import subprocess
import sys
import tempfile
import threading
import unittest
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"

MR_LIST_REQUEST = b"GET /v1/projects/org/app/merge-requests HTTP/1.0\r\n\r\n"
BRANCH_LIST_REQUEST = b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"
DOCTOR_REQUEST = b"GET /v1/projects/org/app/doctor HTTP/1.0\r\n\r\n"
BUNDLE_REQUEST = b"GET /v1/projects/org/app/branches/main/bundle HTTP/1.0\r\n\r\n"
SHARED_DEPTH = 6
MR_COUNT = 8
STALE_INDEX = 3
APPROVED_INDEX = 5
ENDPOINT = "cf+http://127.0.0.1:9/org/app"
TARGET_BRANCH = "main"
FIXED_UPDATED_AT = "2026-07-30T00:00:00Z"
# Every merge-request record, the one consulted branch record, and the pinned
# project descriptor are captured and fenced exactly once per request.
CONSULTED_AUTHORITY_FILES = 2
# The cardinalities the memoized escrow work must stay constant across.
SHARED_HEAD_CARDINALITIES = (1, 8, 24, 80, 200)
# One typed diagnostic family answers every unusable filesystem authority.
AUTHORITY_FAILURE_PREFIX = "filesystem authority is unusable: "
AUTHORITY_CHANGED_PREFIX = "pinned authority record changed: "


def post_request(path, body=b"{}"):
    """Build one HTTP/1.0 POST request with an exact Content-Length."""
    return (
        f"POST {path} HTTP/1.0\r\nContent-Length: {len(body)}\r\n\r\n".encode("ascii")
        + body
    )


# Every production deadline-poll helper the site resolver must look through to
# reach the exact production call site that owns a poll.
POLL_HELPER_FRAMES = frozenset(
    {
        "poll",
        "_authority_poll",
        "_validation_poll",
        "_projection_poll",
        "_revalidation_poll",
        "poll_authority",
        "check",
        "<lambda>",
    }
)

# Every production function this lane owns a deadline-poll site in.  The site
# enumeration below is derived from live frames, never from the helper names, so
# it cannot be satisfied by replacing one phase helper.
MR_LIST_POLL_OWNERS = frozenset(
    {
        "handle_http_request_list",
        "collect_http_merge_request_list_payload",
        "capture_authority",
        "validate_records",
        "validate_record",
        "validate_subject",
        "project",
        "finalize_authority",
        "finalize_sessions",
        "escrow_object_set",
        "_collect_escrow_object_ids",
        "_digest_escrow_records",
        "_digest_subject",
        "stable_project_id",
        "consulted_branch",
        "read_json_accounted",
        "revalidate_json_authority_capture",
        "accounted_sorted_json_paths",
        "prepare_json_owned",
    }
)

# The documented maximum admitted-work quantum between two consecutive
# production polls of one MR-list request: ledger operations plus physical
# descriptor opens, reads, metadata calls, and directory scans.
MAXIMUM_ADMITTED_WORK_QUANTUM = 64


def resolve_poll_site(depth=2):
    """Name the exact production call site that owns the running poll."""
    frame = sys._getframe(depth)
    while frame is not None and frame.f_code.co_name in POLL_HELPER_FRAMES:
        frame = frame.f_back
    if frame is None:
        return ("<unknown>", 0)
    return (frame.f_code.co_name, frame.f_lineno)


class _FailingSocketFile:
    """Wrap one real socket file object and fail its next read or write."""

    def __init__(self, wrapped, error, *, on):
        self._wrapped = wrapped
        self._error = error
        self._on = on
        self.failures = 0

    def _maybe_fail(self, name):
        if name in self._on:
            self.failures += 1
            raise self._error

    def write(self, data):
        self._maybe_fail("write")
        return self._wrapped.write(data)

    def read(self, *args):
        self._maybe_fail("read")
        return self._wrapped.read(*args)

    def readline(self, *args):
        self._maybe_fail("readline")
        return self._wrapped.readline(*args)

    def flush(self):
        self._maybe_fail("flush")
        return self._wrapped.flush()

    def close(self):
        return self._wrapped.close()

    def __getattr__(self, name):
        return getattr(self._wrapped, name)


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


class _TriggeredClock(_Clock):
    """Expire only when an exact production boundary arms this clock.

    Silencing one poll cannot shift the expiry point, so the admitted work a
    silenced site would have cancelled is measured against the same instant.
    """

    def __init__(self):
        super().__init__()
        self.expired = False

    def __call__(self):
        self.reads += 1
        return 0.011 if self.expired else 0.0


class PythonHTTPMergeRequestListDecisionWorkTests(unittest.TestCase):
    """Exercise the live merge-request-list route's request-owned work."""

    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.template = Path(tempfile.mkdtemp(prefix="codefire-mr-template-"))
        try:
            cls._build_template()
        except BaseException:
            shutil.rmtree(cls.template, ignore_errors=True)
            raise

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.template, ignore_errors=True)

    @classmethod
    def _seed_local_history(cls, local):
        """Build one real two-commit history whose manifest references blobs.

        The escrow object set reached through a merge request's subject includes
        content blobs that sealed commit validation never loads, so the fixture
        needs a real committed file to cover the whole traversal.
        """

        def run(*args, cwd):
            subprocess.run(
                [str(CODEFIRE), *args],
                cwd=cwd,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

        run("init", cwd=local)
        worktree = local / "work"
        run("open", "main", str(worktree), cwd=local)
        (worktree / "src").mkdir(parents=True)
        (worktree / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\nEXPIRES_AFTER_MINUTES = 30\n",
            encoding="utf-8",
        )
        (worktree / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n",
            encoding="utf-8",
        )
        scan = subprocess.run(
            [str(CODEFIRE), "scan"],
            cwd=worktree,
            check=True,
            capture_output=True,
            text=True,
        )
        for line in scan.stdout.splitlines():
            stripped = line.strip()
            if stripped.startswith("FIRE-"):
                run(
                    "extinguish",
                    stripped.split()[0],
                    "--resolution",
                    "changed",
                    "--evidence",
                    "fixture seed",
                    cwd=worktree,
                )
        run("verify", cwd=worktree)
        run("commit", "-m", "fixture seed commit", cwd=worktree)
        shutil.rmtree(worktree)

    @classmethod
    def _build_template(cls):
        """Seed one project whose merge requests share a deep commit ancestry."""
        namespace = cls.namespace
        local = cls.template / "local"
        local.mkdir()
        cls._seed_local_history(local)
        local_objects = local / ".codefire/objects"
        root_head = json.loads(
            (local / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]
        base_records = namespace["collect_object_records"](local_objects, root_head)
        root_commit = next(
            record for record in base_records if record["object_id"] == root_head
        )

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
        cls.target_head = tip
        cls.stale_target_head = spine[2]["object_id"]
        source_heads = []
        for index in range(MR_COUNT):
            commit = child_commit(tip, f"source head {index:02d}")
            spine.append(commit)
            source_heads.append(commit["object_id"])

        cls.all_records = base_records + spine
        cls.source_heads = source_heads
        cls.root_head = root_head
        cls.source_branches = [f"s{index:02d}" for index in range(MR_COUNT)]
        cls.commit_count = sum(
            1 for record in cls.all_records if record["type"] == "commit"
        )
        cls.blob_count = sum(
            1 for record in cls.all_records if record["type"] == "blob"
        )
        if cls.blob_count < 1:
            raise AssertionError("fixture reached no escrow content blob")

        storage = cls.template / "storage"
        status, payload = namespace["handle_http_upload"](
            storage,
            "org",
            "app",
            TARGET_BRANCH,
            {
                "branch": TARGET_BRANCH,
                "head": cls.target_head,
                "objects": cls.all_records,
            },
        )
        if status != 200:
            raise AssertionError(f"template upload failed: {status} {payload}")
        project_root = namespace["http_project_root"](storage, "org", "app")
        dirs = namespace["remote_dirs"](project_root)
        for name, head in zip(cls.source_branches, source_heads):
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
        cls.merge_request_ids = []
        for index, (name, head) in enumerate(zip(cls.source_branches, source_heads)):
            target_head_at_request = (
                cls.stale_target_head if index == STALE_INDEX else cls.target_head
            )
            record = {
                "version": 1,
                "source_url": f"{ENDPOINT}/{name}",
                "source_head": head,
                "target_url": f"{ENDPOINT}/{TARGET_BRANCH}",
                "target_head_at_request": target_head_at_request,
                "status": "approved" if index == APPROVED_INDEX else "open",
                "created_by": "anonymous",
                "created_at": FIXED_UPDATED_AT,
            }
            mr_id = (
                "MR-"
                + hashlib.sha256(namespace["canonical_json"](record)).hexdigest()[:12]
            )
            record["id"] = mr_id
            subject = namespace["build_merge_request_subject"](
                mr_id,
                {"branch": name, "project_root": project_root},
                {"branch": TARGET_BRANCH, "project_root": project_root},
                head,
                target_head_at_request,
                namespace["merge_request_object_set_digest"](
                    namespace["collect_object_records"](dirs["objects"], head)
                ),
            )
            record["subject"] = subject
            record["subject_digest"] = namespace["merge_request_subject_digest"](
                subject
            )
            namespace["write_json"](
                namespace["remote_merge_request_path"](project_root, mr_id), record
            )
            cls.merge_request_ids.append(mr_id)
        shutil.rmtree(local)

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="codefire-mr-route-"))
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
        request=MR_LIST_REQUEST,
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
                try:
                    client.shutdown(socket.SHUT_WR)
                except OSError:
                    # A deliberately abandoned transport may already have closed
                    # the peer, so this half-close has nothing left to shut down.
                    # Every wire assertion below still sees the exact bytes.
                    pass
                chunks = []
                while True:
                    try:
                        chunk = client.recv(64 * 1024)
                    except ConnectionResetError:
                        break
                    if not chunk:
                        break
                    chunks.append(chunk)
        finally:
            server.server_close()
            thread.join(timeout=30)
        self.assertFalse(thread.is_alive(), "MR-list handler did not terminate")
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
        return self.namespace[
            "collect_http_merge_request_list_payload"
        ].__globals__

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

    @contextlib.contextmanager
    def forbidden_legacy_helpers(self):
        """Fail if the live route reaches any unaccounted legacy helper."""
        globals_ = self.route_globals
        names = (
            "validate_sealed_commit",
            "validate_http_merge_request_record",
            "validate_merge_request_subject",
            "read_remote_merge_request_path",
            "read_remote_branch_record",
            "collect_object_records",
            "merge_request_object_set_digest",
            "merge_request_subject_digest",
            "read_json",
            # A reopened project descriptor is the COULD-1 regression: the
            # pinned descriptor must be parsed from the bytes already owned.
            "remote_project_stable_id",
        )
        with contextlib.ExitStack() as stack:
            for name in names:
                def forbidden(*args, _name=name, **kwargs):
                    raise AssertionError(
                        f"live MR-list route reached the legacy helper {_name}"
                    )

                stack.enter_context(self.patched(globals_, name, forbidden))
            yield

    def assert_sessions_closed(self, sessions, *, expected=1):
        """No sealed-validation session may survive the request."""
        self.assertEqual(
            len(sessions),
            expected,
            "route opened an unexpected number of sealed-validation sessions",
        )
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
            line for line in head.split(b"\r\n") if not line.startswith(b"Date: ")
        ]
        return tuple(lines), body

    def assert_same_response(self, first, second):
        """Two responses agree on everything but the volatile Date header."""
        self.assertEqual(self.stable_response(first), self.stable_response(second))

    def assert_ok(self, wire):
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 200 OK")
        return self.body_of(wire)

    def assert_no_ownership_debt(self, observation):
        """No retained/temporary debt and no open permit survives a request."""
        snapshot = observation["snapshot"]
        self.assertTrue(
            all(value == 0 for value in snapshot["current"].values()),
            f"concurrent ownership debt remained: {snapshot['current']}",
        )
        self.assertEqual(observation["open_permit_count"], 0)

    def assert_no_partial_response(self, wire, observation):
        """A failed request publishes a diagnostic envelope, never a projection."""
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        self.assertNotIn(b'"merge_requests"', wire)
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

    # ------------------------------------------------------------------
    # authority fixture helpers
    # ------------------------------------------------------------------

    def merge_request_paths(self):
        return sorted(self.dirs["merge_requests"].glob("*.json"))

    def merge_request_path(self, mr_id):
        return self.namespace["remote_merge_request_path"](self.project_root, mr_id)

    def read_merge_request(self, mr_id):
        return json.loads(
            self.merge_request_path(mr_id).read_text(encoding="utf-8")
        )

    def write_merge_request(self, mr_id, record):
        self.merge_request_path(mr_id).write_text(
            json.dumps(record, ensure_ascii=False, sort_keys=True), encoding="utf-8"
        )

    def only_merge_request(self, mr_id):
        keep = self.merge_request_path(mr_id)
        for path in self.merge_request_paths():
            if path != keep:
                path.unlink()

    def clear_merge_requests(self):
        for path in self.merge_request_paths():
            path.unlink()

    def object_path(self, object_id):
        return self.namespace["object_record_path"](self.dirs["objects"], object_id)

    def branch_path(self, name):
        return self.namespace["remote_branch_path"](self.project_root, name)

    def descriptor_path(self):
        return self.project_root / "remote_project.json"

    def write_shared_head_merge_requests(self, count):
        """Replace the authority with ``count`` requests sharing one head.

        Every request names the same pinned source head, source branch, and
        target branch, so the escrow object set reached through each subject is
        byte-identical.  Only the merge-request id, its subject binding, and the
        record filename differ, which is exactly the per-record work that may
        still scale with cardinality.
        """
        namespace = self.namespace
        self.clear_merge_requests()
        branch = self.source_branches[0]
        head = self.source_heads[0]
        escrow_digest = namespace["merge_request_object_set_digest"](
            namespace["collect_object_records"](self.dirs["objects"], head)
        )
        identifiers = []
        for index in range(count):
            mr_id = "MR-%012x" % index
            subject = namespace["build_merge_request_subject"](
                mr_id,
                {"branch": branch, "project_root": self.project_root},
                {"branch": TARGET_BRANCH, "project_root": self.project_root},
                head,
                self.target_head,
                escrow_digest,
            )
            record = {
                "version": 1,
                "id": mr_id,
                "source_url": f"{ENDPOINT}/{branch}",
                "source_head": head,
                "target_url": f"{ENDPOINT}/{TARGET_BRANCH}",
                "target_head_at_request": self.target_head,
                "status": "open",
                "created_by": "anonymous",
                "created_at": FIXED_UPDATED_AT,
                "subject": subject,
                "subject_digest": namespace["merge_request_subject_digest"](
                    subject
                ),
            }
            namespace["write_json"](self.merge_request_path(mr_id), record)
            identifiers.append(mr_id)
        return identifiers

    @contextlib.contextmanager
    def recorded_accounted_reads(self):
        """Record every path the live route reads through the accounted reader."""
        globals_ = self.route_globals
        original = globals_["read_json_accounted"]
        reads = []

        def recording(path, ledger, **kwargs):
            reads.append(str(path))
            return original(path, ledger, **kwargs)

        with self.patched(globals_, "read_json_accounted", recording):
            yield reads

    def captured_handler_logs(self, records, *, configure=None):
        """Configure one handler whose error log is captured instead of printed."""

        def configure_handler(handler):
            if configure is not None:
                configure(handler)

            def log_error(owner, template, *args):
                records.append(template % args if args else template)

            handler.log_error = log_error

        return configure_handler

    @contextlib.contextmanager
    def abandoned_connection_expected(self):
        """Silence the server traceback a deliberately abandoned request logs."""
        server = self.namespace["ReusableThreadingHTTPServer"]
        with self.patched(server, "handle_error", lambda *args: None):
            yield

    @contextlib.contextmanager
    def eager_authority_faults(self):
        """Restore the reviewed regression: raise a capture fault immediately."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]

        def eager_defer(owner, entry, exc):
            raise exc

        with self.patched(Operation, "_defer", eager_defer):
            yield

    @contextlib.contextmanager
    def unmemoized_escrow(self):
        """Restore the reviewed regression: re-traverse and re-digest per MR."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        original = Operation.escrow_object_set

        def uncached(owner, session, source_head):
            owner._escrow_sets.clear()
            return original(owner, session, source_head)

        with self.patched(Operation, "escrow_object_set", uncached):
            yield

    def blob_object_id(self):
        """One non-commit record reachable only through the escrow traversal."""
        return min(
            record["object_id"]
            for record in self.all_records
            if record["type"] == "blob"
        )

    # ------------------------------------------------------------------
    # the reproduced pre-fix route
    # ------------------------------------------------------------------

    def legacy_request_list(self, storage_root, *, hold_gate=True):
        """Reproduce the pre-fix unaccounted/legacy ``handle_http_request_list``.

        Names resolve through the production module globals so a patched legacy
        helper is observed exactly as the pre-fix route observed it.
        """
        namespace = self.route_globals
        project_root = namespace["http_project_root"](storage_root, "org", "app")
        dirs = namespace["remote_dirs"](project_root)
        if not dirs["merge_requests"].exists():
            return namespace["http_error"](404, "remote project not found")
        gate = (
            namespace["remote_project_gate"](project_root, shared=True)
            if hold_gate
            else contextlib.nullcontext()
        )
        with gate:
            requests = []
            for path in sorted(dirs["merge_requests"].glob("*.json")):
                mr = namespace["read_remote_merge_request_path"](path)
                source_info = namespace["parse_cf_http_url"](mr["source_url"])
                source_root = namespace["http_project_root"](
                    storage_root, source_info["org"], source_info["app"]
                )
                target_info = namespace["parse_cf_http_url"](mr["target_url"])
                target_root = namespace["http_project_root"](
                    storage_root, target_info["org"], target_info["app"]
                )
                if target_root != project_root:
                    raise namespace["CFError"](
                        "merge request target does not belong to the listed project"
                    )
                namespace["reject_legacy_cross_project_merge_request_read"](
                    mr, source_root, target_root
                )
                namespace["validate_http_merge_request_record"](storage_root, mr)
                current_target = namespace["read_remote_branch_record"](
                    target_root, target_info["branch"]
                )
                status = mr.get("status", "open")
                if (
                    current_target
                    and current_target["head"] != mr["target_head_at_request"]
                ):
                    status = "stale"
                requests.append(
                    {
                        "id": mr["id"],
                        "status": status,
                        "source_url": mr["source_url"],
                        "target_url": mr["target_url"],
                    }
                )
        return 200, {"merge_requests": requests}

    # ------------------------------------------------------------------
    # request-owned ledger and deadline identity
    # ------------------------------------------------------------------

    def test_route_threads_the_exact_request_ledger_and_deadline(self):
        """Every MR-list stage runs on the handler's own ledger and deadline."""
        globals_ = self.route_globals
        seen = {}
        sessions = []
        projections = []
        operations = []
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
            sessions.append((owner, ledger))
            return original_session_init(owner, objects_root, ledger, **kwargs)

        Projection = globals_["AccountedHTTPMergeRequestList"]
        original_projection_init = Projection.__init__

        def recording_projection_init(owner, ledger, requests, retained_bytes):
            projections.append((owner, ledger))
            return original_projection_init(owner, ledger, requests, retained_bytes)

        Operation = globals_["_HTTPMergeRequestListOperation"]
        original_operation_init = Operation.__init__

        def recording_operation_init(owner, *args, **kwargs):
            operations.append(owner)
            return original_operation_init(owner, *args, **kwargs)

        original_list = globals_["handle_http_request_list"]

        def recording_list(storage_root, org, app, ledger, deadline=None, **kwargs):
            list_args.append((ledger, deadline, kwargs.get("response_slot")))
            return original_list(
                storage_root, org, app, ledger, deadline, **kwargs
            )

        original_collect = globals_["collect_http_merge_request_list_payload"]

        def recording_collect(
            storage_root, project_root, dirs, ledger, deadline=None, **kwargs
        ):
            collect_args.append(
                (ledger, deadline, kwargs.get("response_slot"), kwargs.get("gate"))
            )
            return original_collect(
                storage_root, project_root, dirs, ledger, deadline, **kwargs
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
            stack.enter_context(
                self.patched(Operation, "__init__", recording_operation_init)
            )
            stack.enter_context(
                self.patched(globals_, "handle_http_request_list", recording_list)
            )
            stack.enter_context(
                self.patched(
                    globals_,
                    "collect_http_merge_request_list_payload",
                    recording_collect,
                )
            )
            stack.enter_context(self.forbidden_legacy_helpers())
            wire, observation = self.exchange(
                profile, clock=_Clock(), configure_handler=configure
            )

        body = self.assert_ok(wire)
        self.assertEqual(len(body["merge_requests"]), MR_COUNT)

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
        # One prepared-response owner and one gate fence reach the collector, so
        # the response is encoded and fenced inside the handler's own request.
        read_slot = list_args[0][2]
        self.assertIsNotNone(read_slot)
        self.assertIs(collect_args[0][2], read_slot)
        self.assertIsNotNone(collect_args[0][3])
        self.assertEqual(read_slot.prepare_count, 1)
        self.assertTrue(read_slot.state_proved)
        self.assertFalse(read_slot.is_prepared)
        self.assertEqual(collect_args[0][3].fence_count, 1)

        # One MR-list operation owner, one session, one projection, all bound to
        # the same request ledger and deadline.
        self.assertEqual(len(operations), 1)
        self.assertIs(operations[0].ledger, ledger)
        self.assertIs(operations[0].deadline, deadline)
        self.assertEqual(len(sessions), 1)
        self.assertIs(sessions[0][1], ledger)
        self.assertIs(sessions[0][0].ledger, ledger)
        self.assertTrue(sessions[0][0].closed)
        self.assertTrue(sessions[0][0].finalized)
        self.assertEqual(len(projections), 1)
        self.assertIs(projections[0][1], ledger)

        self.assertEqual(deadline.limit_ms, 30_000)
        self.assertGreater(deadline.poll_count, 0)
        self.assertEqual(observation["deadline"]["poll_count"], deadline.poll_count)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_ownership_debt(observation)

    def test_pre_fix_target_reaches_the_unaccounted_legacy_path(self):
        """Restoring the pre-fix body restores the unaccounted/legacy defect."""
        globals_ = self.route_globals
        observed = {}
        reached = Counter()

        legacy_names = (
            "read_remote_merge_request_path",
            "validate_sealed_commit",
            "collect_object_records",
            "read_remote_branch_record",
        )

        def replay(
            storage_root, project_root, dirs, ledger, deadline=None, **kwargs
        ):
            # The pre-fix body never prepared a response before its fence, so it
            # ignores the prepared-response slot and returns a payload.
            before = (
                dict(ledger.cumulative),
                dict(ledger.counters),
                deadline.poll_count,
            )
            with contextlib.ExitStack() as stack:
                for name in legacy_names:
                    original = globals_[name]

                    def counting(*args, _name=name, _original=original, **kwargs):
                        reached[_name] += 1
                        return _original(*args, **kwargs)

                    stack.enter_context(self.patched(globals_, name, counting))
                _, payload = self.legacy_request_list(
                    storage_root, hold_gate=False
                )
            observed["before"] = before
            observed["after"] = (
                dict(ledger.cumulative),
                dict(ledger.counters),
                deadline.poll_count,
            )
            observed["payload"] = payload
            return payload

        profile = self.profile(deadline_ms=30_000)
        with self.patched(
            globals_, "collect_http_merge_request_list_payload", replay
        ):
            legacy_wire, legacy_observation = self.exchange(
                profile, clock=_Clock()
            )
        legacy_body = self.assert_ok(legacy_wire)

        # The reproduced pre-fix body reaches every unaccounted/legacy helper.
        self.assertEqual(reached["read_remote_merge_request_path"], MR_COUNT)
        self.assertEqual(reached["read_remote_branch_record"], MR_COUNT)
        self.assertGreaterEqual(
            reached["validate_sealed_commit"], 2 * MR_COUNT
        )
        self.assertEqual(reached["collect_object_records"], MR_COUNT)

        # And it charges the request ledger nothing and never polls the deadline
        # while it reads, validates, and projects the whole authority snapshot.
        self.assertEqual(observed["before"], observed["after"])

        # The accounted route publishes the same bytes while charging real work
        # and polling the same request deadline.
        fixed_wire, fixed_observation = self.exchange(profile, clock=_Clock())
        self.assert_ok(fixed_wire)
        self.assertEqual(self.body_of(fixed_wire), legacy_body)
        legacy_snapshot = legacy_observation["snapshot"]
        fixed_snapshot = fixed_observation["snapshot"]
        for dimension in ("object_records", "graph_nodes", "aggregate_bytes"):
            self.assertEqual(legacy_snapshot["cumulative"].get(dimension, 0), 0)
            self.assertGreater(fixed_snapshot["cumulative"][dimension], 0)
        self.assertEqual(
            legacy_snapshot["cumulative"].get("snapshot_revalidation_records", 0), 0
        )
        self.assertGreater(
            fixed_snapshot["cumulative"]["snapshot_revalidation_records"], 0
        )
        self.assertGreater(
            fixed_observation["deadline"]["poll_count"],
            legacy_observation["deadline"]["poll_count"],
        )
        self.assert_no_ownership_debt(fixed_observation)

    # ------------------------------------------------------------------
    # one shared session per distinct pinned object root
    # ------------------------------------------------------------------

    def test_shared_dag_validates_once_per_object_root_session(self):
        """Every MR reference and escrow record is read once per request."""
        with self.tracked_sessions() as sessions:
            wire, observation = self.exchange()
        body = self.assert_ok(wire)

        # All source and target references resolve to one pinned object root,
        # so the whole request shares exactly one sealed-validation session.
        self.assert_sessions_closed(sessions, expected=1)
        self.assertTrue(sessions[0].finalized)

        snapshot = observation["snapshot"]
        cumulative = snapshot["cumulative"]
        counters = snapshot["counters"]
        unique_objects = len(self.all_records)

        # The shared immutable ancestry and escrow set are read, validated, and
        # revalidated once per object, not once per merge request.
        self.assertEqual(cumulative["object_records"], unique_objects)
        self.assertEqual(counters["memo_misses"], unique_objects)
        self.assertEqual(cumulative["graph_nodes"], self.commit_count)
        self.assertEqual(
            counters["semantic_validations"], unique_objects + self.commit_count
        )
        # Every object plus every consumed authority file is fenced once.
        self.assertEqual(
            cumulative["snapshot_revalidation_records"],
            unique_objects + MR_COUNT + CONSULTED_AUTHORITY_FILES,
        )
        self.assertGreater(counters["memo_hits"], MR_COUNT)
        self.assertEqual(len(body["merge_requests"]), MR_COUNT)

        # Measure the cold, one-session-per-reference cost the defect produced.
        cold = self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(),
            "cold per-reference probe",
            command_kind="http_request",
        )
        for index, head in enumerate(self.source_heads):
            target = (
                self.stale_target_head
                if index == STALE_INDEX
                else self.target_head
            )
            for reference in (head, target):
                session = self.namespace["SealedCommitValidationSession"](
                    self.dirs["objects"], cold
                )
                try:
                    session.validate_commit(reference)
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
        # A per-reference traversal re-reads the shared spine for every MR.
        self.assertGreaterEqual(
            cold_cumulative["object_records"], MR_COUNT * (SHARED_DEPTH + 1)
        )
        self.assert_no_ownership_debt(observation)

    def test_target_branch_authority_is_captured_once_per_subject(self):
        """One consulted branch head is read once, not once per merge request."""
        globals_ = self.route_globals
        original = globals_["read_json_accounted"]
        branch_reads = []
        branch_root = self.dirs["branches"]

        def recording(path, ledger, **kwargs):
            if Path(path).parent == branch_root:
                branch_reads.append(Path(path).name)
            return original(path, ledger, **kwargs)

        with self.patched(globals_, "read_json_accounted", recording):
            wire, observation = self.exchange()
        self.assert_ok(wire)
        self.assertEqual(
            branch_reads,
            [self.branch_path(TARGET_BRANCH).name],
            "the consulted branch head was not captured exactly once",
        )
        self.assert_no_ownership_debt(observation)

    def test_cold_and_reused_route_publish_identical_bytes_and_work(self):
        """A cold request and a warm one publish identical bytes and work."""
        first_wire, first_observation = self.exchange()
        second_wire, second_observation = self.exchange()
        self.assert_ok(first_wire)
        self.assert_same_response(first_wire, second_wire)
        first = first_observation["snapshot"]
        second = second_observation["snapshot"]
        for field in ("cumulative", "counters", "peak", "maximum", "phases"):
            self.assertEqual(first[field], second[field], field)

        # The session-backed projection matches the legacy independent
        # validation of the same authority snapshot, field for field.
        status, expected = self.legacy_request_list(self.storage)
        self.assertEqual(status, 200)
        self.assertEqual(self.body_of(first_wire), expected)

        # A failing store fails identically on a cold and a reused route.
        self.object_path(self.target_head).write_text("{broken", encoding="utf-8")
        cold_failure = self.exchange()
        warm_failure = self.exchange()
        self.assert_same_response(cold_failure[0], warm_failure[0])
        self.assertEqual(
            self.status_line(cold_failure[0]), b"HTTP/1.0 400 Bad Request"
        )
        for wire, observation in (cold_failure, warm_failure):
            self.assert_no_partial_response(wire, observation)

    # ------------------------------------------------------------------
    # authority cardinality, status parity, and compatibility
    # ------------------------------------------------------------------

    def test_absent_empty_one_and_many_merge_request_authority(self):
        """Cardinality boundaries all publish the compatible domain envelope."""
        absent_wire, absent = self.exchange(storage=self.tmp / "absent")
        self.assertEqual(self.status_line(absent_wire), b"HTTP/1.0 404 Not Found")
        self.assertEqual(
            self.body_of(absent_wire), {"error": "remote project not found"}
        )
        self.assert_no_ownership_debt(absent)

        many_wire, many = self.exchange()
        many_body = self.assert_ok(many_wire)
        self.assertEqual(len(many_body["merge_requests"]), MR_COUNT)
        self.assertEqual(
            [entry["id"] for entry in many_body["merge_requests"]],
            sorted(self.merge_request_ids),
            "the deterministic MR filename order was not preserved",
        )
        self.assert_no_ownership_debt(many)

        kept = sorted(self.merge_request_ids)[0]
        expected_record = self.read_merge_request(kept)
        # Merge-request IDs are content digests over real commit objects, so
        # which record sorts first varies between fixture builds.  The retained
        # record may therefore be the stale one, and the expected status must be
        # derived from the same legacy rule the route applies rather than from the
        # persisted field alone.
        consulted_head = json.loads(
            self.branch_path(TARGET_BRANCH).read_text(encoding="utf-8")
        )["head"]
        expected_status = (
            "stale"
            if consulted_head != expected_record["target_head_at_request"]
            else expected_record["status"]
        )
        self.only_merge_request(kept)
        one_wire, one = self.exchange()
        self.assertEqual(
            self.assert_ok(one_wire),
            {
                "merge_requests": [
                    {
                        "id": kept,
                        "status": expected_status,
                        "source_url": expected_record["source_url"],
                        "target_url": expected_record["target_url"],
                    }
                ]
            },
        )
        self.assert_no_ownership_debt(one)

        # The single-record cardinality is also exercised deterministically on the
        # stale record, which the digest-ordered selection above only reaches by
        # chance.  A one-record snapshot must still consult the branch head.
        self.setUp()
        stale_id = self.merge_request_ids[STALE_INDEX]
        stale_record = self.read_merge_request(stale_id)
        self.only_merge_request(stale_id)
        stale_wire, stale = self.exchange()
        self.assertEqual(
            self.assert_ok(stale_wire),
            {
                "merge_requests": [
                    {
                        "id": stale_id,
                        "status": "stale",
                        "source_url": stale_record["source_url"],
                        "target_url": stale_record["target_url"],
                    }
                ]
            },
        )
        self.assert_no_ownership_debt(stale)

        self.clear_merge_requests()
        empty_wire, empty = self.exchange()
        self.assertEqual(self.assert_ok(empty_wire), {"merge_requests": []})
        snapshot = empty["snapshot"]
        self.assertEqual(snapshot["cumulative"].get("graph_nodes", 0), 0)
        self.assertEqual(snapshot["cumulative"].get("object_records", 0), 0)
        self.assertEqual(snapshot["terminal_reason"], "complete")
        self.assert_no_ownership_debt(empty)

    def test_open_and_stale_status_parity(self):
        """Open, stale, and persisted statuses match the legacy computation."""
        wire, observation = self.exchange()
        body = self.assert_ok(wire)
        by_id = {entry["id"]: entry for entry in body["merge_requests"]}
        stale_id = self.merge_request_ids[STALE_INDEX]
        approved_id = self.merge_request_ids[APPROVED_INDEX]
        self.assertEqual(by_id[stale_id]["status"], "stale")
        self.assertEqual(by_id[approved_id]["status"], "approved")
        open_ids = [
            mr_id
            for index, mr_id in enumerate(self.merge_request_ids)
            if index not in {STALE_INDEX, APPROVED_INDEX}
        ]
        self.assertTrue(open_ids)
        for mr_id in open_ids:
            self.assertEqual(by_id[mr_id]["status"], "open")
        self.assert_no_ownership_debt(observation)

        # Advancing the consulted branch head turns every matching request
        # stale, exactly as the legacy per-record read did.
        record = json.loads(
            self.branch_path(TARGET_BRANCH).read_text(encoding="utf-8")
        )
        record["head"] = self.source_heads[0]
        self.branch_path(TARGET_BRANCH).write_text(
            json.dumps(record, ensure_ascii=False, sort_keys=True), encoding="utf-8"
        )
        status, expected = self.legacy_request_list(self.storage)
        self.assertEqual(status, 200)
        advanced_wire, advanced = self.exchange()
        self.assertEqual(self.assert_ok(advanced_wire), expected)
        self.assertEqual(
            {entry["status"] for entry in expected["merge_requests"]},
            {"stale"},
        )
        self.assert_no_ownership_debt(advanced)

        # A removed consulted branch head keeps the persisted status.
        self.branch_path(TARGET_BRANCH).unlink()
        missing_wire, missing = self.exchange()
        missing_body = self.assert_ok(missing_wire)
        self.assertEqual(
            {entry["id"]: entry["status"] for entry in missing_body["merge_requests"]},
            {
                mr_id: ("approved" if index == APPROVED_INDEX else "open")
                for index, mr_id in enumerate(self.merge_request_ids)
            },
        )
        self.assert_no_ownership_debt(missing)

    def test_legacy_cross_project_and_error_precedence_parity(self):
        """Cross-project and legacy rejections keep their exact diagnostics."""
        namespace = self.namespace
        cases = []

        # A target that does not belong to the listed project.
        foreign = self.read_merge_request(self.merge_request_ids[0])
        foreign["target_url"] = "cf+http://127.0.0.1:9/other/app/main"
        cases.append(
            (
                "foreign target",
                foreign,
                "merge request target does not belong to the listed project",
            )
        )

        # A legacy cross-project request without a materialized subject.
        legacy = self.read_merge_request(self.merge_request_ids[1])
        legacy.pop("subject")
        legacy.pop("subject_digest")
        legacy["source_url"] = "cf+http://127.0.0.1:9/other/app/feature"
        cases.append(
            (
                "legacy cross-project",
                legacy,
                "legacy cross-project merge request cannot be read without holding "
                "both project gates; run request-review to materialize its "
                "target-local subject",
            )
        )

        # Mismatched remote endpoints.
        endpoints = self.read_merge_request(self.merge_request_ids[2])
        endpoints["source_url"] = "cf+https://127.0.0.1:9/org/app/s02"
        cases.append(
            (
                "mismatched endpoints",
                endpoints,
                "HTTP merge requests across different remote endpoints are not "
                "supported",
            )
        )

        # A future persisted record version.
        future = self.read_merge_request(self.merge_request_ids[4])
        future["version"] = 2
        cases.append(
            (
                "future version",
                future,
                "unsupported remote_merge_request.version 2; readable version is 1",
            )
        )

        # A diverged subject digest.
        diverged = self.read_merge_request(self.merge_request_ids[6])
        diverged["subject_digest"] = "sha256:" + "0" * 64
        cases.append(
            ("diverged subject digest", diverged, "merge request subject digest diverged")
        )

        # A subject whose escrow object-set digest no longer matches.
        drifted = self.read_merge_request(self.merge_request_ids[7])
        drifted["subject"] = dict(drifted["subject"])
        drifted["subject"]["source_object_set_digest"] = "sha256:" + "0" * 64
        drifted["subject_digest"] = namespace["merge_request_subject_digest"](
            drifted["subject"]
        )
        cases.append(
            (
                "drifted escrow digest",
                drifted,
                "merge request subject does not match the current request",
            )
        )

        for label, record, message in cases:
            with self.subTest(case=label):
                self.setUp()
                try:
                    self.write_merge_request(record["id"], record)
                    legacy_status, legacy_payload = None, None
                    try:
                        legacy_status, legacy_payload = self.legacy_request_list(
                            self.storage
                        )
                    except namespace["CFError"] as exc:
                        legacy_payload = {"error": str(exc)}
                        legacy_status = 400
                    wire, observation = self.exchange()
                    self.assertEqual(
                        self.status_line(wire), b"HTTP/1.0 400 Bad Request"
                    )
                    self.assertEqual(self.body_of(wire), {"error": message})
                    self.assertEqual(legacy_status, 400)
                    self.assertEqual(legacy_payload, {"error": message})
                    self.assert_no_partial_response(wire, observation)
                finally:
                    self.tearDown()

    def test_malformed_and_invalid_sealed_references_fail_closed(self):
        """Unparseable, malformed, and absent references never partially publish."""
        absent = "CF-COMMIT-" + "0" * 24
        cases = [
            (
                "unparseable record",
                lambda: self.merge_request_path(
                    self.merge_request_ids[0]
                ).write_text("{not json", encoding="utf-8"),
                lambda message: self.assertIn("invalid JSON", message),
            ),
            (
                "non-string source head",
                lambda: self._mutate_merge_request(
                    self.merge_request_ids[1], {"source_head": 7}
                ),
                lambda message: self.assertEqual(
                    message,
                    "merge request validation failed: source_head must be a "
                    "commit object id",
                ),
            ),
            (
                "absent target head",
                lambda: self._mutate_merge_request(
                    self.merge_request_ids[2], {"target_head_at_request": absent}
                ),
                lambda message: self.assertEqual(
                    message,
                    "merge request validation failed: target_head_at_request: "
                    f"object not found: {absent}",
                ),
            ),
            (
                "malformed subject",
                lambda: self._mutate_merge_request(
                    self.merge_request_ids[4], {"subject": "not-an-object"}
                ),
                lambda message: self.assertEqual(
                    message, "merge request subject is malformed"
                ),
            ),
            (
                "invalid merge-request id",
                lambda: self._mutate_merge_request(
                    self.merge_request_ids[5], {"id": "bad/id"}
                ),
                lambda message: self.assertIn("merge-request ID", message),
            ),
            (
                "corrupt escrow blob",
                lambda: self.object_path(self.blob_object_id()).write_text(
                    "{broken", encoding="utf-8"
                ),
                lambda message: self.assertIn("invalid object JSON", message),
            ),
            (
                "absent escrow blob",
                lambda: self.object_path(self.blob_object_id()).unlink(),
                lambda message: self.assertIn(
                    "object not found while bundling", message
                ),
            ),
        ]
        for label, corrupt, check in cases:
            with self.subTest(case=label):
                self.setUp()
                try:
                    corrupt()
                    wire, observation = self.exchange()
                    self.assertEqual(
                        self.status_line(wire), b"HTTP/1.0 400 Bad Request"
                    )
                    check(self.body_of(wire)["error"])
                    self.assert_no_partial_response(wire, observation)
                finally:
                    self.tearDown()

    def _mutate_merge_request(self, mr_id, changes):
        record = self.read_merge_request(mr_id)
        record.update(changes)
        self.write_merge_request(mr_id, record)

    # ------------------------------------------------------------------
    # multi-fault first-error precedence against the pre-fix route
    # ------------------------------------------------------------------

    # Each injector faults one merge request (or the shared branch authority) in
    # one class.  ``compatible`` is False for the classes on which the pre-fix
    # route did not fail closed — it either crashed untyped or silently followed
    # an authority symlink — and where the contract therefore requires an
    # explicit typed rejection instead of pre-fix parity.  Every other class is
    # a strict-parity class: the repaired route must reproduce the pre-fix first
    # error exactly, whichever filename carries it.
    def _fault_injectors(self):
        namespace = self.namespace

        def unparseable(mr_id):
            self.merge_request_path(mr_id).write_text("{not json", encoding="utf-8")

        def directory_authority(mr_id):
            path = self.merge_request_path(mr_id)
            path.unlink()
            path.mkdir()

        def dangling_symlink(mr_id):
            path = self.merge_request_path(mr_id)
            path.unlink()
            path.symlink_to("absent-authority.json")

        def following_symlink(mr_id):
            path = self.merge_request_path(mr_id)
            sibling = path.with_name("shadow-" + path.name)
            sibling.write_bytes(path.read_bytes())
            path.unlink()
            path.symlink_to(sibling.name)

        def future_version(mr_id):
            self._mutate_merge_request(mr_id, {"version": 2})

        def foreign_target(mr_id):
            self._mutate_merge_request(
                mr_id, {"target_url": "cf+http://127.0.0.1:9/other/app/main"}
            )

        def legacy_cross_project(mr_id):
            record = self.read_merge_request(mr_id)
            record.pop("subject", None)
            record.pop("subject_digest", None)
            record["source_url"] = "cf+http://127.0.0.1:9/other/app/feature"
            self.write_merge_request(mr_id, record)

        def sealed_reference(mr_id):
            self._mutate_merge_request(mr_id, {"source_head": 7})

        def subject_digest(mr_id):
            self._mutate_merge_request(
                mr_id, {"subject_digest": "sha256:" + "0" * 64}
            )

        def escrow_digest(mr_id):
            record = self.read_merge_request(mr_id)
            record["subject"] = dict(record["subject"])
            record["subject"]["source_object_set_digest"] = "sha256:" + "0" * 64
            record["subject_digest"] = namespace["merge_request_subject_digest"](
                record["subject"]
            )
            self.write_merge_request(mr_id, record)

        def branch_version(_mr_id):
            path = self.branch_path(TARGET_BRANCH)
            record = json.loads(path.read_text(encoding="utf-8"))
            record["version"] = 2
            path.write_text(json.dumps(record, sort_keys=True), encoding="utf-8")

        def branch_symlink(_mr_id):
            path = self.branch_path(TARGET_BRANCH)
            sibling = path.with_name("shadow-main.json")
            sibling.write_bytes(path.read_bytes())
            path.unlink()
            path.symlink_to(sibling.name)

        return [
            ("invalid JSON", unparseable, False),
            ("directory authority", directory_authority, False),
            ("dangling symlink", dangling_symlink, False),
            ("followed symlink", following_symlink, False),
            ("branch symlink", branch_symlink, False),
            ("future version", future_version, True),
            ("foreign target URL", foreign_target, True),
            ("legacy cross-project", legacy_cross_project, True),
            ("sealed reference", sealed_reference, True),
            ("subject digest", subject_digest, True),
            ("escrow digest", escrow_digest, True),
            ("branch version", branch_version, True),
        ]

    TYPED_CAPTURE_PREFIXES = (
        AUTHORITY_FAILURE_PREFIX,
        "invalid JSON: ",
        "bounded JSON file is invalid: ",
        "merge request record is unavailable: ",
    )

    def assert_typed_capture_rejection(self, message):
        """A documented capture divergence still names one typed family."""
        self.assertTrue(
            message.startswith(self.TYPED_CAPTURE_PREFIXES),
            f"unclassified capture diagnostic: {message}",
        )

    def _legacy_outcome(self):
        """Classify the pre-fix route's outcome for the current storage tree."""
        try:
            status, payload = self.legacy_request_list(self.storage)
        except self.namespace["CFError"] as exc:
            return ("typed", str(exc))
        except BaseException as exc:  # untyped pre-fix crash: no publication
            return ("untyped", type(exc).__name__)
        if status == 200:
            return ("published", len(payload["merge_requests"]))
        return ("typed", payload.get("error"))

    def _target_outcome(self):
        wire, observation = self.exchange()
        self.assert_no_ownership_debt(observation)
        if wire.startswith(b"HTTP/1.0 200 OK"):
            return ("published", len(self.body_of(wire)["merge_requests"])), wire
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assert_no_partial_response(wire, observation)
        return ("typed", self.body_of(wire)["error"]), wire

    def test_multi_fault_error_precedence_matches_the_pre_fix_route(self):
        """Two faulty requests resolve to the pre-fix route's first error.

        Every ordered pair of distinct fault classes is injected at the first and
        last filename and then swapped.  For a pair of strict-parity classes the
        repaired route must publish exactly the pre-fix diagnostic, in either
        order.  For a pair involving a class the pre-fix route did not fail
        closed on, the repaired route must publish either that same pre-fix
        diagnostic or one member of the documented typed-capture family — never a
        partial list, never an empty wire, never ownership debt.
        """
        injectors = self._fault_injectors()
        ordered_ids = sorted(self.merge_request_ids)
        first, last = ordered_ids[0], ordered_ids[-1]
        strict = 0
        documented = 0
        for early_index, (early_label, early, early_ok) in enumerate(injectors):
            for late_label, late, late_ok in injectors[early_index + 1 :]:
                for order, (a, b) in (
                    ("early-first", ((first, early), (last, late))),
                    ("late-first", ((first, late), (last, early))),
                ):
                    with self.subTest(
                        early=early_label, late=late_label, order=order
                    ):
                        self.setUp()
                        try:
                            a[1](a[0])
                            b[1](b[0])
                            legacy = self._legacy_outcome()
                            target, _ = self._target_outcome()
                            if early_ok and late_ok:
                                strict += 1
                                self.assertEqual(legacy[0], "typed")
                                self.assertEqual(
                                    target,
                                    legacy,
                                    "the repaired route reported a different "
                                    "first error than the pre-fix route",
                                )
                                continue
                            documented += 1
                            self.assertEqual(target[0], "typed")
                            if legacy[0] == "typed" and target[1] == legacy[1]:
                                continue
                            self.assert_typed_capture_rejection(target[1])
                        finally:
                            self.tearDown()
        strict_classes = sum(1 for _, _, ok in injectors if ok)
        self.assertEqual(strict, strict_classes * (strict_classes - 1))
        self.assertGreater(documented, 0)

    def test_eager_capture_faults_break_multi_fault_precedence(self):
        """The precedence fixtures are non-vacuous: eager capture diverges.

        Restoring the reviewed regression — raising a record's read/parse/version
        fault inside authority capture instead of retaining it — must make at
        least one multi-fault fixture report a different first error than the
        pre-fix route, and the repaired route must agree on that same fixture.
        """
        ordered_ids = sorted(self.merge_request_ids)
        first, last = ordered_ids[0], ordered_ids[-1]
        fixtures = [
            (
                "sealed reference then version",
                lambda: self._mutate_merge_request(first, {"source_head": 7}),
                lambda: self._mutate_merge_request(last, {"version": 2}),
            ),
            (
                "foreign target then version",
                lambda: self._mutate_merge_request(
                    first, {"target_url": "cf+http://127.0.0.1:9/other/app/main"}
                ),
                lambda: self._mutate_merge_request(last, {"version": 2}),
            ),
            (
                "subject digest then unparseable",
                lambda: self._mutate_merge_request(
                    first, {"subject_digest": "sha256:" + "0" * 64}
                ),
                lambda: self.merge_request_path(last).write_text(
                    "{not json", encoding="utf-8"
                ),
            ),
        ]
        for label, corrupt_first, corrupt_last in fixtures:
            with self.subTest(case=label):
                self.setUp()
                try:
                    corrupt_first()
                    corrupt_last()
                    legacy = self._legacy_outcome()
                    self.assertEqual(legacy[0], "typed")
                    repaired, _ = self._target_outcome()
                    self.assertEqual(repaired, legacy)
                    with self.eager_authority_faults():
                        wire, observation = self.exchange()
                    self.assertEqual(
                        self.status_line(wire), b"HTTP/1.0 400 Bad Request"
                    )
                    self.assertNotEqual(
                        self.body_of(wire)["error"],
                        legacy[1],
                        "eager capture faults did not change the first error, "
                        "so this fixture proves nothing",
                    )
                    self.assert_no_partial_response(wire, observation)
                finally:
                    self.tearDown()

    # ------------------------------------------------------------------
    # no route answers an unusable filesystem authority with a blank wire
    # ------------------------------------------------------------------

    def assert_typed_authority_failure(self, wire, observation, target=None):
        """One complete typed 400 names the offending filesystem authority."""
        self.assertTrue(wire, "the route closed the connection with no reply")
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertTrue(message.startswith(AUTHORITY_FAILURE_PREFIX), message)
        if target is not None:
            # A route that rejects the authority through an already-open
            # directory capability names it relative to that capability.
            self.assertIn(Path(str(target)).name, message)
        self.assert_no_partial_response(wire, observation)
        return message

    def _symlink_over(self, path):
        """Replace one authority file with a symlink to identical sibling bytes."""
        sibling = path.with_name("shadow-" + path.name)
        sibling.write_bytes(path.read_bytes())
        path.unlink()
        path.symlink_to(sibling.name)
        return sibling

    def test_symlinked_authority_records_publish_a_typed_diagnostic(self):
        """A symlinked pinned record is rejected, never followed or dropped."""
        victims = {
            "merge request record": lambda: self.merge_request_path(
                sorted(self.merge_request_ids)[0]
            ),
            "consulted branch record": lambda: self.branch_path(TARGET_BRANCH),
            "project descriptor": self.descriptor_path,
        }
        for label, resolve in victims.items():
            with self.subTest(authority=label):
                self.setUp()
                try:
                    victim = resolve()
                    self._symlink_over(victim)
                    wire, observation = self.exchange()
                    self.assert_typed_authority_failure(
                        wire, observation, target=victim
                    )
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"], "complete"
                    )
                finally:
                    self.tearDown()

    def test_every_shared_read_and_post_route_fails_closed_on_a_symlink(self):
        """No neighbouring route closes the connection on a symlinked record."""
        requests = {
            "merge-requests": MR_LIST_REQUEST,
            "branches": BRANCH_LIST_REQUEST,
            "doctor": DOCTOR_REQUEST,
            "bundle": BUNDLE_REQUEST,
            "gc": post_request("/v1/projects/org/app/gc"),
            "upload": post_request(
                "/v1/projects/org/app/branches/main/upload"
            ),
        }
        for label, request in requests.items():
            with self.subTest(route=label):
                self.setUp()
                try:
                    self._symlink_over(self.branch_path(TARGET_BRANCH))
                    wire, observation = self.exchange(request=request)
                    self.assertTrue(
                        wire, f"the {label} route published a zero-byte reply"
                    )
                    self.assertTrue(
                        wire.startswith(b"HTTP/1.0 "), self.status_line(wire)
                    )
                    self.assertIn(b"\r\n\r\n", wire)
                    self.assertIn("error", self.body_of(wire))
                    self.assertNotIn(b'"merge_requests"', wire)
                    self.assert_no_ownership_debt(observation)
                finally:
                    self.tearDown()

    def test_dispatcher_publishes_every_route_filesystem_failure(self):
        """``do_GET``/``do_POST`` publish a typed 400 for any escaping OSError."""
        globals_ = self.route_globals
        victim = "/authority/main.json"
        routes = {
            "handle_http_list": BRANCH_LIST_REQUEST,
            "handle_http_doctor": DOCTOR_REQUEST,
            "handle_http_bundle": BUNDLE_REQUEST,
            "handle_http_request_list": MR_LIST_REQUEST,
            "handle_http_upload": post_request(
                "/v1/projects/org/app/branches/main/upload"
            ),
            "handle_http_gc": post_request("/v1/projects/org/app/gc"),
            "handle_http_request_merge": post_request(
                "/v1/projects/org/app/merge-requests"
            ),
        }
        for name, request in routes.items():
            with self.subTest(route=name):

                def raising(*args, _victim=victim, **kwargs):
                    raise OSError(40, "Too many levels of symbolic links", _victim)

                with self.patched(globals_, name, raising):
                    wire, observation = self.exchange(request=request)
                message = self.assert_typed_authority_failure(
                    wire, observation, target=victim
                )
                self.assertIn("Too many levels of symbolic links", message)

        # Non-vacuity: the arm is specific to filesystem faults.  A non-OSError
        # failure still abandons the connection, so the typed assertions above
        # cannot be satisfied by a catch-all handler.
        def exploding(*args, **kwargs):
            raise MemoryError("not a filesystem fault")

        with self.abandoned_connection_expected():
            with self.patched(globals_, "handle_http_request_list", exploding):
                wire, observation = self.exchange()
        self.assertEqual(wire, b"")
        self.assert_no_ownership_debt(observation)

        # A route-raised ``ConnectionError``/``TimeoutError`` is a filesystem
        # authority failure, not a client disconnect: the class alone proves
        # nothing, so it must still publish the typed diagnostic.
        for exception in (
            ConnectionResetError(104, "Connection reset by peer", victim),
            TimeoutError(110, "Connection timed out", victim),
        ):
            with self.subTest(route_raised=type(exception).__name__):

                def raising(*args, _exception=exception, **kwargs):
                    raise _exception

                with self.patched(
                    globals_, "handle_http_request_list", raising
                ):
                    wire, observation = self.exchange()
                self.assert_typed_authority_failure(
                    wire, observation, target=victim
                )

    # ------------------------------------------------------------------
    # the final snapshot/digest fence
    # ------------------------------------------------------------------

    def _mutate_before_final_fence(self, victim, mutate):
        """Run the live route, mutating ``victim`` just before the last fence."""
        Session = self.route_globals["SealedCommitValidationSession"]
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

    @staticmethod
    def _same_length_digit_swap(path, before):
        """Change one content byte while keeping inode, size, and mtime."""
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

    def test_reachable_record_mutation_fails_closed_at_the_final_fence(self):
        """Same path, inode, size, and mtime never authorize stale content."""
        for label, object_id in (
            ("escrow blob", self.blob_object_id()),
            ("shared commit", self.target_head),
        ):
            with self.subTest(case=label):
                self.setUp()
                try:
                    victim = self.object_path(object_id)
                    wire, observation, before, after = (
                        self._mutate_before_final_fence(
                            victim, self._same_length_digit_swap
                        )
                    )
                    # The mutation is invisible to path, inode, size, and mtime.
                    self.assertEqual(after.st_ino, before.st_ino)
                    self.assertEqual(after.st_size, before.st_size)
                    self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
                    self.assertEqual(
                        self.status_line(wire), b"HTTP/1.0 400 Bad Request"
                    )
                    message = self.body_of(wire)["error"]
                    self.assertIn("sealed validation subject changed", message)
                    self.assertIn("digest_changed", message)
                    self.assertIn(object_id, message)
                    self.assertIn(str(victim), message)
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"],
                        "subject_changed",
                    )
                    self.assert_no_partial_response(wire, observation)
                finally:
                    self.tearDown()

    def _mutate_before_authority_fence(self, victim, mutate):
        """Run the live route, mutating ``victim`` just before the authority fence."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        original = Operation.finalize_authority
        before = victim.stat()
        state = {"mutated": False, "after": None}

        def mutating_fence(owner, *args, **kwargs):
            if not state["mutated"]:
                state["mutated"] = True
                mutate(victim, before)
                state["after"] = victim.stat()
            return original(owner, *args, **kwargs)

        with self.patched(Operation, "finalize_authority", mutating_fence):
            wire, observation = self.exchange()
        self.assertTrue(state["mutated"], "the authority fence never ran")
        return wire, observation, before, state["after"]

    def test_authority_fence_rejects_same_metadata_replacement(self):
        """Path, inode, size, and mtime never authorize replaced authority bytes.

        Every consumed authority file — each pinned merge-request record, the
        consulted branch record, and the project descriptor — is reproved by
        content digest, so a replacement that preserves all observable metadata
        is rejected instead of published.
        """
        victims = {
            "merge request record": (
                lambda: self.merge_request_path(sorted(self.merge_request_ids)[0]),
                True,
            ),
            "consulted branch record": (
                lambda: self.branch_path(TARGET_BRANCH),
                True,
            ),
            # The ordered project gate keeps its own descriptor identity fence,
            # which observes a replaced descriptor first; either fence must fail
            # the request closed and name the offending authority.
            "project descriptor": (self.descriptor_path, False),
        }
        for label, (resolve, exact) in victims.items():
            with self.subTest(authority=label):
                self.setUp()
                try:
                    victim = resolve()
                    wire, observation, before, after = (
                        self._mutate_before_authority_fence(
                            victim, self._same_length_digit_swap
                        )
                    )
                    self.assertEqual(after.st_ino, before.st_ino)
                    self.assertEqual(after.st_size, before.st_size)
                    self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
                    self.assertEqual(
                        self.status_line(wire), b"HTTP/1.0 400 Bad Request"
                    )
                    message = self.body_of(wire)["error"]
                    if exact:
                        self.assertTrue(
                            message.startswith(AUTHORITY_CHANGED_PREFIX), message
                        )
                        self.assertIn(str(victim), message)
                        self.assertIn("digest_changed", message)
                    else:
                        self.assertTrue(
                            message.startswith(AUTHORITY_CHANGED_PREFIX)
                            or message.startswith(
                                "remote project descriptor changed: "
                            ),
                            message,
                        )
                        self.assertIn(str(victim.parent), message)
                    self.assert_no_partial_response(wire, observation)
                finally:
                    self.tearDown()

    def test_authority_fence_rejects_an_appearing_absent_authority(self):
        """An authority captured as absent may not be created mid-request."""
        branch = self.branch_path(TARGET_BRANCH)
        pinned = branch.read_bytes()
        branch.unlink()
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        original = Operation.finalize_authority
        state = {"restored": False}

        def restoring_fence(owner, *args, **kwargs):
            if not state["restored"]:
                state["restored"] = True
                branch.write_bytes(pinned)
            return original(owner, *args, **kwargs)

        with self.patched(Operation, "finalize_authority", restoring_fence):
            wire, observation = self.exchange()
        self.assertTrue(state["restored"])
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertTrue(message.startswith(AUTHORITY_CHANGED_PREFIX), message)
        self.assertIn("appeared", message)
        self.assert_no_partial_response(wire, observation)

    def test_pinned_mr_and_branch_authority_mutation_has_no_effect(self):
        """A late authority mutation cannot reach the pinned projection."""
        baseline_wire, baseline = self.exchange()
        baseline_body = self.assert_ok(baseline_wire)
        self.assert_no_ownership_debt(baseline)

        victims = {
            "merge request record": lambda: self.merge_request_path(
                sorted(self.merge_request_ids)[0]
            ),
            "consulted branch record": lambda: self.branch_path(TARGET_BRANCH),
        }
        for label, resolve in victims.items():
            with self.subTest(case=label):
                self.setUp()
                target = resolve()
                try:
                    wire, observation, before, after = (
                        self._mutate_before_final_fence(
                            target, self._same_length_digit_swap
                        )
                    )
                    self.assertEqual(after.st_ino, before.st_ino)
                    self.assertEqual(after.st_size, before.st_size)
                    self.assertEqual(after.st_mtime_ns, before.st_mtime_ns)
                    self.assertNotEqual(
                        target.read_bytes(),
                        (self.template / "storage").joinpath(
                            target.relative_to(self.storage)
                        ).read_bytes(),
                    )
                    # The response is the pinned snapshot, not the mutation.
                    self.assertEqual(self.assert_ok(wire), baseline_body)
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"], "complete"
                    )
                    self.assert_no_ownership_debt(observation)
                finally:
                    self.tearDown()

    # ------------------------------------------------------------------
    # memoized escrow work and the shared-head counter curves
    # ------------------------------------------------------------------

    # Object traversal, hashing, and semantic counters that must not move at all
    # when only the merge-request count grows, because every request shares one
    # pinned source head and therefore one memoized escrow object set.
    CONSTANT_ESCROW_COUNTERS = (
        "object_records",
        "memo_misses",
        "unique_nodes_discovered",
        "semantic_validations",
        "edge_occurrences",
    )
    CONSTANT_ESCROW_DIMENSIONS = ("graph_nodes", "graph_edges", "object_records")
    # Per-record work that may still scale: each added merge request reads its
    # own record and fences it, revalidates it once, and offers its own two
    # sealed references to the shared session memo.
    AFFINE_RECORD_COUNTERS = (
        ("files_read", 2),
        ("snapshot_revalidation_records", 1),
        ("frontier_insertion_attempts", 2),
        ("revisits", 2),
    )

    def _shared_head_observation(self, count, *, profile=None):
        """Measure one request over ``count`` requests sharing one head."""
        self.setUp()
        try:
            identifiers = self.write_shared_head_merge_requests(count)
            with self.tracked_sessions() as sessions:
                with self.recorded_accounted_reads() as reads:
                    wire, observation = self.exchange(
                        profile or self.profile(deadline_ms=600_000),
                        clock=_Clock(),
                    )
            body = self.assert_ok(wire)
            self.assertEqual(len(body["merge_requests"]), count)
            self.assert_sessions_closed(sessions, expected=1)
            self.assertEqual(
                sorted(set(reads)), sorted(reads), "a path was read twice"
            )
            self.assert_no_ownership_debt(observation)
            return {
                "count": count,
                "identifiers": identifiers,
                "snapshot": observation["snapshot"],
                "polls": observation["deadline"]["poll_count"],
                "reads": list(reads),
            }
        finally:
            self.tearDown()

    def test_memoized_escrow_work_is_constant_in_merge_request_count(self):
        """One shared head keeps traversal, hashing, and reads out of the slope.

        At 1/8/24/80/200 merge requests against one pinned source head the route
        opens exactly one sealed-validation session, reads no path twice, keeps
        every traversal and semantic counter constant, and grows the remaining
        per-record counters with one exact, cardinality-independent slope.
        """
        measured = [
            self._shared_head_observation(count)
            for count in SHARED_HEAD_CARDINALITIES
        ]
        first = measured[0]["snapshot"]
        for observed in measured[1:]:
            snapshot = observed["snapshot"]
            for counter in self.CONSTANT_ESCROW_COUNTERS:
                self.assertEqual(
                    snapshot["counters"][counter],
                    first["counters"][counter],
                    f"{counter} scaled with merge-request count",
                )
            for dimension in self.CONSTANT_ESCROW_DIMENSIONS:
                self.assertEqual(
                    snapshot["cumulative"][dimension],
                    first["cumulative"][dimension],
                    f"{dimension} scaled with merge-request count",
                )
            self.assertEqual(
                snapshot["maximum"]["maximum_frontier"],
                first["maximum"]["maximum_frontier"],
            )
            # Memo reuse grows, which is the memo doing its job.
            self.assertGreater(
                snapshot["counters"]["memo_hits"], first["counters"]["memo_hits"]
            )

        # The remaining per-record work is exactly affine in cardinality with an
        # exact, cardinality-independent slope, and hashing has one flat slope.
        for counter, slope in self.AFFINE_RECORD_COUNTERS + (("bytes_hashed", None),):
            slopes = {
                (later["count"] - earlier["count"]): (
                    later["snapshot"]["counters"][counter]
                    - earlier["snapshot"]["counters"][counter]
                )
                for earlier, later in zip(measured, measured[1:])
            }
            observed = {delta / step for step, delta in slopes.items()}
            self.assertEqual(
                len(observed),
                1,
                f"{counter} is not affine in merge-request count: {slopes}",
            )
            if slope is not None:
                self.assertEqual(observed.pop(), float(slope), counter)

    def test_removing_the_escrow_memo_restores_the_linear_slope(self):
        """The constancy assertions are non-vacuous: no memo means linear work."""
        counts = (1, 8, 24)
        memoized = [self._shared_head_observation(count) for count in counts]
        unmemoized = []
        for count in counts:
            self.setUp()
            try:
                self.write_shared_head_merge_requests(count)
                with self.unmemoized_escrow():
                    wire, observation = self.exchange(
                        self.profile(deadline_ms=600_000), clock=_Clock()
                    )
                self.assert_ok(wire)
                self.assert_no_ownership_debt(observation)
                unmemoized.append(observation["snapshot"])
            finally:
                self.tearDown()
        self.assertEqual(
            memoized[0]["snapshot"]["counters"]["edge_occurrences"],
            memoized[-1]["snapshot"]["counters"]["edge_occurrences"],
            "edge occurrences were not constant with the memo",
        )
        for counter in ("edge_occurrences", "revisits", "bytes_hashed"):
            self.assertGreater(
                unmemoized[-1]["counters"][counter],
                unmemoized[0]["counters"][counter],
                f"{counter} did not grow without the memo",
            )
            self.assertGreater(
                unmemoized[-1]["counters"][counter],
                memoized[-1]["snapshot"]["counters"][counter],
                f"the memo saved no {counter}",
            )
        self.assertGreater(
            unmemoized[-1]["cumulative"]["graph_edges"],
            memoized[-1]["snapshot"]["cumulative"]["graph_edges"],
        )

    def test_stable_project_id_reads_the_descriptor_once_from_charged_bytes(self):
        """``stable_project_id`` owns exactly one accounted descriptor read.

        This oracle is scoped to that helper's accounted-read boundary.  The
        whole-route physical descriptor protocol — including the project gate's
        pin, acquisition recheck, and terminal fence reads — is asserted by
        ``test_descriptor_read_protocol_is_exact_and_route_wide``.
        """
        descriptor = self.descriptor_path()
        with self.forbidden_legacy_helpers():
            with self.recorded_accounted_reads() as reads:
                wire, observation = self.exchange()
        self.assert_ok(wire)
        self.assertEqual(
            [path for path in reads if path == str(descriptor)],
            [str(descriptor)],
            "the project descriptor was not read exactly once",
        )
        counters = observation["snapshot"]["counters"]
        # Capture charges the exact bytes consumed and the fence charges them
        # again; a stat-derived charge could not agree with both.
        self.assertEqual(
            counters["snapshot_revalidation_bytes"]
            - sum(
                self.object_path(record["object_id"]).stat().st_size
                for record in self.all_records
            )
            - sum(path.stat().st_size for path in self.merge_request_paths())
            - self.branch_path(TARGET_BRANCH).stat().st_size,
            descriptor.stat().st_size,
        )
        self.assert_no_ownership_debt(observation)

    # ------------------------------------------------------------------
    # exact, exact-minus-one, and exact-plus-one resource bounds
    # ------------------------------------------------------------------

    def test_exact_minus_one_and_plus_one_limits_bound_every_stage(self):
        """Each stage's measured cost passes at exactly its limit and no less."""
        baseline_wire, baseline = self.exchange()
        self.assert_ok(baseline_wire)
        snapshot = baseline["snapshot"]
        cumulative = snapshot["cumulative"]
        peak = snapshot["peak"]
        maximum = snapshot["maximum"]
        counters = snapshot["counters"]
        cases = [
            # authority capture and pinned record reads
            ("aggregate_bytes", cumulative["aggregate_bytes"]),
            ("object_records", cumulative["object_records"]),
            # shared object/commit and escrow traversal work
            ("graph_nodes", cumulative["graph_nodes"]),
            ("graph_edges", cumulative["graph_edges"]),
            ("maximum_frontier", maximum["maximum_frontier"]),
            # the final snapshot fence
            (
                "snapshot_revalidation_records",
                cumulative["snapshot_revalidation_records"],
            ),
            (
                "snapshot_revalidation_bytes",
                cumulative["snapshot_revalidation_bytes"],
            ),
            # retained/temporary projection ownership
            ("retained_bytes", peak["retained_bytes"]),
            ("temporary_bytes", peak["temporary_bytes"]),
            # every counted file, record, node, and edge occurrence
            ("counter_values", max(counters.values())),
            # response encoding and publication
            ("response_bytes", cumulative["response_bytes"]),
        ]
        for dimension, exact in cases:
            self.assertGreater(exact, 0, dimension)
            for bound, limit in (("exact", exact), ("exact-plus-one", exact + 1)):
                with self.subTest(dimension=dimension, bound=bound):
                    wire, observation = self.exchange(
                        self.profile({dimension: limit})
                    )
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
                self.assertIn(
                    diagnostic["phase"],
                    set(self.namespace["DECISION_WORK_PHASES"]),
                )
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], "budget_exceeded"
                )
                self.assertIsNone(observation["deadline_diagnostic"])
                self.assert_no_partial_response(wire, observation)

    def test_files_records_nodes_and_edges_are_exactly_accounted(self):
        """Every counted read, record, node, and edge has an exact total."""
        wire, observation = self.exchange()
        self.assert_ok(wire)
        snapshot = observation["snapshot"]
        counters = snapshot["counters"]
        cumulative = snapshot["cumulative"]
        unique_objects = len(self.all_records)

        object_bytes = sum(
            self.object_path(record["object_id"]).stat().st_size
            for record in self.all_records
        )
        mr_bytes = sum(path.stat().st_size for path in self.merge_request_paths())
        branch_bytes = self.branch_path(TARGET_BRANCH).stat().st_size
        descriptor_bytes = (self.project_root / "remote_project.json").stat().st_size

        authority_bytes = mr_bytes + branch_bytes + descriptor_bytes
        authority_files = MR_COUNT + CONSULTED_AUTHORITY_FILES

        # Each object is read once for validation and once for the final fence;
        # each MR record, the one consulted branch, and the pinned project
        # descriptor are read exactly once for capture and once for the final
        # authority fence.  Nothing is reopened.
        self.assertEqual(
            counters["files_read"],
            2 * unique_objects + 2 * authority_files,
        )
        self.assertEqual(
            counters["file_bytes_read"],
            2 * object_bytes + 2 * authority_bytes,
        )
        self.assertEqual(counters["object_records"], unique_objects)
        self.assertEqual(cumulative["object_records"], unique_objects)
        self.assertEqual(counters["unique_nodes_discovered"], unique_objects)
        self.assertEqual(counters["record_reads"], unique_objects + 1)
        self.assertEqual(counters["memo_misses"], unique_objects)
        self.assertEqual(cumulative["graph_nodes"], self.commit_count)
        self.assertEqual(counters["edge_occurrences"], cumulative["graph_edges"])
        self.assertEqual(
            counters["snapshot_revalidation_records"],
            unique_objects + authority_files,
        )
        self.assertEqual(
            counters["snapshot_revalidation_records"],
            cumulative["snapshot_revalidation_records"],
        )
        self.assertEqual(
            cumulative["snapshot_revalidation_bytes"],
            object_bytes + authority_bytes,
        )
        self.assertEqual(
            cumulative["response_bytes"], int(self._content_length(wire))
        )
        self.assert_no_ownership_debt(observation)

    @staticmethod
    def _content_length(wire):
        head = wire.split(b"\r\n\r\n", 1)[0]
        for line in head.split(b"\r\n"):
            if line.lower().startswith(b"content-length:"):
                return line.split(b":", 1)[1].strip()
        raise AssertionError("response carried no Content-Length header")

    # ------------------------------------------------------------------
    # deterministic deadline expiry
    # ------------------------------------------------------------------

    def test_deadline_expires_deterministically_at_every_boundary(self):
        """Authority, validation, fence, and response phases all fail closed."""
        profile = self.profile(deadline_ms=10)
        phases, baseline = self.record_deadline_phases(profile)
        counts = Counter(phases)
        expected_phases = [
            "merge request list authority",
            "merge request list validation",
            "merge request list projection",
            "merge request list snapshot revalidation",
            "merge request list response encoding",
            "merge request list response headers",
            "merge request list response publication",
        ]
        for phase in (
            "merge request list planning",
            "merge request list project gate",
        ):
            self.assertGreaterEqual(counts[phase], 1, phase)
        for phase in expected_phases:
            self.assertGreaterEqual(counts[phase], 1, phase)
        self.assertGreater(counts["merge request list authority"], MR_COUNT)
        self.assertGreater(counts["merge request list validation"], MR_COUNT)
        self.assertGreater(counts["merge request list snapshot revalidation"], 0)

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
        # any merge-request authority record at all.
        immediate_wire, immediate = self.exchange(profile, clock=_ExpiringClock(1))
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
    # one stable polling policy per bounded work quantum
    # ------------------------------------------------------------------

    def test_poll_density_is_affine_and_free_of_per_request_escrow_polls(self):
        """Deadline polling grows only with per-record work, never with escrow.

        With one shared source head the escrow traversal and digest are polled
        once per request, so the total poll count is exactly affine in
        merge-request count.  Removing the memo makes the same measurement
        superlinear, which is what the policy is defending against.
        """
        counts = (1, 8, 24, 80)
        measured = [self._shared_head_observation(count) for count in counts]
        slopes = {
            later["count"] - earlier["count"]: later["polls"] - earlier["polls"]
            for earlier, later in zip(measured, measured[1:])
        }
        unit_slopes = {delta / step for step, delta in slopes.items()}
        self.assertEqual(
            len(unit_slopes),
            1,
            f"deadline polling is not affine in merge-request count: {slopes}",
        )
        per_request = unit_slopes.pop()
        self.assertGreater(per_request, 0, "cancellation coverage stopped scaling")

        unmemoized = []
        for count in counts:
            self.setUp()
            try:
                self.write_shared_head_merge_requests(count)
                with self.unmemoized_escrow():
                    wire, observation = self.exchange(
                        self.profile(deadline_ms=600_000), clock=_Clock()
                    )
                self.assert_ok(wire)
                unmemoized.append(observation["deadline"]["poll_count"])
            finally:
                self.tearDown()
        naive_slopes = {
            later - earlier: naive_later - naive_earlier
            for (earlier, later), (naive_earlier, naive_later) in zip(
                zip(counts, counts[1:]), zip(unmemoized, unmemoized[1:])
            )
        }
        naive_unit = {delta / step for step, delta in naive_slopes.items()}
        self.assertGreater(
            min(naive_unit),
            per_request,
            "the memo removed no polling, so this measurement proves nothing",
        )
        self.assertLess(
            measured[-1]["polls"],
            unmemoized[-1],
            "memoized polling is not cheaper than per-merge-request polling",
        )

    def test_a_long_single_traversal_stays_interruptible(self):
        """One request over one long traversal cancels at many exact boundaries."""
        profile = self.profile(deadline_ms=10)
        self.write_shared_head_merge_requests(1)
        phases, baseline = self.record_deadline_phases(profile)
        validation = [
            index
            for index, phase in enumerate(phases, start=1)
            if phase == "merge request list validation"
        ]
        self.assertGreater(
            len(validation),
            len(self.all_records),
            "one merge request's traversal is not polled per visited record",
        )
        for fraction in (0.1, 0.25, 0.5, 0.75, 0.9):
            expire_read = validation[int(len(validation) * fraction)]
            with self.subTest(expire_read=expire_read):
                wire, observation = self.exchange(
                    profile, clock=_ExpiringClock(expire_read)
                )
                self.assertEqual(
                    self.status_line(wire), b"HTTP/1.0 503 Service Unavailable"
                )
                self.assertEqual(
                    observation["deadline"]["expired_phase"],
                    "merge request list validation",
                )
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"],
                    "deadline_exceeded",
                )
                self.assert_no_partial_response(wire, observation)
        self.assertEqual(baseline["snapshot"]["terminal_reason"], "complete")

    def test_removing_a_production_poll_breaks_its_phase_cancellation(self):
        """Every phase poll is required: silencing one removes its cancellation."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        profile = self.profile(deadline_ms=10)
        phases, _ = self.record_deadline_phases(profile)
        baseline_counts = Counter(phases)
        # Silencing one phase helper must remove every poll it owns.  The
        # projection phase keeps exactly one poll that the payload collector
        # names directly at its own final boundary.
        for helper, phase, residual in (
            ("_authority_poll", "merge request list authority", 0),
            ("_validation_poll", "merge request list validation", 0),
            ("_projection_poll", "merge request list projection", 1),
            (
                "_revalidation_poll",
                "merge request list snapshot revalidation",
                0,
            ),
        ):
            polls = [
                index
                for index, observed in enumerate(phases, start=1)
                if observed == phase
            ]
            self.assertTrue(polls, phase)
            expire_read = polls[len(polls) // 2]
            with self.subTest(poll=helper):
                # With the production poll in place the request cancels in this
                # exact phase.
                wire, observation = self.exchange(
                    profile, clock=_ExpiringClock(expire_read)
                )
                self.assertEqual(
                    observation["deadline"]["expired_phase"], phase
                )
                self.assertEqual(
                    self.status_line(wire), b"HTTP/1.0 503 Service Unavailable"
                )

                # Silencing it removes every poll that helper owns.
                observed_counts = Counter()
                deadline = self.route_globals["HTTPRequestDeadline"]
                original_check = deadline.check

                def record_check(owner, observed):
                    observed_counts[observed] += 1
                    return original_check(owner, observed)

                with self.patched(Operation, helper, lambda owner: None):
                    with self.patched(deadline, "check", record_check):
                        silenced_wire, silenced = self.exchange(
                            self.profile(deadline_ms=600_000), clock=_Clock()
                        )
                self.assert_ok(silenced_wire)
                self.assertGreater(baseline_counts[phase], residual, phase)
                self.assertEqual(
                    observed_counts[phase],
                    residual,
                    f"{phase} survived silencing {helper}, so the production "
                    "poll it names is not the one under test",
                )
                self.assert_no_ownership_debt(silenced)

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
            if phase == "merge request list validation"
        ]

        def budget_case():
            return self.exchange(self.profile({"graph_nodes": exact_nodes - 1}))

        def deadline_case():
            return self.exchange(
                deadline_profile,
                clock=_ExpiringClock(validation_polls[len(validation_polls) // 2]),
            )

        def subject_case():
            victim = self.object_path(self.blob_object_id())
            wire, observation, _, _ = self._mutate_before_final_fence(
                victim, self._same_length_digit_swap
            )
            return wire, observation

        def validation_case():
            self.object_path(self.blob_object_id()).unlink()
            return self.exchange()

        cases = [
            ("budget_exceeded", budget_case),
            ("deadline_exceeded", deadline_case),
            ("subject_changed", subject_case),
            # A typed 400 still publishes, so the request completes cleanly.
            ("complete", validation_case),
        ]
        for reason, run_case in cases:
            with self.subTest(terminal_reason=reason):
                self.setUp()
                try:
                    with self.tracked_ledgers() as ledgers:
                        with self.tracked_sessions() as sessions:
                            wire, observation = run_case()
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"], reason
                    )
                    self.assert_no_partial_response(wire, observation)
                    # One request ledger; no hidden second or nested local one.
                    self.assertEqual(len(ledgers), 1)
                    self.assert_sessions_closed(sessions, expected=1)
                finally:
                    self.tearDown()

    # ------------------------------------------------------------------
    # MUST-1: leaf absence is never inferred from unavailable parent authority
    # ------------------------------------------------------------------

    def assert_typed_authority_change(self, wire, observation, reason, target=None):
        """One complete typed 400 names a changed pinned authority record."""
        self.assertTrue(wire, "the route closed the connection with no reply")
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        message = self.body_of(wire)["error"]
        self.assertTrue(message.startswith(AUTHORITY_CHANGED_PREFIX), message)
        self.assertIn(reason, message)
        if target is not None:
            self.assertIn(str(target), message)
        self.assert_no_partial_response(wire, observation)
        return message

    def assert_stale_downgrade_is_at_stake(self):
        """The unmodified fixture publishes one stale status from the branch head."""
        wire, observation = self.exchange()
        statuses = [entry["status"] for entry in self.assert_ok(wire)["merge_requests"]]
        self.assertIn("stale", statuses, statuses)
        self.assert_no_ownership_debt(observation)
        return statuses

    @contextlib.contextmanager
    def failing_directory_open(self, victim, error):
        """Fail exactly one no-follow parent-directory capability open."""
        globals_ = self.route_globals
        original = globals_["_open_directory_nofollow"]
        target = os.path.abspath(victim)
        state = {"failures": 0}

        def opening(path):
            if os.path.abspath(path) == target:
                state["failures"] += 1
                raise error
            return original(path)

        with self.patched(globals_, "_open_directory_nofollow", opening):
            yield state

    @contextlib.contextmanager
    def failing_directory_metadata(self, victim, error):
        """Fail the pinned parent directory's metadata capture, not its open."""
        globals_ = self.route_globals
        original = globals_["_fd_identity"]
        victim_inode = victim.stat().st_ino
        state = {"failures": 0}

        def identity(fd):
            if os.fstat(fd).st_ino == victim_inode:
                state["failures"] += 1
                raise error
            return original(fd)

        with self.patched(globals_, "_fd_identity", identity):
            yield state

    def _replace_directory_with_symlink(self, victim):
        """Point one authority directory at an identical sibling directory."""
        sibling = victim.with_name("shadow-" + victim.name)
        shutil.copytree(victim, sibling)
        shutil.rmtree(victim)
        victim.symlink_to(sibling.name, target_is_directory=True)
        return sibling

    def test_unavailable_parent_authority_is_never_leaf_absence(self):
        """A missing, replaced, or unreadable parent fails closed, never open.

        The consulted branch record's parent directory decides whether a stale
        merge request is downgraded, so treating a parent-capability failure as
        leaf absence would publish an authoritative 200 with a wrong status.
        """
        baseline_statuses = self.assert_stale_downgrade_is_at_stake()

        def missing_parent(branches):
            shutil.move(str(branches), str(branches.with_name("moved-branches")))

        def parent_is_a_symlink(branches):
            self._replace_directory_with_symlink(branches)

        def parent_is_a_regular_file(branches):
            shutil.rmtree(branches)
            branches.write_text("{}\n", encoding="utf-8")

        def denied_parent(branches):
            branches.chmod(0o000)

        cases = {
            "missing parent": (missing_parent, None),
            "symlinked parent": (parent_is_a_symlink, None),
            "parent replaced by a file": (parent_is_a_regular_file, None),
            "denied parent": (denied_parent, "root ignores directory permissions"),
        }
        for label, (mutate, skip_as_root) in cases.items():
            with self.subTest(parent=label):
                if skip_as_root is not None and os.geteuid() == 0:
                    self.skipTest(skip_as_root)
                self.setUp()
                try:
                    branches = self.dirs["branches"]
                    mutate(branches)
                    wire, observation = self.exchange()
                    self.assert_typed_authority_failure(
                        wire, observation, target=branches
                    )
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"], "complete"
                    )
                finally:
                    if self.dirs["branches"].is_dir():
                        self.dirs["branches"].chmod(0o700)
                    self.tearDown()

        # An injected parent-open I/O failure and an injected parent metadata
        # failure are the same typed family, not a silent absence.
        for label, injector in (
            (
                "parent open I/O failure",
                lambda branches: self.failing_directory_open(
                    branches, OSError(errno.EIO, "Input/output error", str(branches))
                ),
            ),
            (
                "parent metadata failure",
                lambda branches: self.failing_directory_metadata(
                    branches, OSError(errno.EIO, "Input/output error", str(branches))
                ),
            ),
            (
                "parent read timeout",
                lambda branches: self.failing_directory_open(
                    branches,
                    TimeoutError(errno.ETIMEDOUT, "Connection timed out", str(branches)),
                ),
            ),
        ):
            with self.subTest(injected=label):
                self.setUp()
                try:
                    branches = self.dirs["branches"]
                    with injector(branches) as state:
                        wire, observation = self.exchange()
                    self.assertGreater(state["failures"], 0, "no fault was injected")
                    self.assert_typed_authority_failure(
                        wire, observation, target=branches
                    )
                finally:
                    self.tearDown()

        # Non-vacuity: with the parent available the same fixture still publishes
        # the stale downgrade, so every failure above changed a real outcome.
        self.setUp()
        try:
            self.assertEqual(self.assert_stale_downgrade_is_at_stake(), baseline_statuses)
        finally:
            self.tearDown()

    def test_absent_authority_fence_reproves_its_parent_identity_and_state(self):
        """An unreadable or replaced parent is never continued leaf absence."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]

        def at_the_authority_fence(install):
            original = Operation.finalize_authority
            state = {"installed": False, "stack": None}

            def fence(owner, *args, **kwargs):
                if not state["installed"]:
                    state["installed"] = True
                    state["stack"] = contextlib.ExitStack()
                    state["stack"].enter_context(install())
                return original(owner, *args, **kwargs)

            @contextlib.contextmanager
            def installed():
                with self.patched(Operation, "finalize_authority", fence):
                    try:
                        yield state
                    finally:
                        if state["stack"] is not None:
                            state["stack"].close()

            return installed()

        # The consulted branch record is captured as absent, then its pinned
        # parent becomes unreadable before the final fence.
        self.branch_path(TARGET_BRANCH).unlink()
        branches = self.dirs["branches"]
        with at_the_authority_fence(
            lambda: self.failing_directory_open(
                branches, OSError(errno.EIO, "Input/output error", str(branches))
            )
        ) as state:
            wire, observation = self.exchange()
        self.assertTrue(state["installed"], "the authority fence never ran")
        self.assert_typed_authority_failure(wire, observation, target=branches)

        # A pinned parent replaced by a different directory is rejected even
        # though the leaf is still absent in the replacement.
        self.setUp()
        self.branch_path(TARGET_BRANCH).unlink()
        branches = self.dirs["branches"]
        original = Operation.finalize_authority
        replaced = {"done": False}

        def replacing_fence(owner, *args, **kwargs):
            if not replaced["done"]:
                replaced["done"] = True
                shutil.move(str(branches), str(branches.with_name("old-branches")))
                branches.mkdir()
            return original(owner, *args, **kwargs)

        with self.patched(Operation, "finalize_authority", replacing_fence):
            wire, observation = self.exchange()
        self.assertTrue(replaced["done"])
        self.assert_typed_authority_change(
            wire, observation, "parent_changed", self.branch_path(TARGET_BRANCH)
        )

    # ------------------------------------------------------------------
    # MUST-2: failure origin is classified at the boundary that raised it
    # ------------------------------------------------------------------

    def failing_transport(self, error, *, on):
        """Configure one handler whose real socket read or write fails."""

        def configure(handler):
            original_setup = handler.setup

            def setup(owner):
                original_setup(owner)
                if {"read", "readline"} & set(on):
                    owner.rfile = _FailingSocketFile(owner.rfile, error, on=on)
                if {"write", "flush"} & set(on):
                    owner.wfile = _FailingSocketFile(owner.wfile, error, on=on)

            handler.setup = setup

        return configure

    def test_error_origin_is_classified_at_the_boundary_that_raised_it(self):
        """Filesystem and transport failures are told apart by origin, not class.

        ``TimeoutError`` and ``ConnectionError`` are raised both by a client
        socket and by a remote filesystem, so the exception class alone cannot
        decide whether to publish a typed authority failure or abandon the
        connection.
        """
        globals_ = self.route_globals
        victim = "/storage/authority.json"
        filesystem_faults = {
            "filesystem timeout": TimeoutError(
                errno.ETIMEDOUT, "Connection timed out", victim
            ),
            "filesystem connection reset": ConnectionResetError(
                errno.ECONNRESET, "Connection reset by peer", victim
            ),
            "generic filesystem error": OSError(
                errno.EIO, "Input/output error", victim
            ),
        }
        routes = {
            "handle_http_request_list": MR_LIST_REQUEST,
            "handle_http_request_merge": post_request(
                "/v1/projects/org/app/merge-requests"
            ),
            "handle_http_upload": post_request(
                "/v1/projects/org/app/branches/main/upload"
            ),
        }
        for route, request in routes.items():
            for label, error in filesystem_faults.items():
                with self.subTest(route=route, fault=label):

                    def raising(*args, _error=error, **kwargs):
                        raise _error

                    with self.patched(globals_, route, raising):
                        wire, observation = self.exchange(request=request)
                    message = self.assert_typed_authority_failure(
                        wire, observation, target=victim
                    )
                    self.assertIn(error.strerror, message)
                    self.assertEqual(
                        self.namespace["os_error_origin"](error), "filesystem"
                    )

        # A real socket write or read failure still abandons the connection and
        # never fabricates a second response.
        transport_faults = {
            "socket write reset": (
                ConnectionResetError(errno.ECONNRESET, "Connection reset by peer"),
                ("write",),
            ),
            "socket write broken pipe": (
                BrokenPipeError(errno.EPIPE, "Broken pipe"),
                ("write",),
            ),
            "socket read timeout": (
                TimeoutError(errno.ETIMEDOUT, "Connection timed out"),
                ("readline",),
            ),
            "socket read disconnect": (
                ConnectionResetError(errno.ECONNRESET, "Connection reset by peer"),
                ("read", "readline"),
            ),
        }
        for label, (error, on) in transport_faults.items():
            with self.subTest(transport=label):
                with self.abandoned_connection_expected():
                    wire, observation = self.exchange(
                        configure_handler=self.failing_transport(error, on=on)
                    )
                self.assertEqual(wire, b"")
                self.assertEqual(
                    self.namespace["os_error_origin"](error), "transport"
                )
                self.assert_no_ownership_debt(observation)
                self.assertNotEqual(
                    observation["snapshot"]["terminal_reason"], "cleanup_incomplete"
                )

        # Request cancellation stays a typed 503 and memory exhaustion stays
        # outside the filesystem arm entirely.
        wire, observation = self.exchange(
            self.profile(deadline_ms=10), clock=_ExpiringClock(1)
        )
        self.assertEqual(
            self.status_line(wire), b"HTTP/1.0 503 Service Unavailable"
        )
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "deadline_exceeded"
        )

        def exhausting(*args, **kwargs):
            raise MemoryError("not a filesystem fault")

        with self.abandoned_connection_expected():
            with self.patched(
                globals_, "handle_http_request_list", exhausting
            ):
                wire, observation = self.exchange()
        self.assertEqual(wire, b"")
        self.assert_no_ownership_debt(observation)

    def test_primary_diagnostic_survives_a_cleanup_failure(self):
        """A cleanup fault is recorded, never swallowed, and never wins."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        original_close = Operation.close
        recorded = []

        def failing_close(owner):
            recorded.append(owner)
            if len(recorded) == 1:
                error = RuntimeError("injected primary-path cleanup failure")
                owner._cleanup_faults.append(error)
                return error
            return original_close(owner)

        # A validation failure is the primary diagnostic; the injected cleanup
        # failure is retried and recorded without replacing it.
        self.object_path(self.blob_object_id()).unlink()
        with self.patched(Operation, "close", failing_close):
            wire, observation = self.exchange()
        self.assertEqual(len(recorded), 2, "cleanup was not retried")
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assertIn("object not found while bundling", self.body_of(wire)["error"])
        self.assert_no_partial_response(wire, observation)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")

    # ------------------------------------------------------------------
    # MUST-3: cleanup keeps retry ownership until its release succeeds
    # ------------------------------------------------------------------

    @contextlib.contextmanager
    def failing_cleanup_release(self, ordinal, *, times=1):
        """Fail one exact ledger release performed by the operation's cleanup."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        Ledger = self.route_globals["DecisionWorkLedger"]
        original_close = Operation.close
        original_release = Ledger.release
        state = {"depth": 0, "releases": 0, "failures": 0, "attempts": 0}

        def close(owner):
            state["depth"] += 1
            state["attempts"] += 1
            try:
                return original_close(owner)
            finally:
                state["depth"] -= 1

        def release(owner, dimension, amount):
            if state["depth"]:
                state["releases"] += 1
                if (
                    ordinal is not None
                    and state["releases"] == ordinal
                    and state["failures"] < times
                ):
                    state["failures"] += 1
                    raise RuntimeError(
                        f"injected cleanup release failure for {dimension}"
                    )
            return original_release(owner, dimension, amount)

        with self.patched(Operation, "close", close):
            with self.patched(Ledger, "release", release):
                yield state

    def test_cleanup_retries_every_owner_release_without_losing_ownership(self):
        """Each cleanup release can fail once and still reach zero debt."""
        baseline_wire, baseline = self.exchange()
        baseline_body = self.assert_ok(baseline_wire)

        with self.failing_cleanup_release(None) as survey:
            wire, observation = self.exchange()
        self.assert_ok(wire)
        release_count = survey["releases"]
        self.assertGreater(release_count, 0, "cleanup released no ownership")
        self.assertEqual(survey["attempts"], 1, "cleanup retried without a fault")

        for ordinal in range(1, release_count + 1):
            with self.subTest(cleanup_release=ordinal):
                self.setUp()
                try:
                    with self.failing_cleanup_release(ordinal) as state:
                        wire, observation = self.exchange()
                    self.assertEqual(state["failures"], 1, "no fault was injected")
                    self.assertEqual(state["attempts"], 2, "cleanup was not retried")
                    self.assertEqual(self.assert_ok(wire), baseline_body)
                    self.assert_no_ownership_debt(observation)
                    self.assertEqual(
                        observation["snapshot"]["terminal_reason"], "complete"
                    )
                finally:
                    self.tearDown()

        # A cleanup session close that fails once is retried, not swallowed, and
        # the session is never removed from the operation before it closes.
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        Session = self.route_globals["SealedCommitValidationSession"]
        original_operation_close = Operation.close
        original_session_close = Session.close
        cleanup = {"depth": 0, "attempts": 0}
        session_calls = []

        def counted_operation_close(owner):
            cleanup["depth"] += 1
            cleanup["attempts"] += 1
            try:
                return original_operation_close(owner)
            finally:
                cleanup["depth"] -= 1

        def failing_session_close(owner, terminal_reason=None):
            session_calls.append(cleanup["depth"])
            if cleanup["depth"] and session_calls.count(1) == 1:
                raise RuntimeError("injected session close failure")
            return original_session_close(owner, terminal_reason)

        self.setUp()
        try:
            with self.patched(Operation, "close", counted_operation_close):
                with self.patched(Session, "close", failing_session_close):
                    wire, observation = self.exchange()
            self.assertGreaterEqual(
                session_calls.count(1), 2, "the cleanup close was not retried"
            )
            self.assertEqual(cleanup["attempts"], 2)
            self.assertEqual(self.assert_ok(wire), baseline_body)
            self.assert_no_ownership_debt(observation)
            self.assertEqual(
                observation["snapshot"]["terminal_reason"], "complete"
            )
        finally:
            self.tearDown()

        # A prepared-response close that fails once is retried after the exact
        # bytes were published, so no accounting is lost and none is doubled.
        Owner = self.route_globals["_PreparedHTTPResponseOwner"]
        original_owner_close = Owner.close
        owner_calls = []

        def failing_owner_close(owner):
            owner_calls.append(owner)
            if len(owner_calls) == 1:
                raise RuntimeError("injected prepared response close failure")
            return original_owner_close(owner)

        self.setUp()
        try:
            with self.patched(Owner, "close", failing_owner_close):
                wire, observation = self.exchange()
            self.assertGreaterEqual(len(owner_calls), 2, "close was not retried")
            self.assertEqual(self.assert_ok(wire), baseline_body)
            self.assert_no_ownership_debt(observation)
            self.assertEqual(
                observation["snapshot"]["terminal_reason"], "complete"
            )
        finally:
            self.tearDown()

    def test_unrecoverable_cleanup_never_publishes_hidden_current_work(self):
        """A success cannot publish while any route ownership survives."""
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        Ledger = self.route_globals["DecisionWorkLedger"]
        original_close = Operation.close
        original_release = Ledger.release
        state = {"depth": 0, "failures": 0}

        def close(owner):
            state["depth"] += 1
            try:
                return original_close(owner)
            finally:
                state["depth"] -= 1

        def release(owner, dimension, amount):
            if state["depth"]:
                state["failures"] += 1
                raise RuntimeError("injected permanent cleanup release failure")
            return original_release(owner, dimension, amount)

        with self.abandoned_connection_expected():
            with self.patched(Operation, "close", close):
                with self.patched(Ledger, "release", release):
                    wire, observation = self.exchange()
        self.assertGreater(state["failures"], 0, "no fault was injected")
        # No 200 with residual ownership: the request publishes the cleanup
        # diagnostic instead, and the ledger names the exact terminal reason.
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        self.assertNotIn(b'"merge_requests"', wire)
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )
        self.assertTrue(
            any(observation["snapshot"]["current"].values()),
            "the injected fault released everything after all",
        )

    # ------------------------------------------------------------------
    # MUST-4: terminal metadata stays bound through the read and the digest
    # ------------------------------------------------------------------

    def _alias_path(self, victim, prefix):
        """One hardlink alias outside every scanned authority directory."""
        return self.tmp / f"{prefix}-{victim.name}"

    def _authority_mutations(self):
        """Deterministic authority changes and the exact typed reason each gives.

        ``detectable_after_read`` marks the changes an already-open descriptor can
        still observe.  A same-length content swap or a path relink after the
        fence has read and hashed its owned bytes is *not* observable on that
        descriptor: those pinned bytes are exactly what the response projects, so
        the documented outcome there is the pinned snapshot, not a rejection.
        """

        def change_mode(path):
            path.chmod(
                0o644
                if stat_module.S_IMODE(path.stat().st_mode) != 0o644
                else 0o600
            )

        def add_hardlink(path):
            os.link(path, self._alias_path(path, "added"))

        def prepare_hardlink(path):
            os.link(path, self._alias_path(path, "removed"))

        def remove_hardlink(path):
            self._alias_path(path, "removed").unlink()

        def replace_with_symlink(path):
            self._symlink_over(path)

        def change_size(path):
            with path.open("ab") as handle:
                handle.write(b" ")

        def change_identity(path):
            raw = path.read_bytes()
            path.unlink()
            path.write_bytes(raw)

        def change_content(path):
            self._same_length_digit_swap(path, path.stat())

        return {
            "mode": (None, change_mode, ("mode_changed",), True),
            "hardlink added": (None, add_hardlink, ("link_count_changed",), True),
            "hardlink removed": (
                prepare_hardlink,
                remove_hardlink,
                ("link_count_changed",),
                True,
            ),
            # A replaced leaf is refused by the no-follow open before it is
            # reopened, and by the pinned link count once it is already open.
            "symlink replacement": (
                None,
                replace_with_symlink,
                ("unreadable", "link_count_changed"),
                True,
            ),
            "size": (None, change_size, ("size_changed",), True),
            "identity": (
                None,
                change_identity,
                ("identity_changed", "link_count_changed"),
                True,
            ),
            "content": (None, change_content, ("digest_changed",), False),
        }

    def _mutate_inside_the_final_fence(self, victim, mutate, boundary):
        """Run the live route, mutating ``victim`` at one exact fence boundary.

        Every hook is scoped to the victim's own final authority revalidation, so
        an earlier capture of an unrelated record can never absorb the change.
        """
        globals_ = self.route_globals
        original_revalidate = globals_["revalidate_json_authority_capture"]
        state = {"mutated": False, "active": False}

        def once():
            if state["active"] and not state["mutated"]:
                state["mutated"] = True
                mutate(victim)

        def revalidate(capture, ledger, poll=None):
            active = str(capture.get("path")) == str(victim)
            previous = state["active"]
            state["active"] = active or previous
            if boundary == "before open" and active:
                once()
            try:
                return original_revalidate(capture, ledger, poll)
            finally:
                state["active"] = previous

        with contextlib.ExitStack() as stack:
            stack.enter_context(
                self.patched(
                    globals_, "revalidate_json_authority_capture", revalidate
                )
            )
            if boundary == "during read":
                original_read = os.read

                def read(*args, **kwargs):
                    data = original_read(*args, **kwargs)
                    once()
                    return data

                stack.enter_context(self.patched(os, "read", read))
            elif boundary == "during digest":
                original_digest = globals_["_sha256_hexdigest_polling"]

                def digest(data, poll=None, **kwargs):
                    once()
                    return original_digest(data, poll, **kwargs)

                stack.enter_context(
                    self.patched(globals_, "_sha256_hexdigest_polling", digest)
                )
            elif boundary != "before open":
                raise AssertionError(f"unknown boundary {boundary}")
            wire, observation = self.exchange()
        return wire, observation, state

    def test_terminal_metadata_is_reproved_through_the_read_and_digest(self):
        """No mode or link change after the last ``fstat`` reaches publication.

        The reviewed candidate reproved only device, inode, and size after the
        final read, so a mode change or an added hardlink during the interruptible
        digest was published as a valid 200.
        """
        baseline_wire, _ = self.exchange()
        baseline_body = self.assert_ok(baseline_wire)
        mutations = self._authority_mutations()
        for boundary in ("before open", "during read", "during digest"):
            for label, (
                prepare,
                mutate,
                reasons,
                detectable_after_read,
            ) in mutations.items():
                detected = boundary == "before open" or detectable_after_read
                with self.subTest(boundary=boundary, change=label):
                    self.setUp()
                    try:
                        victim = self.branch_path(TARGET_BRANCH)
                        if prepare is not None:
                            prepare(victim)
                        wire, observation, state = (
                            self._mutate_inside_the_final_fence(
                                victim, mutate, boundary
                            )
                        )
                        self.assertTrue(
                            state["mutated"], f"no {label} change was applied"
                        )
                        if not detected:
                            # The fence already owns and hashed these exact bytes,
                            # so the pinned snapshot is published unchanged.
                            self.assertEqual(self.assert_ok(wire), baseline_body)
                            self.assert_no_ownership_debt(observation)
                            continue
                        self.assertEqual(
                            self.status_line(wire),
                            b"HTTP/1.0 400 Bad Request",
                            f"a {label} change at {boundary} was published",
                        )
                        message = self.body_of(wire)["error"]
                        self.assertTrue(
                            message.startswith(AUTHORITY_CHANGED_PREFIX)
                            or message.startswith(AUTHORITY_FAILURE_PREFIX),
                            message,
                        )
                        self.assertTrue(
                            any(reason in message for reason in reasons),
                            f"{message} names none of {reasons}",
                        )
                        self.assert_no_partial_response(wire, observation)
                    finally:
                        self.tearDown()

    def test_stable_pre_existing_links_stay_compatible_authority(self):
        """A stable multi-link authority file is compatible, not rejected.

        The exact link count is captured and reproved, so pre-existing aliases on
        a merge-request or branch record keep working while any change to the
        count fails closed.  The project descriptor keeps the legacy baseline's
        own one-link contract, which ``_read_regular_bounded_at`` has enforced for
        every bounded authority file since ``7327e2a``; the repaired route adds no
        second, incidental rejection of its own.
        """
        baseline_wire, _ = self.exchange()
        baseline_body = self.assert_ok(baseline_wire)
        compatible = {
            "merge request record": lambda: self.merge_request_path(
                sorted(self.merge_request_ids)[0]
            ),
            "consulted branch record": lambda: self.branch_path(TARGET_BRANCH),
        }
        for label, resolve in compatible.items():
            with self.subTest(authority=label):
                self.setUp()
                try:
                    victim = resolve()
                    os.link(victim, self._alias_path(victim, "stable"))
                    self.assertEqual(victim.stat().st_nlink, 2)
                    wire, observation = self.exchange()
                    self.assertEqual(self.assert_ok(wire), baseline_body)
                    self.assert_no_ownership_debt(observation)
                finally:
                    self.tearDown()

        # The descriptor's one-link rule is the legacy baseline's, reached through
        # the shared project gate, and the accounted route reproduces it exactly.
        self.setUp()
        try:
            descriptor = self.descriptor_path()
            os.link(descriptor, self._alias_path(descriptor, "stable"))
            wire, observation = self.exchange()
            self.assertEqual(
                self.status_line(wire), b"HTTP/1.0 400 Bad Request"
            )
            self.assertEqual(
                self.body_of(wire)["error"],
                "bounded authority file is invalid: remote_project.json",
            )
            with self.assertRaises(self.namespace["CFError"]) as legacy:
                self.legacy_request_list(self.storage)
            self.assertEqual(
                str(legacy.exception),
                "bounded authority file is invalid: remote_project.json",
                "the legacy baseline did not reject a multi-link descriptor",
            )
            self.assert_no_partial_response(wire, observation)
        finally:
            self.tearDown()

    # ------------------------------------------------------------------
    # SHOULD-1: the terminal fence immediately precedes the first byte
    # ------------------------------------------------------------------

    # Landmarks the production publication order must visit, in order.
    PUBLICATION_LANDMARKS = (
        "encode",
        "authority fence",
        "session fence",
        "project fence",
        "cleanup",
        "publishable proof",
        "take",
        "publish",
    )
    # Fallible teardown operations one MR-list request performs after its last
    # authority-dependent fence: the shared lock's unlock plus its two descriptor
    # closes, and the project gate's two descriptor closes.
    POST_FENCE_TEARDOWN_OPERATIONS = 5

    @contextlib.contextmanager
    def recorded_publication_order(self, *, fences=None):
        """Record the publication landmarks and every fallible teardown call.

        A named-method trace alone cannot see the gate/lock teardown, so
        ``fcntl.flock`` and ``os.close`` are observed too.  They are recorded only
        once the project fence has run, which bounds the trace to exactly the
        window between the last authority fence and the first response byte.
        """
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        Slot = globals_["_HTTPReadResponseSlot"]
        Gate = globals_["_RemoteProjectGateFence"]
        Owner = globals_["_PreparedHTTPResponseOwner"]
        fcntl_module = globals_["fcntl"]
        order = []
        armed = {"value": False}

        def landmark(name, target, attribute):
            original = getattr(target, attribute)

            def recording(owner, *args, **kwargs):
                order.append(name)
                if name == "project fence":
                    armed["value"] = True
                    if fences is not None:
                        fences.append(owner)
                return original(owner, *args, **kwargs)

            return self.patched(target, attribute, recording)

        original_flock = fcntl_module.flock
        original_close = os.close

        def flock(fd, operation, *args):
            if armed["value"]:
                order.append(f"fallible flock({operation})")
            return original_flock(fd, operation, *args)

        def close(fd):
            if armed["value"]:
                order.append("fallible os.close")
            return original_close(fd)

        with contextlib.ExitStack() as stack:
            stack.enter_context(landmark("encode", Slot, "prepare"))
            stack.enter_context(
                landmark("authority fence", Operation, "finalize_authority")
            )
            stack.enter_context(
                landmark("session fence", Operation, "finalize_sessions")
            )
            stack.enter_context(landmark("project fence", Gate, "fence"))
            stack.enter_context(landmark("cleanup", Operation, "close"))
            stack.enter_context(
                landmark("publishable proof", Slot, "require_publishable_state")
            )
            stack.enter_context(landmark("take", Slot, "take"))
            stack.enter_context(landmark("publish", Owner, "publish"))
            stack.enter_context(self.patched(fcntl_module, "flock", flock))
            stack.enter_context(self.patched(os, "close", close))
            yield order

    def test_the_terminal_fence_immediately_precedes_the_first_response_byte(self):
        """The final proof runs after every teardown, and publication is last.

        Every authority mutation before the terminal fence fails closed, the
        pinned lock and gate descriptors are unlocked and closed *before* the
        final handler-owned proof, and the only work left after that proof is
        transferring already-frozen bytes onto the wire.
        """
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        Slot = globals_["_HTTPReadResponseSlot"]
        fences = []
        with self.recorded_publication_order(fences=fences) as order:
            wire, observation = self.exchange()
        body = self.assert_ok(wire)
        self.assertEqual(
            [event for event in order if not event.startswith("fallible ")],
            list(self.PUBLICATION_LANDMARKS),
        )
        proof = order.index("publishable proof")
        teardown = [
            event
            for event in order[order.index("cleanup") + 1 : proof]
            if event.startswith("fallible ")
        ]
        fallible_after_proof = [
            event for event in order[proof + 1 :] if event.startswith("fallible ")
        ]
        # The shared lock's unlock and all four pinned descriptor closes happen
        # strictly before the final proof, which is what makes the proof effective.
        self.assertEqual(
            len(teardown), self.POST_FENCE_TEARDOWN_OPERATIONS, order
        )
        self.assertTrue(
            any(event.startswith("fallible flock") for event in teardown), order
        )
        self.assertEqual(
            fallible_after_proof,
            [],
            "a fallible operation runs between the final proof and the wire",
        )
        # After the proof only the owner transfer and the publication remain.
        self.assertEqual(order[proof + 1 :], ["take", "publish"])
        # The project gate is fenced exactly once: the route's terminal fence is
        # not repeated after the publication point has been reached.
        self.assertEqual(len(fences), 1)
        self.assertEqual(fences[0].fence_count, 1)
        self.assert_no_ownership_debt(observation)

        # Deliberate regression: restoring the proof to the inside of the pinned
        # gate makes the same oracle observe the teardown after the proof.
        original_collect = globals_["collect_http_merge_request_list_payload"]

        def proving_inside_the_gate(
            storage_root, project_root, dirs, ledger, deadline=None, **kwargs
        ):
            result = original_collect(
                storage_root, project_root, dirs, ledger, deadline, **kwargs
            )
            slot = kwargs.get("response_slot")
            if slot is not None and slot.is_prepared:
                slot.require_publishable_state()
            return result

        with self.patched(
            globals_,
            "collect_http_merge_request_list_payload",
            proving_inside_the_gate,
        ):
            with self.recorded_publication_order() as regressed:
                regressed_wire, regressed_observation = self.exchange()
        self.assertEqual(self.assert_ok(regressed_wire), body)
        first_proof = regressed.index("publishable proof")
        self.assertEqual(
            len(
                [
                    event
                    for event in regressed[first_proof + 1 :]
                    if event.startswith("fallible ")
                ]
            ),
            self.POST_FENCE_TEARDOWN_OPERATIONS,
            "the regressed ordering was not observed by the oracle",
        )
        self.assert_no_ownership_debt(regressed_observation)

        # A mutation after every fence cannot reach the frozen projection.
        original_state = Slot.require_publishable_state
        mutated = {"done": False}
        victim = self.merge_request_path(sorted(self.merge_request_ids)[0])

        def mutating_state(owner):
            if not mutated["done"]:
                mutated["done"] = True
                self._same_length_digit_swap(victim, victim.stat())
            return original_state(owner)

        with self.patched(Slot, "require_publishable_state", mutating_state):
            late_wire, late = self.exchange()
        self.assertTrue(mutated["done"])
        self.assertEqual(self.assert_ok(late_wire), body)
        self.assert_no_ownership_debt(late)

        # A mutation before the terminal project fence still fails closed.
        self.setUp()
        original_sessions = Operation.finalize_sessions
        victim = self.merge_request_path(sorted(self.merge_request_ids)[0])
        fenced = {"done": False}

        def mutating_sessions(owner, *args, **kwargs):
            result = original_sessions(owner, *args, **kwargs)
            if not fenced["done"]:
                fenced["done"] = True
                shutil.move(
                    str(self.descriptor_path()),
                    str(self.descriptor_path().with_suffix(".moved")),
                )
                shutil.copy(
                    str(self.descriptor_path().with_suffix(".moved")),
                    str(self.descriptor_path()),
                )
            return result

        with self.patched(Operation, "finalize_sessions", mutating_sessions):
            wire, observation = self.exchange()
        self.assertTrue(fenced["done"])
        self.assertEqual(self.status_line(wire), b"HTTP/1.0 400 Bad Request")
        self.assertIn(
            "remote project descriptor changed", self.body_of(wire)["error"]
        )
        self.assert_no_partial_response(wire, observation)

    # ------------------------------------------------------------------
    # the post-teardown window and the publishable guard
    # ------------------------------------------------------------------

    @contextlib.contextmanager
    def failing_gate_unlock(self, error):
        """Fail the shared project gate's real unlock exactly once."""
        fcntl_module = self.route_globals["fcntl"]
        original = fcntl_module.flock
        state = {"failures": 0}

        def flock(fd, operation, *args):
            if operation == fcntl_module.LOCK_UN and not state["failures"]:
                state["failures"] += 1
                original(fd, operation, *args)
                raise error
            return original(fd, operation, *args)

        with self.patched(fcntl_module, "flock", flock):
            yield state

    def test_post_teardown_filesystem_failure_replaces_the_prepared_success(self):
        """A real unlock-boundary ``EIO`` is published, never silently dropped.

        The gate's unlock and descriptor closes run after the last authority
        fence.  A genuine filesystem fault there reaches ``do_GET``'s typed arm,
        and the already-prepared ``200`` is disposed of instead of replacing that
        diagnostic.
        """
        victim = str(self.project_root / "locks" / "project-gate.lock")
        error = OSError(errno.EIO, "Input/output error", victim)
        logs = []
        with self.failing_gate_unlock(error) as state:
            wire, observation = self.exchange(
                configure_handler=self.captured_handler_logs(logs)
            )
        self.assertEqual(state["failures"], 1, "no unlock fault was injected")
        # The typed filesystem-authority diagnostic is what publishes, and the
        # prepared success never reaches the wire.
        message = self.assert_typed_authority_failure(wire, observation, target=victim)
        self.assertIn("Input/output error", message)
        self.assertNotIn(b'"merge_requests"', wire)
        self.assertEqual(
            self.namespace["os_error_origin"](error),
            "filesystem",
            "the unlock fault lost its filesystem origin",
        )
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")

        # Non-vacuity: without the injection the same request publishes the
        # prepared success, so the outcome above is caused by the fault alone.
        clean_wire, clean = self.exchange()
        self.assertEqual(len(self.assert_ok(clean_wire)["merge_requests"]), MR_COUNT)
        self.assert_no_ownership_debt(clean)

    def test_unpublishable_prepared_response_never_reaches_the_wire(self):
        """A proof failure publishes no success byte and names its terminal state.

        The publishable guard is the only transition that authorizes publication,
        so residual ownership at the proof point makes the prepared ``200``
        unpublishable rather than merely observed.
        """
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        original_sessions = Operation.finalize_sessions
        leaked = {"bytes": 0}

        def leaking_sessions(owner, *args, **kwargs):
            result = original_sessions(owner, *args, **kwargs)
            if not leaked["bytes"]:
                # One retained owner that the operation does not know about, so
                # cleanup cannot release it and the proof must refuse to publish.
                owner.ledger.acquire("retained_bytes", 4096, "record_read")
                leaked["bytes"] = 4096
            return result

        logs = []
        with self.patched(Operation, "finalize_sessions", leaking_sessions):
            wire, observation = self.exchange(
                configure_handler=self.captured_handler_logs(logs)
            )
        self.assertEqual(leaked["bytes"], 4096, "no residual owner was injected")
        # No success byte and no projection reached the wire.
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        self.assertNotIn(b'"merge_requests"', wire)
        # The typed diagnostic stays observable even when the terminal
        # control-plane state cannot publish it.
        self.assertTrue(
            any(
                "prepared read response cannot publish with unaccounted current work"
                in entry
                for entry in logs
            ),
            logs,
        )
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )
        # The prepared owner was released exactly once: only the injected residual
        # survives, and no permit is left open.
        self.assertEqual(
            observation["snapshot"]["current"].get("retained_bytes", 0), 4096
        )
        self.assertEqual(
            observation["snapshot"]["current"].get("temporary_bytes", 0), 0
        )
        self.assertEqual(observation["open_permit_count"], 0)

        # Non-vacuity: the guard is the reason.  Silencing only the proof restores
        # the rejected behaviour — a 200 published with live debt.
        Slot = globals_["_HTTPReadResponseSlot"]
        original_take = Slot.take

        def unproved_take(owner):
            owner.state = Slot.PROVED
            return original_take(owner)

        leaked["bytes"] = 0
        with self.patched(Operation, "finalize_sessions", leaking_sessions):
            with self.patched(
                Slot, "require_publishable_state", lambda owner: None
            ):
                with self.patched(Slot, "take", unproved_take):
                    inert_wire, inert = self.exchange()
        self.assertEqual(leaked["bytes"], 4096)
        self.assertEqual(self.status_line(inert_wire), b"HTTP/1.0 200 OK")
        self.assertTrue(
            any(inert["snapshot"]["current"].values()),
            "the residual owner did not survive the silenced guard",
        )

    def test_prepared_disposal_failure_publishes_nothing(self):
        """A disposal failure wins over publication and is recorded, not hidden.

        When the prepared owner cannot be released, the ledger state is unknown,
        so no replacement response may follow it onto the wire.
        """
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        Owner = globals_["_PreparedHTTPResponseOwner"]
        original_sessions = Operation.finalize_sessions
        original_close = Owner.close
        raised = {"route": False, "disposal": 0}

        def failing_sessions(owner, *args, **kwargs):
            result = original_sessions(owner, *args, **kwargs)
            if not raised["route"]:
                raised["route"] = True
                raise self.namespace["CFError"]("injected post-encode route failure")
            return result

        def failing_close(owner):
            raised["disposal"] += 1
            raise RuntimeError("injected prepared response disposal failure")

        logs = []
        with self.abandoned_connection_expected():
            with self.patched(Operation, "finalize_sessions", failing_sessions):
                with self.patched(Owner, "close", failing_close):
                    wire, observation = self.exchange(
                        configure_handler=self.captured_handler_logs(logs)
                    )
        self.assertTrue(raised["route"])
        self.assertGreater(raised["disposal"], 0, "disposal was never attempted")
        # Deterministic precedence: nothing is published at all.
        self.assertEqual(wire, b"")
        self.assertTrue(
            any(
                "prepared response disposal failed while replacing it" in entry
                for entry in logs
            ),
            logs,
        )
        # The unreleased prepared owner is visible as residual work, never hidden
        # behind a published response.
        self.assertTrue(any(observation["snapshot"]["current"].values()))
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )

    def test_owner_over_release_fails_closed(self):
        """An exact release larger than the owner is a typed accounting error."""
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        CFError = self.namespace["CFError"]
        ledger = globals_["DecisionWorkLedger"](
            self.profile(), "http request", command_kind="http_request"
        )
        operation = Operation(self.storage, self.project_root, self.dirs, ledger)
        ledger.acquire("retained_bytes", 100, "record_read")
        operation._sessions_retained = 100

        error = operation._release_exact("retained_bytes", "_sessions_retained", 200)
        self.assertIsInstance(error, CFError)
        self.assertIn("decision-work owner over-release", str(error))
        self.assertIn("_sessions_retained", str(error))
        # Nothing was released and the fault is retained for the caller.
        self.assertEqual(operation._sessions_retained, 100)
        self.assertEqual(ledger.current.get("retained_bytes", 0), 100)
        self.assertIn(error, operation._cleanup_faults)

        # An exact release still works, and a zero release stays a no-op.
        self.assertIsNone(
            operation._release_exact("retained_bytes", "_sessions_retained", 0)
        )
        self.assertIsNone(
            operation._release_exact("retained_bytes", "_sessions_retained", 100)
        )
        self.assertEqual(operation._sessions_retained, 0)
        self.assertEqual(ledger.current.get("retained_bytes", 0), 0)

        # Non-vacuity: the rejected defensive form masks the same over-release.
        def silent_release_exact(owner, dimension, attribute, amount):
            owned = getattr(owner, attribute)
            if not amount or amount > owned:
                return None
            owner.ledger.release(dimension, amount)
            setattr(owner, attribute, owned - amount)
            return None

        operation._sessions_retained = 100
        ledger.acquire("retained_bytes", 100, "record_read")
        with self.patched(Operation, "_release_exact", silent_release_exact):
            self.assertIsNone(
                operation._release_exact(
                    "retained_bytes", "_sessions_retained", 200
                ),
                "the rejected form did not mask the over-release",
            )
        ledger.release("retained_bytes", 100)

    def test_recovered_cleanup_faults_reach_a_production_diagnostic(self):
        """A retried cleanup fault is consumed, never left as unread state."""
        globals_ = self.route_globals
        Operation = globals_["_HTTPMergeRequestListOperation"]
        Slot = globals_["_HTTPReadResponseSlot"]
        recorded = []
        original_report = Slot.report_cleanup_recovery

        def report(owner, diagnostic):
            recorded.append(diagnostic)
            return original_report(owner, diagnostic)

        logs = []
        with self.patched(Slot, "report_cleanup_recovery", report):
            with self.failing_cleanup_release(1) as state:
                wire, observation = self.exchange(
                    configure_handler=self.captured_handler_logs(logs)
                )
        self.assertEqual(state["failures"], 1)
        self.assertEqual(state["attempts"], 2)
        self.assert_ok(wire)
        self.assert_no_ownership_debt(observation)
        # The recovered fault reached the handler's own error log with its exact
        # attempt count and diagnostic text.
        self.assertEqual(len(recorded), 1, recorded)
        self.assertEqual(recorded[0]["attempts"], 2)
        self.assertTrue(recorded[0]["recovered"])
        self.assertEqual(recorded[0]["fault_count"], 1)
        self.assertTrue(
            any("injected cleanup release failure" in fault for fault in recorded[0]["faults"]),
            recorded,
        )
        self.assertTrue(
            any("recovered cleanup fault" in entry for entry in logs), logs
        )

        # A clean request produces no cleanup diagnostic and no log line at all.
        clean_recorded = []

        def clean_report(owner, diagnostic):
            clean_recorded.append(diagnostic)
            return original_report(owner, diagnostic)

        clean_logs = []
        with self.patched(Slot, "report_cleanup_recovery", clean_report):
            clean_wire, clean = self.exchange(
                configure_handler=self.captured_handler_logs(clean_logs)
            )
        self.assert_ok(clean_wire)
        self.assertEqual(clean_recorded, [])
        self.assertEqual(clean_logs, [])
        self.assert_no_ownership_debt(clean)

    # ------------------------------------------------------------------
    # COULD-1: the descriptor read protocol states its exact route-wide scope
    # ------------------------------------------------------------------

    @contextlib.contextmanager
    def recorded_descriptor_opens(self):
        """Record every physical descriptor open with its owner and phase."""
        opens = Counter()
        original = os.open

        def opening(path, *args, **kwargs):
            if path == "remote_project.json":
                opens[
                    (
                        sys._getframe(1).f_code.co_name,
                        sys._getframe(2).f_code.co_name,
                    )
                ] += 1
            return original(path, *args, **kwargs)

        with self.patched(os, "open", opening):
            yield opens

    # Every physical ``remote_project.json`` descriptor read one MR-list request
    # performs, keyed by ``(reader, owner)`` with its documented purpose:
    #   * the gate pins the descriptor identity and bytes before locking;
    #   * the gate reproves them once on acquisition and once at the terminal
    #     fence, which is the route's last authority observation;
    #   * ``stable_project_id`` charges and parses the one accounted copy the
    #     subject comparison uses, and memoizes the derived identity;
    #   * the final authority fence reproves those exact owned bytes.
    # No read may be eliminated without dropping one of those proofs.
    DESCRIPTOR_READ_PROTOCOL = {
        ("_read_regular_bounded_at", "remote_project_gate"): 1,
        ("_read_regular_bounded_at", "prove_pinned_project_state"): 2,
        ("read_json_accounted", "stable_project_id"): 1,
        ("revalidate_json_authority_capture", "finalize_authority"): 1,
    }

    def test_descriptor_read_protocol_is_exact_and_route_wide(self):
        """Every physical descriptor read has a named owner and exact count."""
        with self.forbidden_legacy_helpers():
            with self.recorded_descriptor_opens() as opens:
                wire, observation = self.exchange()
        self.assert_ok(wire)
        self.assertEqual(dict(opens), self.DESCRIPTOR_READ_PROTOCOL)
        descriptor = self.descriptor_path()
        # The one accounted read is scoped to ``stable_project_id``'s owned
        # bytes; the gate's and fence's reads are separate documented owners.
        with self.recorded_accounted_reads() as reads:
            accounted_wire, _ = self.exchange()
        self.assert_ok(accounted_wire)
        self.assertEqual(
            [path for path in reads if path == str(descriptor)],
            [str(descriptor)],
            "the accounted descriptor read is not exactly one",
        )
        counters = observation["snapshot"]["counters"]
        self.assertEqual(
            counters["snapshot_revalidation_bytes"]
            - sum(
                self.object_path(record["object_id"]).stat().st_size
                for record in self.all_records
            )
            - sum(path.stat().st_size for path in self.merge_request_paths())
            - self.branch_path(TARGET_BRANCH).stat().st_size,
            descriptor.stat().st_size,
        )
        self.assert_no_ownership_debt(observation)

        # Duplicating the scoped read fails the oracle for the intended reason.
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        original = Operation.stable_project_id

        def unmemoized(owner, project_root):
            owner._project_identities.clear()
            return original(owner, project_root)

        with self.patched(Operation, "stable_project_id", unmemoized):
            with self.recorded_descriptor_opens() as duplicated:
                wire, _ = self.exchange()
        self.assert_ok(wire)
        self.assertNotEqual(dict(duplicated), self.DESCRIPTOR_READ_PROTOCOL)
        self.assertGreater(
            duplicated[("read_json_accounted", "stable_project_id")],
            self.DESCRIPTOR_READ_PROTOCOL[
                ("read_json_accounted", "stable_project_id")
            ],
        )

        # Bypassing the accounted owner also fails the oracle: the legacy helper
        # reopens the descriptor outside the charged bytes.
        globals_ = self.route_globals
        legacy = globals_["remote_project_stable_id"]

        def bypassing(owner, project_root):
            return legacy(project_root)

        with self.patched(Operation, "stable_project_id", bypassing):
            with self.recorded_descriptor_opens() as bypassed:
                wire, _ = self.exchange()
        self.assert_ok(wire)
        self.assertNotEqual(dict(bypassed), self.DESCRIPTOR_READ_PROTOCOL)
        self.assertEqual(
            bypassed.get(("read_json_accounted", "stable_project_id"), 0), 0
        )

    # ------------------------------------------------------------------
    # COULD-2: every production poll site is individually required
    # ------------------------------------------------------------------

    @contextlib.contextmanager
    def measured_poll_sites(self, *, silenced=None, arm_at=None, clock=None):
        """Instrument every production poll site and admitted work quantum."""
        globals_ = self.route_globals
        Ledger = globals_["DecisionWorkLedger"]
        Deadline = globals_["HTTPRequestDeadline"]
        original_apply = Ledger.apply
        original_release = Ledger.release
        original_check = Deadline.check
        primitives = ("read", "open", "fstat", "stat", "scandir")
        originals = {name: getattr(os, name) for name in primitives}
        state = {
            "sites": Counter(),
            "phases": {},
            "work": 0,
            "maximum": 0,
            "lag": 0,
            "armed": False,
        }

        def admit():
            state["work"] += 1
            if state["armed"]:
                state["lag"] += 1

        def apply(owner, *args, **kwargs):
            admit()
            return original_apply(owner, *args, **kwargs)

        def release(owner, *args, **kwargs):
            admit()
            return original_release(owner, *args, **kwargs)

        def primitive(name):
            original = originals[name]

            def wrapper(*args, **kwargs):
                admit()
                return original(*args, **kwargs)

            return wrapper

        def check(owner, phase):
            site = resolve_poll_site()
            if arm_at is not None and site == arm_at and not state["armed"]:
                state["armed"] = True
                clock.expired = True
            if silenced is not None and site == silenced:
                return None
            state["sites"][site] += 1
            state["phases"][site] = phase
            state["maximum"] = max(state["maximum"], state["work"])
            state["work"] = 0
            return original_check(owner, phase)

        with contextlib.ExitStack() as stack:
            stack.enter_context(self.patched(Ledger, "apply", apply))
            stack.enter_context(self.patched(Ledger, "release", release))
            stack.enter_context(self.patched(Deadline, "check", check))
            for name in primitives:
                stack.enter_context(self.patched(os, name, primitive(name)))
            yield state

    def test_every_production_poll_site_is_individually_required(self):
        """Silencing one exact production poll costs a real cancellation point.

        Poll sites are resolved from live production frames, so the enumeration
        is independent of the phase helpers that implement them.  Each site is
        silenced on its own: the request must then admit work that the site would
        have cancelled, which no redundant poll could.
        """
        self.write_shared_head_merge_requests(1)
        with self.measured_poll_sites(clock=_Clock()) as survey:
            wire, observation = self.exchange(
                self.profile(deadline_ms=600_000), clock=_Clock()
            )
        self.assert_ok(wire)
        sites = dict(survey["sites"])
        phases = dict(survey["phases"])
        self.assertEqual(
            sum(sites.values()), observation["deadline"]["poll_count"]
        )
        # The documented maximum admitted-work quantum holds with every poll in
        # place: no bounded quantum admits more work than this between polls.
        self.assertLessEqual(
            survey["maximum"],
            MAXIMUM_ADMITTED_WORK_QUANTUM,
            f"one admitted-work quantum exceeded its documented bound: {survey['maximum']}",
        )
        owned = sorted(site for site in sites if site[0] in MR_LIST_POLL_OWNERS)
        self.assertGreaterEqual(len(owned), 18, owned)
        self.assertEqual(
            {phases[site] for site in owned},
            {
                "merge request list planning",
                "merge request list project gate",
                "merge request list authority",
                "merge request list validation",
                "merge request list projection",
                "merge request list snapshot revalidation",
                "merge request list response encoding",
                "merge request list response headers",
                "merge request list response publication",
            },
        )

        for site in owned:
            with self.subTest(poll_site=f"{site[0]}:{site[1]}"):
                self.setUp()
                try:
                    self.write_shared_head_merge_requests(1)
                    clock = _TriggeredClock()
                    with self.measured_poll_sites(
                        silenced=site, arm_at=site, clock=clock
                    ) as state:
                        wire, observation = self.exchange(
                            self.profile(deadline_ms=10), clock=clock
                        )
                    self.assertTrue(
                        state["armed"], f"{site} never ran in production"
                    )
                    self.assertEqual(
                        state["sites"].get(site, 0),
                        0,
                        f"{site} survived being silenced",
                    )
                    # The deadline expired at this exact site.  With the site
                    # silenced the request admitted work no other poll could
                    # cancel, which proves this poll is not redundant.
                    self.assertGreater(
                        state["lag"],
                        0,
                        f"{site} bounded no admitted work, so it is redundant",
                    )
                    self.assert_no_ownership_debt(observation)
                finally:
                    self.tearDown()

        # This oracle is strictly finer than replacing one phase helper: the
        # reviewed candidate's mutation removed every site a helper owns at once,
        # so it could not have detected two adjacent redundant polls.
        Operation = self.route_globals["_HTTPMergeRequestListOperation"]
        self.setUp()
        try:
            self.write_shared_head_merge_requests(1)
            with self.patched(Operation, "_validation_poll", lambda owner: None):
                with self.measured_poll_sites(clock=_Clock()) as helper:
                    wire, _ = self.exchange(
                        self.profile(deadline_ms=600_000), clock=_Clock()
                    )
            self.assert_ok(wire)
            silenced_by_helper = {
                site
                for site in sites
                if site[0] in MR_LIST_POLL_OWNERS
                and helper["sites"].get(site, 0) == 0
            }
            self.assertGreater(
                len(silenced_by_helper),
                1,
                "replacing one phase helper silences only one site, so the "
                "reviewed per-helper mutation was not weaker after all",
            )
        finally:
            self.tearDown()

    def test_neighbouring_read_routes_keep_their_own_phases(self):
        """The MR-list phase flag never leaks into a neighbouring read route."""
        branch_wire, branch = self.exchange(
            self.profile(deadline_ms=30_000),
            clock=_Clock(),
            request=b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n",
        )
        branch_body = self.assert_ok(branch_wire)
        self.assertEqual(len(branch_body["branches"]), MR_COUNT + 1)
        self.assert_no_ownership_debt(branch)

        doctor_wire, doctor = self.exchange(
            self.profile(deadline_ms=30_000),
            clock=_Clock(),
            request=b"GET /v1/projects/org/app/doctor HTTP/1.0\r\n\r\n",
        )
        self.assertEqual(self.status_line(doctor_wire), b"HTTP/1.0 200 OK")
        self.assert_no_ownership_debt(doctor)

        deadline = self.route_globals["HTTPRequestDeadline"]
        original_check = deadline.check
        observed = []

        def record_check(owner, phase):
            observed.append(phase)
            return original_check(owner, phase)

        with self.patched(deadline, "check", record_check):
            wire, observation = self.exchange(
                self.profile(deadline_ms=30_000),
                clock=_Clock(),
                request=b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n",
            )
        self.assert_ok(wire)
        self.assertFalse(
            [phase for phase in observed if phase.startswith("merge request list")],
            "the branch-list route polled a merge-request-list phase",
        )
        self.assertIn("branch list response publication", observed)
        self.assert_no_ownership_debt(observation)

    def _observed_response_phases(self, request):
        """Run one request and return its dispatched read-route phase names."""
        dispatched = []
        flags = []

        def configure(handler):
            original_phase = handler.read_route_phase
            original_request = handler.handle_one_request

            def read_route_phase(owner, suffix):
                phase = original_phase(owner, suffix)
                dispatched.append((suffix, phase))
                return phase

            def handle_one_request(owner):
                # Poison the flag before the request is dispatched: a route
                # that fails to reset it per request would inherit a stale
                # merge-request-list response phase from this value.
                owner._mr_list_deadline_active = True
                try:
                    return original_request(owner)
                finally:
                    flags.append(owner._mr_list_deadline_active)

            handler.read_route_phase = read_route_phase
            handler.handle_one_request = handle_one_request

        wire, observation = self.exchange(
            self.profile(deadline_ms=30_000),
            clock=_Clock(),
            configure_handler=configure,
            request=request,
        )
        return wire, observation, dispatched, flags

    def test_response_phase_flag_is_exact_and_reset_for_every_request(self):
        """The MR-list response phase is exact and never leaks to another route.

        The live server never advertises ``HTTP/1.1``, so a second request can
        never reach a handler instance over the same connection.  Per-request
        isolation is therefore proven where it is actually decided: the flag is
        poisoned immediately before ``handle_one_request`` dispatches, so a
        route that inherited it instead of resetting it would name every one of
        its own response boundaries with this route's phase.
        """
        wire, observation, dispatched, flags = self._observed_response_phases(
            MR_LIST_REQUEST
        )
        self.assert_ok(wire)
        self.assertTrue(dispatched)
        # Every read-route response boundary is named for this route exactly.
        self.assertEqual(
            {suffix: phase for suffix, phase in dispatched},
            {
                suffix: f"merge request list {suffix}"
                for suffix in {suffix for suffix, _ in dispatched}
            },
        )
        for suffix in (
            "response encoding",
            "response headers",
            "response publication",
        ):
            self.assertIn(
                (suffix, f"merge request list {suffix}"),
                dispatched,
                suffix,
            )
        self.assertEqual(flags, [True])
        self.assert_no_ownership_debt(observation)

        for label, request in (
            ("branch list", b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n"),
            ("doctor", b"GET /v1/projects/org/app/doctor HTTP/1.0\r\n\r\n"),
            ("unknown", b"GET /v1/projects/org/app/nope HTTP/1.0\r\n\r\n"),
        ):
            with self.subTest(route=label):
                other_wire, other, other_dispatched, other_flags = (
                    self._observed_response_phases(request)
                )
                self.assertFalse(
                    [
                        phase
                        for _, phase in other_dispatched
                        if phase is not None
                        and phase.startswith("merge request list")
                    ],
                    "a neighbouring route dispatched an MR-list response phase",
                )
                # The poisoned flag was reset for this request, not inherited.
                self.assertEqual(other_flags, [False])
                self.assertNotIn(
                    b'"merge_requests":[{', other_wire, "an MR list was published"
                )
                self.assert_no_ownership_debt(other)


    def test_successive_requests_on_one_server_stay_isolated(self):
        """Back-to-back requests on one server share no ledger, phase, or debt."""
        observations = []
        ledgers = []
        phases = []
        globals_ = self.route_globals
        Ledger = globals_["DecisionWorkLedger"]
        original_ledger_init = Ledger.__init__
        deadline = globals_["HTTPRequestDeadline"]
        original_check = deadline.check

        def counting_init(owner, *args, **kwargs):
            ledgers.append(owner)
            return original_ledger_init(owner, *args, **kwargs)

        def record_check(owner, phase):
            phases.append((len(observations), phase))
            return original_check(owner, phase)

        handler = self.namespace["make_http_handler"](
            self.storage,
            self.profile(deadline_ms=30_000),
            _decision_work_observer=observations.append,
            _monotonic_clock=_Clock(),
        )
        server = self.namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        requests = [
            MR_LIST_REQUEST,
            b"GET /v1/projects/org/app/branches HTTP/1.0\r\n\r\n",
            MR_LIST_REQUEST,
        ]
        wires = []
        try:
            with contextlib.ExitStack() as stack:
                stack.enter_context(self.patched(Ledger, "__init__", counting_init))
                stack.enter_context(self.patched(deadline, "check", record_check))
                for request in requests:
                    thread = threading.Thread(
                        target=server.handle_request, daemon=True
                    )
                    thread.start()
                    try:
                        with socket.create_connection(
                            server.server_address, timeout=30
                        ) as client:
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
                        thread.join(timeout=30)
                    self.assertFalse(thread.is_alive())
                    wires.append(b"".join(chunks))
        finally:
            server.server_close()

        self.assertEqual(len(wires), 3)
        self.assertEqual(len(observations), 3)
        # One fresh ledger per request; no ledger is shared or reused.
        self.assertEqual(len(ledgers), 3)
        self.assertEqual(len({id(ledger) for ledger in ledgers}), 3)
        for observation in observations:
            self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
            self.assert_no_ownership_debt(observation)

        # Each request names only its own route's phases, in either order.
        by_request = {index: set() for index in range(3)}
        for index, phase in phases:
            by_request.setdefault(index, set()).add(phase)
        for index, expected_prefix, foreign_prefix in (
            (0, "merge request list ", "branch list "),
            (1, "branch list ", "merge request list "),
            (2, "merge request list ", "branch list "),
        ):
            observed = by_request[index]
            self.assertTrue(
                any(phase.startswith(expected_prefix) for phase in observed),
                f"request {index} polled no {expected_prefix.strip()} phase",
            )
            self.assertFalse(
                [phase for phase in observed if phase.startswith(foreign_prefix)],
                f"request {index} polled a foreign {foreign_prefix.strip()} phase",
            )

        # The two merge-request-list responses are byte-identical, and the
        # interleaved branch-list response never carries a merge-request list.
        self.assert_same_response(wires[0], wires[2])
        self.assertIn(b'"merge_requests"', wires[0])
        self.assertNotIn(b'"merge_requests"', wires[1])
        self.assertIn(b'"branches"', wires[1])


    def test_eight_request_sequence_on_one_server_stays_isolated(self):
        """Eight mixed requests share no ledger, phase, debt, or response state.

        The sequence interleaves a poisoned deadline, a repaired success, a
        corrupt escrow object, a symlinked authority, and the neighbouring
        branch, doctor, and upload routes, so a leak of MR-list phase, debt, or
        response state across any of those boundaries would be observed.
        """
        observations = []
        ledgers = []
        phases = []
        globals_ = self.route_globals
        Ledger = globals_["DecisionWorkLedger"]
        original_ledger_init = Ledger.__init__
        deadline = globals_["HTTPRequestDeadline"]
        original_check = deadline.check
        state = {"index": 0}

        def counting_init(owner, *args, **kwargs):
            ledgers.append(owner)
            return original_ledger_init(owner, *args, **kwargs)

        def record_check(owner, phase):
            phases.append((state["index"], phase))
            return original_check(owner, phase)

        handler = self.namespace["make_http_handler"](
            self.storage,
            self.profile(deadline_ms=30_000),
            _decision_work_observer=observations.append,
            _monotonic_clock=_Clock(),
        )
        server = self.namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        victim_object = self.object_path(self.blob_object_id())
        pinned_object = victim_object.read_bytes()
        branch = self.branch_path(TARGET_BRANCH)
        pinned_branch = branch.read_bytes()

        def corrupt_escrow():
            victim_object.write_text("{broken", encoding="utf-8")

        def repair_escrow():
            victim_object.write_bytes(pinned_object)

        def symlink_branch():
            self._symlink_over(branch)

        def repair_branch():
            branch.unlink()
            branch.write_bytes(pinned_branch)

        steps = [
            ("mr-list clean", MR_LIST_REQUEST, None, b"HTTP/1.0 200 OK"),
            ("mr-list corrupt escrow", MR_LIST_REQUEST, corrupt_escrow, b"HTTP/1.0 400 Bad Request"),
            ("mr-list repaired", MR_LIST_REQUEST, repair_escrow, b"HTTP/1.0 200 OK"),
            ("mr-list symlinked branch", MR_LIST_REQUEST, symlink_branch, b"HTTP/1.0 400 Bad Request"),
            ("branch list", BRANCH_LIST_REQUEST, repair_branch, b"HTTP/1.0 200 OK"),
            ("doctor", DOCTOR_REQUEST, None, b"HTTP/1.0 200 OK"),
            (
                "upload",
                post_request("/v1/projects/org/app/branches/main/upload"),
                None,
                b"HTTP/1.0 400 Bad Request",
            ),
            ("mr-list final", MR_LIST_REQUEST, None, b"HTTP/1.0 200 OK"),
        ]
        wires = []
        try:
            with contextlib.ExitStack() as stack:
                stack.enter_context(self.patched(Ledger, "__init__", counting_init))
                stack.enter_context(self.patched(deadline, "check", record_check))
                for index, (label, request, prepare, expected) in enumerate(steps):
                    state["index"] = index
                    if prepare is not None:
                        prepare()
                    thread = threading.Thread(
                        target=server.handle_request, daemon=True
                    )
                    thread.start()
                    try:
                        with socket.create_connection(
                            server.server_address, timeout=30
                        ) as client:
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
                        thread.join(timeout=30)
                    self.assertFalse(thread.is_alive(), label)
                    wire = b"".join(chunks)
                    wires.append(wire)
                    self.assertEqual(self.status_line(wire), expected, label)
        finally:
            server.server_close()

        self.assertEqual(len(wires), len(steps))
        self.assertEqual(len(observations), len(steps))
        # One fresh ledger per request; no ledger object is ever shared.
        self.assertEqual(len(ledgers), len(steps))
        self.assertEqual(len({id(ledger) for ledger in ledgers}), len(steps))
        for index, observation in enumerate(observations):
            self.assert_no_ownership_debt(observation)
            self.assertIn(
                observation["snapshot"]["terminal_reason"],
                {"complete", "budget_exceeded", "deadline_exceeded"},
                steps[index][0],
            )

        by_request = {index: set() for index in range(len(steps))}
        for index, phase in phases:
            by_request[index].add(phase)
        for index, (label, request, _prepare, _expected) in enumerate(steps):
            observed = by_request[index]
            foreign = [
                phase
                for phase in observed
                if phase.startswith("merge request list")
            ]
            if request == MR_LIST_REQUEST:
                self.assertTrue(foreign, f"{label} polled no MR-list phase")
            else:
                self.assertFalse(foreign, f"{label} polled an MR-list phase")
                # ``"merge_requests":true`` is a Doctor layout flag, so the
                # leaked-projection check names the list envelope itself.
                self.assertNotIn(b'"merge_requests":[', wires[index], label)

        # The three clean merge-request-list responses are byte-identical.
        self.assert_same_response(wires[0], wires[2])
        self.assert_same_response(wires[0], wires[7])

    def test_http_1_0_never_serves_a_pipelined_second_request(self):
        """Two requests in one write yield one response and no shared state."""
        wire, observation = self.exchange(request=MR_LIST_REQUEST * 2)
        body = self.assert_ok(wire)
        self.assertEqual(len(body["merge_requests"]), MR_COUNT)
        self.assertEqual(wire.count(b"HTTP/1.0 "), 1)
        self.assertEqual(wire.count(b'"merge_requests"'), 1)
        self.assert_no_ownership_debt(observation)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")


if __name__ == "__main__":
    unittest.main()
