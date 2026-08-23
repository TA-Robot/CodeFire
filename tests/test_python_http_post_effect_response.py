"""CFR-168 C14-LIVE-A2/B3: the post-effect mutation-response lifecycle.

Every row of the frozen ``T01``-``T15`` counterexample matrix is exercised here
against the real production path.  The rows that concern ownership handoff,
wire framing, cleanup retry, terminal truth and next-request isolation drive the
real ``ReusableThreadingHTTPServer`` handler over a real socket; the pair/slot
transition graph and foreign-ledger rejection are additionally proved by direct
unit tests, as the packet requires.

The behaviour under test is the conservative one-way marker
``EFFECT_MAY_HAVE_STARTED`` (``M``) plus the ownership rules that hang off it:

* before ``M`` the route proves that no authoritative effect began, so a
  disposal fault publishes zero response bytes;
* at and after ``M`` no such proof exists, so the frozen success/durable
  response stays primary and every cleanup fault is strictly secondary;
* the selected owner reaches the mutation slot *before* any fallible
  rejected-owner cleanup, the wire only borrows it, and the request loop retries
  a failed release exactly once on the identical owner.

``M`` is never evidence that a durable byte became visible, that a peer received
anything, or that a transaction committed.
"""

import errno
import json
import re
import runpy
import shutil
import socket
import subprocess
import tempfile
import threading
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"

#: Every matrix row this module is required to cover.
MATRIX_ROWS = tuple(f"T{index:02d}" for index in range(1, 16))

#: Volatile per-run identity that a semantic baseline comparison must mask.
VOLATILE = re.compile(
    r"(MR-[0-9a-f]{12})"
    r"|(CF-COMMIT-[0-9a-f]+)"
    r"|(\d{4}-\d\d-\d\dT[\d:.]+(?:Z|[+-]\d\d:\d\d))"
)


class FailingWriter:
    """Wrap ``wfile`` and fail one exact write with a real ``OSError``.

    A header or body write may already have put a prefix on the wire before it
    raises, so the failure is injected *after* the delegate call when
    ``leak_prefix`` is set.  That is the case the no-second-document rule has to
    survive: an apparently empty capture buffer is not authority to frame a
    fallback document.
    """

    def __init__(self, delegate, fail_on, leak_prefix=False):
        self.delegate = delegate
        self.fail_on = fail_on
        self.leak_prefix = leak_prefix
        self.writes = 0
        self.written = []

    def write(self, data):
        self.writes += 1
        if self.writes == self.fail_on:
            if self.leak_prefix and data:
                prefix = data[: max(1, len(data) // 2)]
                self.written.append(prefix)
                self.delegate.write(prefix)
            raise OSError(errno.EPIPE, "injected socket write failure")
        self.written.append(data)
        return self.delegate.write(data)

    def flush(self):
        return self.delegate.flush()

    def __getattr__(self, name):
        return getattr(self.delegate, name)


class StubMutationHandler:
    """The minimum handler surface the mutation slot actually uses."""

    def __init__(self, ledger, responses=None):
        self.decision_work = ledger
        self.responses = responses
        self.close_connection = False
        self.wfile = None
        self._response_body_dropped_before_release = False
        self.logged = []

    def prepare_mutation_json(self, status, payload):
        if self.responses is None:
            raise AssertionError("no prepared pair was supplied")
        prepared, self.responses = self.responses, None
        return prepared

    def log_error(self, template, *args):
        self.logged.append(template % args)


class PythonHTTPPostEffectResponseTests(unittest.TestCase):
    """Exact ``T01``-``T15`` coverage for the A2/B3 response lifecycle."""

    maxDiff = None

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        subprocess.run(
            [str(CODEFIRE), "init"],
            cwd=self.tmp,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.globals = self.namespace["handle_http_upload"].__globals__
        self.storage = self.tmp / "bounded-http"
        self.project_root = self.namespace["http_project_root"](
            self.storage, "org", "app"
        )
        branch = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )
        self.head = branch["head"]
        self.records = self.reachable_records(
            self.tmp / ".codefire/objects", self.head
        )
        self.profile = self.namespace["DecisionWorkProfile"].http_v1()
        self.retry_payload = self.namespace["HTTP_MUTATION_RETRY_PAYLOAD"]

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    # ---------------------------------------------------------------- helpers

    def object_record_for(self, objects_root, oid):
        matches = list(objects_root.glob(f"*/*{oid}.json"))
        self.assertEqual(len(matches), 1, oid)
        record = json.loads(matches[0].read_text(encoding="utf-8"))
        return {
            "object_id": record["object_id"],
            "type": record["type"],
            "hash": record["hash"],
            "payload": record["payload"],
        }

    def reachable_records(self, objects_root, root_oid):
        records, seen, stack = [], set(), [root_oid]
        while stack:
            oid = stack.pop()
            if oid in seen:
                continue
            seen.add(oid)
            record = self.object_record_for(objects_root, oid)
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
        return sorted(records, key=lambda record: record["object_id"])

    def upload_request(self, key, version="HTTP/1.0", extra_headers=b""):
        body = json.dumps(
            {
                "branch": "main",
                "head": self.head,
                "objects": self.records,
                "idempotency_key": key,
            },
            separators=(",", ":"),
        ).encode()
        return (
            f"POST /v1/projects/org/app/branches/main/upload {version}\r\n".encode()
            + f"Content-Length: {len(body)}\r\n".encode()
            + extra_headers
            + b"\r\n"
            + body
        )

    def exchange(self, request, configure_handler=None, expected=1):
        observations = []
        handler = self.namespace["make_http_handler"](
            self.storage,
            self.profile,
            _decision_work_observer=observations.append,
        )
        if configure_handler is not None:
            configure_handler(handler)
        server = self.namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        thread = threading.Thread(target=server.handle_request, daemon=True)
        thread.start()
        try:
            with socket.create_connection(
                server.server_address, timeout=10
            ) as client:
                client.settimeout(10)
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
            thread.join(timeout=10)
        self.assertFalse(thread.is_alive(), "handler did not terminate")
        self.assertEqual(len(observations), expected, observations)
        return b"".join(chunks), observations

    def bootstrap(self, key="bootstrap"):
        """Publish the durable layout so later uploads are not bootstraps."""
        response, observations = self.exchange(self.upload_request(key))
        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assert_work_closed(observations[0])
        return response

    def assert_work_closed(self, observation):
        snapshot = observation["snapshot"]
        self.assertTrue(
            all(value == 0 for value in snapshot["current"].values()),
            snapshot["current"],
        )
        self.assertEqual(observation["open_permit_count"], 0)

    def mutation_observation(self, observation):
        mutation = observation["mutation_response"]
        self.assertIsNotNone(mutation, "the mutation slot was never observed")
        return mutation

    def durable_snapshot(self):
        """Hash every durable authority byte this project owns."""
        root = self.project_root
        if not root.exists():
            return {}
        import hashlib

        snapshot = {}
        for path in sorted(root.rglob("*")):
            if path.is_file():
                snapshot[str(path.relative_to(root))] = hashlib.sha256(
                    path.read_bytes()
                ).hexdigest()
        return snapshot

    def status_lines(self, response):
        return response.count(b"HTTP/1.0 ")

    def reused_status_lines(self, response):
        return response.count(b"HTTP/1.1 ")

    def reusable_connection(self, inner=None):
        """Let one connection actually serve a second request.

        ``parse_request`` only clears ``close_connection`` when the *server*
        advertises HTTP/1.1, so the stock handler never reuses a connection at
        all.  Raising the advertised version here is what makes the next-request
        isolation rule observable; every frozen response already carries an
        exact ``Content-Length``, so the framing stays valid.
        """

        def configure(handler_class):
            handler_class.protocol_version = "HTTP/1.1"
            if inner is not None:
                inner(handler_class)

        return configure

    def body_of(self, response):
        return response.split(b"\r\n\r\n", 1)[1]

    def fail_owner_close(self, side, times):
        """Make one side of the prepared pair fail its release ``times`` times.

        The injected failure keeps the exact accounting owner intact, exactly as
        a real release fault does, so the retry operates on the identical owner
        rather than on a reconstruction.
        """
        state = {"faults": 0}

        def configure(handler_class):
            original = handler_class.prepare_mutation_json

            def prepare(handler, status, payload):
                responses = original(handler, status, payload)
                owner = getattr(responses, side)
                self.assertIsNotNone(owner)
                original_close = owner.close

                def close():
                    if state["faults"] < times:
                        state["faults"] += 1
                        # A real release fault drops the payload references and
                        # then fails, keeping the byte counts owned.
                        owner.header = None
                        owner.body = None
                        raise self.namespace["CFError"](
                            f"injected {side} release failure"
                        )
                    original_close()

                owner.close = close
                return responses

            handler_class.prepare_mutation_json = prepare

        return configure, state

    def fail_publication_once(self):
        """Publish durably, then lose the confirmed return to this caller."""
        original = self.globals["execute_remote_json_publication"]
        state = {"failed": False}

        def publish_then_lose(*args, **kwargs):
            result = original(*args, **kwargs)
            if not state["failed"]:
                state["failed"] = True
                raise self.namespace["CFError"](
                    "injected post-publication fault"
                )
            return result

        self.globals["execute_remote_json_publication"] = publish_then_lose
        self.addCleanup(
            self.globals.__setitem__,
            "execute_remote_json_publication",
            original,
        )
        return state

    def stub_pair(self, label="post effect response"):
        """Build a real prepared pair on a real ledger without a socket."""
        ledger = self.namespace["DecisionWorkLedger"](
            self.profile, label, command_kind="http_request"
        )
        ledger.acquire("retained_bytes", 4, "response_body")
        response_permit = ledger.reserve(
            "response_body", charges={"response_bytes": 1}
        )
        header_permit = ledger.reserve(
            "response_headers", charges={"response_header_bytes": 1}
        )
        success = self.namespace["_PreparedHTTPResponseOwner"](
            ledger, b"h", 1, b"s", 1
        )
        durable = self.namespace["_PreparedHTTPResponseOwner"](
            ledger, b"h", 1, b"d", 1
        )
        responses = self.namespace["_PreparedHTTPMutationResponses"](
            ledger, success, durable, response_permit, header_permit, 1, 1, 1, 1
        )
        return ledger, responses

    def stub_slot(self, label="post effect slot"):
        ledger, responses = self.stub_pair(label)
        handler = StubMutationHandler(ledger, responses)
        slot = self.namespace["_HTTPMutationResponseSlot"](handler)
        return ledger, handler, slot

    # -------------------------------------------------------- matrix coverage

    def test_t00_matrix_covers_every_frozen_row(self):
        """Every ``T01``-``T15`` row must be named by an executed test."""
        covered = set()
        for name in dir(type(self)):
            match = re.match(r"test_(t\d\d)_", name)
            if match and match.group(1) != "t00":
                covered.add(match.group(1).upper())
        self.assertEqual(
            covered,
            set(MATRIX_ROWS),
            f"missing rows: {sorted(set(MATRIX_ROWS) - covered)}",
        )

    # ------------------------------------------------------------------- T01

    def test_t01_pre_effect_cleanup_fault_publishes_nothing(self):
        """A pre-``M`` disposal fault beside a typed primary emits zero bytes."""
        self.bootstrap("t01-bootstrap")
        before = self.durable_snapshot()
        self.assertTrue(before)

        primary = self.namespace["CFError"]("injected pre-effect authority fault")
        original = self.globals["preflight_remote_json_publication"]

        def refuse(*args, **kwargs):
            raise primary

        self.globals["preflight_remote_json_publication"] = refuse
        self.addCleanup(
            self.globals.__setitem__,
            "preflight_remote_json_publication",
            original,
        )
        configure, state = self.fail_owner_close("success", times=99)

        response, observations = self.exchange(
            self.upload_request("t01"), configure_handler=configure
        )

        # Zero response bytes: no mutation response and no fallback document.
        self.assertEqual(response, b"")
        self.assertEqual(self.status_lines(response), 0)
        # No authority, object, nonce or transaction difference.
        self.assertEqual(self.durable_snapshot(), before)
        mutation = self.mutation_observation(observations[0])
        self.assertTrue(mutation["publication_forbidden"])
        self.assertIsNone(mutation["selection"])
        self.assertFalse(mutation["effect_started"])
        self.assertEqual(mutation["wire_state"], "not_started")
        self.assertFalse(mutation["second_document_forbidden"])
        self.assertGreaterEqual(state["faults"], 1)

    def test_t01_pre_effect_primary_exception_identity_survives(self):
        """The route re-raises the exact primary object, type and ``str``."""
        ledger, handler, slot = self.stub_slot()
        slot.prepare(200, {"ok": True})
        slot.seal_for_effect()
        responses = slot.responses
        primary = self.namespace["CFError"]("exact primary route error")

        def refuse():
            raise self.namespace["CFError"]("injected disposal failure")

        responses.close = refuse
        slot.abort_before_effect()
        # The disposal fault is secondary: it never becomes the primary and it
        # never publishes.  It only forbids a response and closes the socket.
        self.assertTrue(slot.publication_forbidden)
        self.assertTrue(handler.close_connection)
        self.assertEqual(slot.cleanup_attempts, 1)
        self.assertEqual(len(slot.cleanup_faults), 1)
        self.assertIn("injected disposal failure", slot.cleanup_faults[0])
        # A2 never converts a cleanup fault into the primary error family.
        with self.assertRaises(self.namespace["CFError"]) as caught:
            raise primary
        self.assertIs(caught.exception, primary)
        self.assertEqual(str(caught.exception), "exact primary route error")

    # ------------------------------------------------------------------- T02

    def test_t02_durable_selection_survives_one_shot_rejected_fault(self):
        """A one-shot rejected-success fault cannot cost the durable 503."""
        self.bootstrap("t02-bootstrap")
        self.fail_publication_once()
        configure, state = self.fail_owner_close("success", times=1)

        response, observations = self.exchange(
            self.upload_request("t02"), configure_handler=configure
        )

        self.assertTrue(
            response.startswith(b"HTTP/1.0 503 Service Unavailable\r\n"),
            response,
        )
        self.assertEqual(json.loads(self.body_of(response)), self.retry_payload)
        self.assertEqual(self.status_lines(response), 1)
        self.assertEqual(state["faults"], 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["selection"], "durable")
        self.assertEqual(mutation["wire_state"], "complete")
        self.assertEqual(mutation["cleanup_attempts"], 2)
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assertTrue(mutation["cleanup_recovered"])
        self.assertFalse(mutation["rejected_owner_reachable"])
        self.assertFalse(mutation["selected_owner_reachable"])
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "complete"
        )
        self.assert_work_closed(observations[0])

    # ------------------------------------------------------------------- T03

    def test_t03_persistent_rejected_fault_never_destroys_selection(self):
        """A persistent rejected-durable fault still writes the exact 200."""
        self.bootstrap("t03-bootstrap")
        configure, state = self.fail_owner_close("durable", times=99)

        response, observations = self.exchange(
            self.upload_request("t03"), configure_handler=configure
        )

        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(
            json.loads(self.body_of(response)),
            {"branch": "main", "head": self.head},
        )
        self.assertEqual(self.status_lines(response), 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["selection"], "success")
        self.assertEqual(mutation["wire_state"], "complete")
        # The rejected owner stays reachable with its exact debt; nothing is
        # cleared to make the snapshot look clean.
        self.assertTrue(mutation["rejected_owner_reachable"])
        self.assertGreater(mutation["rejected_owner_retained_bytes"], 0)
        self.assertTrue(mutation["cleanup_incomplete"])
        self.assertEqual(mutation["cleanup_attempts"], 2)
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )
        self.assertGreaterEqual(state["faults"], 2)

    # ------------------------------------------------------------------- T04

    def test_t04_route_finally_fault_stays_secondary_to_the_selection(self):
        """A teardown fault after selection cannot replace or re-frame the 200."""
        self.bootstrap("t04-bootstrap")
        state = {"selected": False, "faults": 0}

        # Fire the teardown fault strictly after selection has installed the
        # response, which is the exact window this row is about.
        slot_class = self.globals["_HTTPMutationResponseSlot"]
        original_select = slot_class.select_success

        def select_success(slot):
            original_select(slot)
            state["selected"] = True

        slot_class.select_success = select_success
        self.addCleanup(setattr, slot_class, "select_success", original_select)

        # The staged-upload owner is released in the route's outer ``finally``,
        # after the return value and the selection already exist.
        original = self.globals["prepare_http_upload_stage"]

        def prepared(*args, **kwargs):
            stage = original(*args, **kwargs)
            original_close = stage.close

            def close():
                original_close()
                if state["selected"]:
                    state["faults"] += 1
                    raise self.namespace["CFError"](
                        "injected route-finally fault"
                    )

            stage.close = close
            return stage

        self.globals["prepare_http_upload_stage"] = prepared
        self.addCleanup(
            self.globals.__setitem__, "prepare_http_upload_stage", original
        )

        response, observations = self.exchange(self.upload_request("t04"))

        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(
            json.loads(self.body_of(response)),
            {"branch": "main", "head": self.head},
        )
        # Exactly one framing attempt: the later fault added no second document.
        self.assertEqual(self.status_lines(response), 1)
        self.assertGreaterEqual(state["faults"], 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["selection"], "success")
        self.assertEqual(mutation["wire_state"], "complete")
        self.assertTrue(mutation["second_document_forbidden"])

    # ------------------------------------------------------------------- T05

    def test_t05_opposite_selection_is_typed_invalid_without_state_change(self):
        """Direction is immutable; the retry reuses the identical owners."""
        ledger, responses = self.stub_pair()
        foreign = self.namespace["DecisionWorkLedger"](
            self.profile, "foreign", command_kind="http_request"
        )
        before = ledger.snapshot()
        foreign_before = foreign.snapshot()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "different decision-work ledger"
        ):
            responses.select_success(foreign)
        self.assertEqual(ledger.snapshot(), before)
        self.assertEqual(foreign.snapshot(), foreign_before)

        responses.seal_for_effect(ledger)
        selected = responses.select_durable(ledger)
        rejected = responses.rejected
        release = ledger.release
        faults = {"count": 0}

        def fail_once(dimension, amount):
            faults["count"] += 1
            if faults["count"] == 1:
                raise self.namespace["CFError"]("injected release failure")
            return release(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(
            self.namespace["CFError"], "injected release failure"
        ):
            responses.release_rejected()
        self.assertIs(responses.rejected, rejected)
        self.assertEqual(responses.state, "cleanup_pending")

        after_fault = ledger.snapshot()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "selection cannot be changed"
        ):
            responses.select_success(ledger)
        self.assertEqual(ledger.snapshot(), after_fault)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "selection already completed"
        ):
            responses.select_durable(ledger)
        self.assertEqual(ledger.snapshot(), after_fault)
        # The selected payload was never revoked by the failed cleanup.
        self.assertIsNotNone(selected.header)
        self.assertIsNotNone(selected.body)

        responses.release_rejected()
        self.assertIsNone(responses.rejected)
        self.assertEqual(responses.selection, "durable")
        selected.close()
        ledger.release = release
        self.assertEqual(ledger.current.get("retained_bytes", 0), 0)

    def test_t05_slot_transition_graph_rejects_impossible_states(self):
        """The slot's declared transition graph is enforced, not advisory."""
        ledger, handler, slot = self.stub_slot()
        self.assertEqual(slot.state, "empty")
        with self.assertRaisesRegex(
            self.namespace["CFError"], "was not prepared"
        ):
            slot.select_success()
        slot.prepare(200, {"ok": True})
        with self.assertRaisesRegex(
            self.namespace["CFError"], "cannot be selected before its effect"
        ):
            slot.select_success()
        self.assertEqual(slot.state, "prepared")
        with self.assertRaisesRegex(
            self.namespace["CFError"], "must be sealed before its effect"
        ):
            slot.mark_effect_started()
        slot.seal_for_effect()
        self.assertEqual(slot.state, "sealed")
        self.assertFalse(slot.effect_started)
        slot.mark_effect_started()
        self.assertEqual(slot.state, "effect_may_have_started")
        self.assertTrue(slot.effect_started)
        # ``M`` is one-way: the pre-effect abort is now permanently illegal.
        with self.assertRaisesRegex(
            self.namespace["CFError"], "after its effect may have started"
        ):
            slot.abort_before_effect()
        slot.select_success()
        self.assertEqual(slot.state, "selected")
        self.assertEqual(slot.selection, "success")
        self.assertTrue(slot.has_selected)
        # Selection cannot precede ``M``, so the abort stays refused here too.
        with self.assertRaisesRegex(
            self.namespace["CFError"], "after its effect may have started"
        ):
            slot.abort_before_effect()
        # The selected owner is not a cleanup remainder until its wire attempt.
        self.assertFalse(slot.selected_release_due)
        self.assertFalse(slot.cleanup_incomplete)
        slot.close()
        self.assertEqual(slot.state, "closed")

    # ------------------------------------------------------------------- T06

    def test_t06_header_write_fault_forbids_any_second_document(self):
        """A header prefix on the wire permanently forbids a fallback."""
        self.bootstrap("t06-bootstrap")

        def configure(handler_class):
            original_setup = handler_class.setup

            def setup(handler):
                original_setup(handler)
                handler.wfile = FailingWriter(
                    handler.wfile, fail_on=1, leak_prefix=True
                )

            handler_class.setup = setup

        response, observations = self.exchange(
            self.upload_request("t06"), configure_handler=configure
        )

        # Whatever prefix escaped, no complete second document followed it.
        self.assertLessEqual(self.status_lines(response), 1)
        self.assertNotIn(b"HTTP/1.0 500", response)
        self.assertNotIn(b"HTTP/1.0 503", response)
        mutation = self.mutation_observation(observations[0])
        # WIRE_STARTED was recorded before the first write was attempted.
        self.assertEqual(mutation["wire_state"], "started")
        self.assertTrue(mutation["second_document_forbidden"])
        self.assertEqual(mutation["selection"], "success")
        # Accounting cleanup ran outside the transport boundary and completed.
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "complete"
        )
        self.assert_work_closed(observations[0])

    # ------------------------------------------------------------------- T07

    def test_t07_body_write_fault_appends_no_error_document(self):
        """A failed body write never appends a JSON error document."""
        self.bootstrap("t07-bootstrap")

        def configure(handler_class):
            original_setup = handler_class.setup

            def setup(handler):
                original_setup(handler)
                handler.wfile = FailingWriter(
                    handler.wfile, fail_on=2, leak_prefix=True
                )

            handler_class.setup = setup

        response, observations = self.exchange(
            self.upload_request("t07"), configure_handler=configure
        )

        self.assertEqual(self.status_lines(response), 1)
        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        # Exactly one header framing attempt and no appended error document.
        self.assertEqual(response.count(b"Content-Type: application/json"), 1)
        self.assertNotIn(b'{"error"', self.body_of(response))
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["wire_state"], "started")
        self.assertTrue(mutation["second_document_forbidden"])
        # The selected owner's accounting still executed.
        self.assertFalse(mutation["selected_owner_reachable"])
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assert_work_closed(observations[0])

    # ------------------------------------------------------------------- T08

    def test_t08_post_write_cleanup_oserror_is_not_transport(self):
        """An accounting ``OSError`` after both writes is never transport."""
        self.bootstrap("t08-bootstrap")
        captured = []
        origin = self.namespace["os_error_origin"]

        def configure(handler_class):
            original = handler_class.prepare_mutation_json

            def prepare(handler, status, payload):
                responses = original(handler, status, payload)
                owner = responses.success
                original_close = owner.close
                state = {"faults": 0}

                def close():
                    if state["faults"] == 0:
                        state["faults"] += 1
                        owner.header = None
                        owner.body = None
                        error = OSError(
                            errno.EIO, "injected accounting release failure"
                        )
                        captured.append(error)
                        raise error
                    original_close()

                owner.close = close
                return responses

            handler_class.prepare_mutation_json = prepare

        response, observations = self.exchange(
            self.upload_request("t08"), configure_handler=configure
        )

        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(
            json.loads(self.body_of(response)),
            {"branch": "main", "head": self.head},
        )
        self.assertEqual(len(captured), 1)
        # The cleanup fault followed both returned writes and is still not
        # transport-origin, so it cannot abandon the connection as a socket
        # failure or hide the accounting debt it actually represents.
        self.assertFalse(self.namespace["is_transport_os_error"](captured[0]))
        self.assertNotEqual(origin(captured[0]), "transport")
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["wire_state"], "complete")
        # One synchronous retry on the identical owner drained the debt.
        self.assertEqual(mutation["cleanup_attempts"], 3)
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assertTrue(mutation["cleanup_recovered"])
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "complete"
        )
        self.assert_work_closed(observations[0])

    # ------------------------------------------------------------------- T09

    def test_t09_request_loop_retries_the_identical_slot_once(self):
        """The handler keeps one slot; the retry is synchronous and bounded."""
        self.bootstrap("t09-bootstrap")
        self.fail_publication_once()
        configure_fault, state = self.fail_owner_close("success", times=1)

        created = []
        retried = []
        slot_class = self.globals["_HTTPMutationResponseSlot"]
        original_init = slot_class.__init__
        original_finish = slot_class.finish_request_cleanup

        def init(slot, handler):
            original_init(slot, handler)
            created.append(slot)

        def finish(slot):
            retried.append(slot)
            return original_finish(slot)

        slot_class.__init__ = init
        slot_class.finish_request_cleanup = finish
        self.addCleanup(setattr, slot_class, "__init__", original_init)
        self.addCleanup(
            setattr, slot_class, "finish_request_cleanup", original_finish
        )

        response, observations = self.exchange(
            self.upload_request("t09"), configure_handler=configure_fault
        )

        self.assertTrue(
            response.startswith(b"HTTP/1.0 503 Service Unavailable\r\n"),
            response,
        )
        # Exactly one slot existed, and the request loop retried that same one.
        self.assertEqual(len(created), 1)
        self.assertEqual(len(retried), 1)
        self.assertIs(created[0], retried[0])
        self.assertIsNone(retried[0].handler._mutation_slot)
        self.assertEqual(state["faults"], 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["cleanup_attempts"], 2)
        self.assertFalse(mutation["cleanup_incomplete"])
        # One terminal observation for the whole request.
        self.assertEqual(len(observations), 1)

    # ------------------------------------------------------------------- T10

    def test_t10_persistent_cleanup_is_reported_truthfully(self):
        """Two failed attempts leave a reachable owner and a truthful terminal."""
        self.bootstrap("t10-bootstrap")
        configure, state = self.fail_owner_close("durable", times=99)

        response, observations = self.exchange(
            self.upload_request("t10"), configure_handler=configure
        )

        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(self.status_lines(response), 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["selection"], "success")
        self.assertEqual(mutation["wire_state"], "complete")
        self.assertEqual(mutation["slot_state"], "cleanup_pending")
        self.assertTrue(mutation["rejected_owner_reachable"])
        self.assertGreater(mutation["rejected_owner_retained_bytes"], 0)
        self.assertEqual(mutation["cleanup_attempts"], 2)
        self.assertEqual(len(mutation["cleanup_faults"]), 2)
        for fault in mutation["cleanup_faults"]:
            self.assertLessEqual(
                len(fault),
                self.namespace["HTTP_MUTATION_CLEANUP_DIAGNOSTIC_LIMIT"],
            )
        self.assertTrue(mutation["cleanup_incomplete"])
        self.assertFalse(mutation["cleanup_recovered"])
        # Exact remaining debt is reported, not zeroed.
        self.assertTrue(any(mutation["current"].values()))
        self.assertEqual(mutation["open_permit_count"], 0)
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )
        self.assertGreaterEqual(state["faults"], 2)

    # ------------------------------------------------------------------- T11

    def test_t11_clean_confirmed_upload_is_complete_and_silent(self):
        """A clean upload emits the exact success and no cleanup diagnostic."""
        self.bootstrap("t11-bootstrap")
        logged = []

        def configure(handler_class):
            original = handler_class.log_error

            def log_error(handler, template, *args):
                logged.append(template % args)
                return original(handler, template, *args)

            handler_class.log_error = log_error

        response, observations = self.exchange(
            self.upload_request("t11"), configure_handler=configure
        )

        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(
            json.loads(self.body_of(response)),
            {"branch": "main", "head": self.head},
        )
        self.assertEqual(self.status_lines(response), 1)
        mutation = self.mutation_observation(observations[0])
        self.assertEqual(mutation["selection"], "success")
        self.assertEqual(mutation["wire_state"], "complete")
        self.assertEqual(mutation["slot_state"], "closed")
        self.assertEqual(mutation["cleanup_faults"], [])
        self.assertFalse(mutation["cleanup_recovered"])
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assertEqual(mutation["open_permit_count"], 0)
        self.assertFalse(any(mutation["current"].values()))
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "complete"
        )
        self.assert_work_closed(observations[0])
        self.assertEqual(
            [entry for entry in logged if "recovered cleanup fault" in entry], []
        )

    # ------------------------------------------------------------------- T12

    def test_t12_recovered_cleanup_reports_one_bounded_diagnostic(self):
        """A repaired fault is consumed exactly once and never becomes primary."""
        self.bootstrap("t12-bootstrap")
        configure_fault, state = self.fail_owner_close("durable", times=1)
        logged = []

        def configure(handler_class):
            configure_fault(handler_class)
            original = handler_class.log_error

            def log_error(handler, template, *args):
                logged.append(template % args)
                return original(handler, template, *args)

            handler_class.log_error = log_error

        response, observations = self.exchange(
            self.upload_request("t12"), configure_handler=configure
        )

        # The exact selected response, not the diagnostic, is the outcome.
        self.assertTrue(response.startswith(b"HTTP/1.0 200 OK\r\n"), response)
        self.assertEqual(
            json.loads(self.body_of(response)),
            {"branch": "main", "head": self.head},
        )
        self.assertEqual(state["faults"], 1)
        recovered = [
            entry for entry in logged if "recovered cleanup fault" in entry
        ]
        self.assertEqual(len(recovered), 1, logged)
        self.assertLess(len(recovered[0]), 1024)
        mutation = self.mutation_observation(observations[0])
        self.assertTrue(mutation["cleanup_recovered"])
        self.assertFalse(mutation["cleanup_incomplete"])
        self.assertEqual(mutation["cleanup_attempts"], 2)
        self.assertFalse(any(mutation["current"].values()))
        self.assertEqual(mutation["open_permit_count"], 0)
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "complete"
        )

    # ------------------------------------------------------------------- T13

    def test_t13_pre_effect_filesystem_error_keeps_typed_authority(self):
        """``ETIMEDOUT``/``EIO`` before ``M`` stays the typed 400 authority."""
        self.bootstrap("t13-bootstrap")
        before = self.durable_snapshot()

        for label, factory in (
            (
                "etimedout",
                lambda: TimeoutError(errno.ETIMEDOUT, "injected fs timeout"),
            ),
            ("eio", lambda: OSError(errno.EIO, "injected fs eio")),
        ):
            with self.subTest(error=label):
                original = self.globals["read_remote_branch_record"]
                raised = []

                def failing(*args, **kwargs):
                    self.globals["read_remote_branch_record"] = original
                    error = factory()
                    raised.append(error)
                    raise error

                self.globals["read_remote_branch_record"] = failing
                try:
                    response, observations = self.exchange(
                        self.upload_request(f"t13-{label}")
                    )
                finally:
                    self.globals["read_remote_branch_record"] = original

                # Baseline observed on L: the typed filesystem-authority 400.
                self.assertTrue(
                    response.startswith(b"HTTP/1.0 400 Bad Request\r\n"),
                    response,
                )
                self.assertIn(
                    "filesystem authority is unusable",
                    json.loads(self.body_of(response))["error"],
                )
                self.assertEqual(self.status_lines(response), 1)
                self.assertEqual(len(raised), 1)
                self.assertFalse(
                    self.namespace["is_transport_os_error"](raised[0])
                )
                # No mutation effect reached the durable authority.
                self.assertEqual(self.durable_snapshot(), before)
                self.assertEqual(
                    observations[0]["snapshot"]["terminal_reason"], "complete"
                )
                self.assert_work_closed(observations[0])

        with self.subTest(error="memoryerror"):
            original = self.globals["read_remote_branch_record"]

            def fatal(*args, **kwargs):
                self.globals["read_remote_branch_record"] = original
                raise MemoryError("injected process-fatal condition")

            self.globals["read_remote_branch_record"] = fatal
            try:
                response, _ = self.exchange(self.upload_request("t13-memory"))
            finally:
                self.globals["read_remote_branch_record"] = original
            # ``MemoryError`` stays outside the filesystem-authority arm.
            self.assertNotIn(b"filesystem authority is unusable", response)

    # ------------------------------------------------------------------- T14

    def test_t14_slotless_routes_keep_their_l_baseline(self):
        """GC and merge-request routes are untouched by the A2 slot."""
        self.bootstrap("t14-bootstrap")

        def post(path, payload):
            body = json.dumps(payload, separators=(",", ":")).encode()
            return (
                f"POST {path} HTTP/1.0\r\nContent-Length: {len(body)}\r\n\r\n".encode()
                + body
            )

        # Every expected value below was observed on the launch base ``L``
        # before any candidate edit, so the candidate cannot manufacture it.
        scenarios = (
            ("gc_success", post("/v1/projects/org/app/gc", {}), 200, None),
            (
                "gc_unknown_project",
                post("/v1/projects/org/nope/gc", {}),
                404,
                "remote project not found",
            ),
            (
                "mr_create_missing_urls",
                post("/v1/projects/org/app/merge-requests", {}),
                400,
                "request-merge requires source_url and target_url",
            ),
            (
                "mr_review_not_found",
                post(
                    "/v1/projects/org/app/merge-requests/MR-000000000000/review",
                    {"decision": "approve", "reviewer": "alice"},
                ),
                404,
                "merge request not found",
            ),
            (
                "mr_review_bad_decision",
                post(
                    "/v1/projects/org/app/merge-requests/MR-000000000000/review",
                    {"decision": "maybe", "reviewer": "alice"},
                ),
                404,
                # Observed on ``L``: the merge request is resolved before the
                # decision is validated, so an absent id still answers 404.
                "merge request not found",
            ),
            (
                "mr_apply_not_found",
                post(
                    "/v1/projects/org/app/merge-requests/MR-000000000000/apply",
                    {},
                ),
                404,
                "merge request not found",
            ),
            (
                "unknown_endpoint",
                post("/v1/projects/org/app/nope", {}),
                404,
                "unknown endpoint",
            ),
            (
                "invalid_json",
                b"POST /v1/projects/org/app/gc HTTP/1.0\r\n"
                b"Content-Length: 3\r\n\r\n{[}",
                400,
                "invalid JSON request",
            ),
        )

        for name, request, status, error in scenarios:
            with self.subTest(route=name):
                response, observations = self.exchange(request)
                self.assertTrue(
                    response.startswith(f"HTTP/1.0 {status} ".encode()),
                    (name, response[:64]),
                )
                self.assertEqual(self.status_lines(response), 1)
                if error is not None:
                    self.assertEqual(
                        json.loads(self.body_of(response)).get("error"), error
                    )
                # A slotless route never acquires an A2 mutation slot.
                self.assertIsNone(observations[0]["mutation_response"])
                self.assertEqual(
                    observations[0]["snapshot"]["terminal_reason"], "complete"
                )
                self.assert_work_closed(observations[0])

    # ------------------------------------------------------------------- T15

    def test_t15_recovered_cleanup_leaves_no_state_for_the_next_request(self):
        """A reused connection sees no A2 selection, wire or cleanup state."""
        self.bootstrap("t15-bootstrap")
        fault_configure, state = self.fail_owner_close("durable", times=1)
        configure = self.reusable_connection(fault_configure)

        first = self.upload_request("t15-first", version="HTTP/1.1")
        second = (
            b"POST /v1/projects/org/app/gc HTTP/1.1\r\n"
            b"Content-Length: 2\r\n"
            b"Connection: close\r\n\r\n{}"
        )

        response, observations = self.exchange(
            first + second, configure_handler=configure, expected=2
        )

        self.assertEqual(state["faults"], 1)
        self.assertEqual(self.reused_status_lines(response), 2)
        self.assertTrue(response.startswith(b"HTTP/1.1 200 OK\r\n"), response)
        first_observation, second_observation = observations
        # The recovered request reset every A2 field it owned.
        first_mutation = self.mutation_observation(first_observation)
        self.assertTrue(first_mutation["cleanup_recovered"])
        self.assertFalse(first_mutation["cleanup_incomplete"])
        # The second request carries no A2 state at all.
        self.assertIsNone(second_observation["mutation_response"])
        self.assertEqual(
            second_observation["snapshot"]["terminal_reason"], "complete"
        )
        self.assert_work_closed(second_observation)

    def test_t15_persistent_cleanup_closes_before_a_second_request(self):
        """A persistent A2 failure never lets the connection serve again."""
        self.bootstrap("t15-persistent-bootstrap")
        fault_configure, _ = self.fail_owner_close("durable", times=99)
        configure = self.reusable_connection(fault_configure)

        # Both requests ask for reuse; only the residual A2 debt may stop the
        # second one from ever being served.
        first = self.upload_request("t15-persistent", version="HTTP/1.1")
        second = (
            b"POST /v1/projects/org/app/gc HTTP/1.1\r\n"
            b"Content-Length: 2\r\n"
            b"Connection: close\r\n\r\n{}"
        )

        response, observations = self.exchange(
            first + second, configure_handler=configure, expected=1
        )

        # Exactly one response: the connection closed on the residual debt.
        self.assertEqual(self.reused_status_lines(response), 1)
        self.assertTrue(response.startswith(b"HTTP/1.1 200 OK\r\n"), response)
        mutation = self.mutation_observation(observations[0])
        self.assertTrue(mutation["cleanup_incomplete"])
        self.assertEqual(
            observations[0]["snapshot"]["terminal_reason"], "cleanup_incomplete"
        )


if __name__ == "__main__":
    unittest.main()
