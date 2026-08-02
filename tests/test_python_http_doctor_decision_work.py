"""CFR-168: the Python HTTP Doctor route must run on the request ledger.

The independent current-state review reproduced that ``GET /doctor`` built a
second ``DecisionWorkLedger(DecisionWorkProfile.local_v1(), "remote doctor")``,
so the route consumed a much larger hidden budget while the request's own
``http_v1`` ledger stayed empty and its deadline was never polled.  Every test
below drives the real production wire route through ``make_http_handler`` with
injected clocks and a temporary filesystem fixture; none of them sleep.
"""

import contextlib
import json
import runpy
import unittest
from pathlib import Path

import tests.test_codefire_cli as cli_tests


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"

DOCTOR_REQUEST = b"GET /v1/projects/org/app/doctor HTTP/1.0\r\n\r\n"
QUICK_REQUEST = b"GET /v1/projects/org/app/doctor?quick=1 HTTP/1.0\r\n\r\n"


class InjectedClock:
    """One deterministic monotonic clock with no wall-clock dependency."""

    def __init__(self, start=0.0):
        self.now = float(start)
        self.reads = 0

    def __call__(self):
        self.reads += 1
        return self.now


class StartThenExpireClock(InjectedClock):
    """Return the request start once, then a value past every deadline."""

    def __init__(self, expired):
        super().__init__(0.0)
        self.expired = float(expired)

    def __call__(self):
        value = 0.0 if self.reads == 0 else self.expired
        self.reads += 1
        return value


class PythonHTTPDoctorDecisionWorkTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.globals = cls.ns["make_http_handler"].__globals__
        cls.CFError = cls.ns["CFError"]

    def setUp(self):
        self.harness = cli_tests.CodeFireCliTests(methodName="runTest")
        self.harness.setUp()
        self.harness.prepare_bounded_http_bundle(self.ns)
        self.storage_root = self.harness.tmp / "bounded-http"
        self.project_root = self.ns["http_project_root"](
            self.storage_root, "org", "app"
        )
        self.url = "cf+http://org/app"

    def tearDown(self):
        self.harness.tearDown()

    # ---------------------------------------------------------------- helpers

    def profile(self, overrides=None, deadline_ms=30_000):
        base = self.ns["DecisionWorkProfile"].http_v1(overrides)
        return self.ns["DecisionWorkProfile"](
            dict(base.limits),
            profile_id=base.profile_id,
            request_deadline_ms=deadline_ms,
            max_active_connections=base.max_active_connections,
            max_queued_connections=base.max_queued_connections,
            applicable_commands=tuple(self.ns["DECISION_WORK_COMMANDS"]),
            applicable_phases=tuple(self.ns["DECISION_WORK_PHASES"]),
        )

    @contextlib.contextmanager
    def patched(self, **replacements):
        originals = {name: self.globals[name] for name in replacements}
        self.globals.update(replacements)
        try:
            yield
        finally:
            self.globals.update(originals)

    @contextlib.contextmanager
    def recorded_ledgers(self):
        """Record every ``DecisionWorkLedger`` built while a request runs."""
        original = self.globals["DecisionWorkLedger"]
        ledgers = []

        class RecordingLedger(original):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                ledgers.append(self)

        with self.patched(DecisionWorkLedger=RecordingLedger):
            yield ledgers

    @contextlib.contextmanager
    def recorded_doctor_payloads(self):
        """Record the ledger each production Doctor payload is bound to."""
        original = self.globals["AccountedDoctorPayload"]
        owners = []

        class RecordingPayload(original):
            def __init__(self, payload, ledger, retained_bytes, owns_ledger=True):
                super().__init__(payload, ledger, retained_bytes, owns_ledger)
                owners.append(
                    {
                        "payload": self,
                        "ledger": ledger,
                        "retained_bytes": retained_bytes,
                        "owns_ledger": owns_ledger,
                        "body": dict(self),
                    }
                )

        with self.patched(AccountedDoctorPayload=RecordingPayload):
            yield owners

    def exchange(self, request=DOCTOR_REQUEST, profile=None, clock=None):
        return self.harness.exchange_http_handler(
            self.ns,
            profile if profile is not None else self.profile(),
            request,
            monotonic_clock=clock,
        )

    def body_of(self, wire):
        return wire.split(b"\r\n\r\n", 1)[1]

    def assert_no_debt(self, observation):
        snapshot = observation["snapshot"]
        self.assertTrue(
            all(value == 0 for value in snapshot["current"].values()),
            snapshot["current"],
        )
        self.assertEqual(observation["open_permit_count"], 0)

    def assert_single_request_ledger(self, ledgers):
        self.assertEqual(len(ledgers), 1)
        ledger = ledgers[0]
        self.assertEqual(ledger.command, "http request")
        self.assertEqual(ledger.command_kind, "http_request")
        self.assertIsNotNone(ledger.profile.request_deadline_ms)
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        return ledger

    def assert_http_request_budget(self, ledger):
        """The route runs on the ``http_v1`` budget, not the larger local one."""
        self.assertEqual(ledger.profile.limit("retained_bytes"), 32 * 1024 * 1024)
        self.assertEqual(
            self.ns["DecisionWorkProfile"].local_v1().limit("retained_bytes"),
            1024 * 1024 * 1024,
        )

    def direct_payload(self, *, quick=False):
        """Run the deliberately local direct/compatibility Doctor path."""
        return self.ns["remote_doctor_payload"](
            self.project_root, self.url, quick=quick
        )

    # ------------------------------------------------------------- ownership

    def test_doctor_payload_is_owned_by_the_exact_http_request_ledger(self):
        with self.recorded_ledgers() as ledgers, self.recorded_doctor_payloads() as owners:
            wire, observation = self.exchange()
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        ledger = self.assert_single_request_ledger(ledgers)
        self.assert_http_request_budget(ledger)
        self.assertEqual(len(owners), 1)
        owner = owners[0]
        self.assertIs(owner["ledger"], ledger)
        self.assertIs(owner["payload"]._ledger, ledger)
        self.assertEqual(owner["payload"]._ledger_identity, id(ledger))
        # The request route, not the payload, owns the terminal decision.
        self.assertFalse(owner["owns_ledger"])
        self.assertGreater(owner["retained_bytes"], 0)
        self.assertTrue(owner["payload"]._closed)
        self.assertEqual(owner["payload"]._retained_bytes, 0)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_debt(observation)

    def test_request_ledger_records_doctor_work_without_a_second_ledger(self):
        with self.recorded_ledgers() as ledgers:
            wire, observation = self.exchange()
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        self.assert_http_request_budget(self.assert_single_request_ledger(ledgers))
        self.assertNotIn("remote doctor", [item.command for item in ledgers])
        snapshot = observation["snapshot"]
        payload = json.loads(self.body_of(wire))
        self.assertTrue(payload["ok"])
        self.assertGreater(payload["checked_objects"], 0)
        self.assertEqual(
            snapshot["cumulative"]["object_records"], payload["checked_objects"]
        )
        self.assertGreater(snapshot["cumulative"]["graph_nodes"], 0)
        self.assertGreater(snapshot["cumulative"]["graph_edges"], 0)
        self.assertGreater(snapshot["cumulative"]["snapshot_revalidation_records"], 0)
        self.assertGreater(snapshot["counters"]["record_reads"], 0)
        self.assertGreater(snapshot["counters"]["semantic_validations"], 0)
        self.assertGreater(observation["deadline"]["poll_count"], 0)
        self.assertIsNone(observation["deadline"]["expired_phase"])
        self.assert_no_debt(observation)

    def test_empty_request_ledger_observation_is_a_regression_test(self):
        """The reproduced defect: Doctor work never reached the request ledger."""
        _wire, observation = self.exchange()
        phases = observation["snapshot"]["phases"]
        # Pre-fix only the transport phases appeared here; every Doctor phase
        # below was charged to a second, hidden ``local_v1`` ledger.
        for phase in [
            "graph_discovery",
            "record_read",
            "semantic_validation",
            "query",
            "snapshot_revalidation",
        ]:
            self.assertIn(phase, phases)
        self.assertGreater(phases["record_read"]["aggregate_bytes"], 0)
        self.assertGreater(
            phases["snapshot_revalidation"]["snapshot_revalidation_records"], 0
        )
        self.assertNotEqual(
            sorted(phases),
            ["request_headers", "response_body", "response_headers"],
        )
        # The deadline is the request's own, so it was actually polled.
        self.assertGreater(observation["deadline"]["poll_count"], 0)

    # ----------------------------------------------------------- domain shape

    def test_quick_and_full_modes_preserve_domain_output_and_order(self):
        for request, quick in [(DOCTOR_REQUEST, False), (QUICK_REQUEST, True)]:
            with self.subTest(quick=quick):
                wire, observation = self.exchange(request)
                self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
                body = self.body_of(wire)
                direct = self.direct_payload(quick=quick)
                try:
                    expected = self.ns["canonical_json"](direct)
                    expected_issues = list(direct["issues"])
                finally:
                    direct.close()
                # Identical JSON bytes, so identical keys, order and issues.
                self.assertEqual(body, expected)
                route_payload = json.loads(body)
                self.assertEqual(route_payload["issues"], expected_issues)
                self.assertIs(route_payload["quick"], quick)
                self.assertEqual(
                    route_payload["skipped_checks"],
                    ["object_hash_and_reference_graph"] if quick else [],
                )
                self.assertEqual(route_payload["issues"], [])
                self.assertTrue(route_payload["ok"])
                self.assertEqual(route_payload["transport"], "http")
                self.assertEqual(route_payload["project"], self.url)
                if quick:
                    self.assertEqual(route_payload["checked_objects"], 0)
                else:
                    self.assertGreater(route_payload["checked_objects"], 0)
                self.assertEqual(
                    observation["snapshot"]["terminal_reason"], "complete"
                )
                self.assert_no_debt(observation)

    def test_full_mode_keeps_diagnostic_precedence_through_the_request_route(self):
        missing = sorted(
            (self.project_root / "objects" / "atom_indexes").glob("*.json")
        )
        self.assertTrue(missing)
        missing[0].unlink()
        with self.recorded_ledgers() as ledgers:
            wire, observation = self.exchange()
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        self.assert_single_request_ledger(ledgers)
        route_payload = json.loads(self.body_of(wire))
        self.assertFalse(route_payload["ok"])
        direct = self.direct_payload()
        try:
            self.assertEqual(self.body_of(wire), self.ns["canonical_json"](direct))
            expected_issues = list(direct["issues"])
            expected_missing = list(direct["missing_objects"])
        finally:
            direct.close()
        # Diagnostic precedence and per-list order stay exactly as the
        # domain layer produced them.
        self.assertEqual(route_payload["issues"], expected_issues)
        self.assertEqual(route_payload["missing_objects"], expected_missing)
        kinds = [issue["kind"] for issue in route_payload["issues"]]
        self.assertEqual(
            set(kinds),
            {"remote_object_reference_missing", "remote_sealed_commit_invalid"},
        )
        # Production emits every missing-reference issue before every
        # sealed-commit issue; the route must not regroup them.
        first_sealed = kinds.index("remote_sealed_commit_invalid")
        self.assertNotIn("remote_object_reference_missing", kinds[first_sealed:])
        self.assertGreater(route_payload["missing_object_references"], 0)
        self.assertGreater(route_payload["invalid_sealed_commit_references"], 0)
        self.assert_no_debt(observation)

    # -------------------------------------------------------- typed exhaustion

    def observed_bounds(self):
        _wire, observation = self.exchange()
        snapshot = observation["snapshot"]
        return {
            "object_records": snapshot["cumulative"]["object_records"],
            "retained_bytes": snapshot["peak"]["retained_bytes"],
        }

    def assert_typed_budget_failure(self, dimension, limit, override=None):
        with self.recorded_ledgers() as ledgers:
            wire, observation = self.exchange(
                profile=self.profile(
                    {dimension: limit if override is None else override}
                )
            )
        self.assertTrue(wire.startswith(b"HTTP/1.0 413 "), wire[:64])
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        payload = json.loads(self.body_of(wire))
        self.assertEqual(payload["kind"], "resource_budget_exceeded")
        self.assertEqual(payload["exit_code"], 60)
        diagnostic = payload["diagnostics"][0]
        self.assertEqual(diagnostic["dimension"], dimension)
        self.assertEqual(diagnostic["limit"], limit)
        self.assertGreater(diagnostic["observed"], limit)
        self.assertEqual(diagnostic["command"], "http request")
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "budget_exceeded"
        )
        # No partial Doctor payload, no retained/temporary scalar and no
        # hidden second ledger survive the typed failure.
        self.assertNotIn(b"codefire_remote_doctor_report", wire)
        self.assert_single_request_ledger(ledgers)
        self.assert_no_debt(observation)
        return diagnostic

    def test_work_limit_exact_passes_and_limit_minus_one_is_typed(self):
        bounds = self.observed_bounds()
        exact = bounds["object_records"]
        self.assertGreater(exact, 1)
        wire, observation = self.exchange(
            profile=self.profile({"object_records": exact})
        )
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        self.assertTrue(json.loads(self.body_of(wire))["ok"])
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_debt(observation)
        self.assert_typed_budget_failure("object_records", exact - 1)

    def test_retained_limit_exact_passes_and_limit_minus_one_is_typed(self):
        bounds = self.observed_bounds()
        exact = bounds["retained_bytes"]
        self.assertGreater(exact, 1)
        wire, observation = self.exchange(
            profile=self.profile({"retained_bytes": ("concurrent", exact)})
        )
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        self.assertTrue(json.loads(self.body_of(wire))["ok"])
        self.assertEqual(observation["snapshot"]["peak"]["retained_bytes"], exact)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_debt(observation)
        self.assert_typed_budget_failure(
            "retained_bytes", exact - 1, override=("concurrent", exact - 1)
        )

    # ------------------------------------------------------------- deadlines

    def assert_deadline_phase(self, clock, expected_phase, **replacements):
        with self.recorded_ledgers() as ledgers, self.patched(**replacements):
            wire, observation = self.exchange(
                profile=self.profile(deadline_ms=10), clock=clock
            )
        self.assertTrue(
            wire.startswith(b"HTTP/1.0 503 Service Unavailable\r\n"), wire[:64]
        )
        self.assertNotIn(b"HTTP/1.0 200 OK", wire)
        self.assertNotIn(b"codefire_remote_doctor_report", wire)
        payload = json.loads(self.body_of(wire))
        self.assertEqual(payload["kind"], "request_deadline_exceeded")
        self.assertEqual(payload["exit_code"], 60)
        self.assertEqual(payload["diagnostics"][0]["phase"], expected_phase)
        self.assertEqual(payload["diagnostics"][0]["limit_ms"], 10)
        self.assertEqual(
            observation["snapshot"]["terminal_reason"], "deadline_exceeded"
        )
        self.assertEqual(observation["deadline"]["expired_phase"], expected_phase)
        self.assert_single_request_ledger(ledgers)
        self.assert_no_debt(observation)

    def test_deadline_expiry_at_the_doctor_route_entry_is_typed(self):
        self.assert_deadline_phase(
            StartThenExpireClock(0.010), "doctor project gate"
        )

    def test_deadline_expiry_inside_doctor_authority_discovery_is_typed(self):
        clock = InjectedClock()
        original = self.globals["accounted_sorted_json_paths"]
        calls = []

        def expiring_paths(*args, **kwargs):
            result = original(*args, **kwargs)
            calls.append(1)
            if len(calls) == 1:
                clock.now = 0.011
            return result

        self.assert_deadline_phase(
            clock,
            "doctor merge request authority",
            accounted_sorted_json_paths=expiring_paths,
        )
        self.assertTrue(calls)

    def test_deadline_expiry_inside_doctor_object_scan_is_typed(self):
        clock = InjectedClock()
        session_type = self.globals["SealedCommitValidationSession"]
        original = session_type.scan_object_hash_errors

        def expiring_scan(session):
            result = original(session)
            clock.now = 0.011
            return result

        session_type.scan_object_hash_errors = expiring_scan
        try:
            self.assert_deadline_phase(clock, "doctor object reference walk")
        finally:
            session_type.scan_object_hash_errors = original

    def test_deadline_expiry_after_doctor_payload_accounting_cleans_up(self):
        """A late expiry must still release the accounted payload owner."""
        clock = InjectedClock()
        original = self.globals["_account_doctor_payload"]
        accounted = []

        def expiring_account(*args, **kwargs):
            payload = original(*args, **kwargs)
            accounted.append(payload)
            clock.now = 0.011
            return payload

        self.assert_deadline_phase(
            clock,
            "doctor response encoding",
            _account_doctor_payload=expiring_account,
        )
        self.assertEqual(len(accounted), 1)
        self.assertTrue(accounted[0]._closed)
        self.assertEqual(accounted[0]._retained_bytes, 0)
        self.assertEqual(len(accounted[0]), 0)

    def test_disabled_deadline_never_reads_the_clock_on_the_doctor_route(self):
        def reject_clock_read():
            raise AssertionError("disabled Doctor deadline read its clock")

        wire, observation = self.exchange(
            profile=self.profile(deadline_ms=None), clock=reject_clock_read
        )
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"))
        self.assertEqual(observation["deadline"]["poll_count"], 0)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_debt(observation)

    # -------------------------------------------------- local compatibility

    def test_direct_compatibility_path_keeps_its_intentional_local_ledger(self):
        with self.recorded_ledgers() as ledgers, self.recorded_doctor_payloads() as owners:
            payload = self.direct_payload()
            self.assertEqual(len(ledgers), 1)
            ledger = ledgers[0]
            self.assertEqual(ledger.command, "remote doctor")
            self.assertEqual(ledger.command_kind, "local_graph")
            self.assertEqual(
                ledger.profile.limit("retained_bytes"), 1024 * 1024 * 1024
            )
            self.assertEqual(len(owners), 1)
            self.assertIs(owners[0]["ledger"], ledger)
            self.assertTrue(owners[0]["owns_ledger"])
            self.assertTrue(payload["ok"])
            self.assertIsNone(ledger.terminal_reason)
            self.assertGreater(ledger.current.get("retained_bytes", 0), 0)
            payload.close()
        self.assertEqual(ledger.terminal_reason, "complete")
        self.assertEqual(ledger.current.get("retained_bytes", 0), 0)
        self.assertEqual(ledger.current.get("temporary_bytes", 0), 0)
        self.assertFalse(ledger.has_open_permits())

    def test_handle_http_doctor_without_a_request_ledger_stays_compatible(self):
        with self.recorded_ledgers() as ledgers:
            status, payload = self.ns["handle_http_doctor"](
                self.storage_root, "org", "app"
            )
        self.assertEqual(status, 200)
        self.assertEqual(len(ledgers), 1)
        self.assertEqual(ledgers[0].command, "remote doctor")
        try:
            self.assertTrue(payload["ok"])
            self.assertEqual(payload["project"], self.url)
        finally:
            payload.close()
        self.assertEqual(ledgers[0].terminal_reason, "complete")

    def test_supplied_request_ledger_is_never_replaced_by_a_nested_ledger(self):
        """A supplied ledger is used as-is, including on typed failure."""
        request_ledger = self.ns["DecisionWorkLedger"](
            self.ns["DecisionWorkProfile"].http_v1({"object_records": 1}),
            "http request",
            command_kind="http_request",
        )
        with self.recorded_ledgers() as ledgers:
            with self.assertRaises(self.ns["ResourceBudgetExceeded"]) as caught:
                self.ns["handle_http_doctor"](
                    self.storage_root, "org", "app", False, request_ledger
                )
        self.assertEqual(ledgers, [])
        self.assertEqual(caught.exception.diagnostic()["dimension"], "object_records")
        self.assertEqual(request_ledger.terminal_reason, "budget_exceeded")
        self.assertTrue(
            all(value == 0 for value in request_ledger.current.values()),
            dict(request_ledger.current),
        )
        self.assertFalse(request_ledger.has_open_permits())

    def test_supplied_request_ledger_keeps_pre_existing_retained_scalars(self):
        """Doctor failure unwinds only its own concurrent acquisitions."""
        request_ledger = self.ns["DecisionWorkLedger"](
            self.ns["DecisionWorkProfile"].http_v1({"object_records": 1}),
            "http request",
            command_kind="http_request",
        )
        request_ledger.acquire("retained_bytes", 512, "request_headers")
        with self.assertRaises(self.ns["ResourceBudgetExceeded"]):
            self.ns["handle_http_doctor"](
                self.storage_root, "org", "app", False, request_ledger
            )
        self.assertEqual(request_ledger.current.get("retained_bytes", 0), 512)
        self.assertEqual(request_ledger.current.get("temporary_bytes", 0), 0)
        self.assertFalse(request_ledger.has_open_permits())

    # -------------------------------------------- corrupted object diagnostics

    def corrupt_one_object_record(self):
        """Break one committed object record's hash the way operators find it.

        ``scan_object_hash_errors`` then reports this record as a ``Path``, which
        is the input the accounting path used to charge before projection.
        """
        commits = sorted((self.project_root / "objects" / "commits").glob("*.json"))
        self.assertTrue(commits)
        target = commits[0]
        record = json.loads(target.read_text(encoding="utf-8"))
        record["hash"] = "bad-hash"
        target.write_text(
            json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        return target

    @contextlib.contextmanager
    def guarded_json_accounting(self):
        """Fail if production ever charges a ``Path`` through canonical sizing.

        Pre-fix this raised ``TypeError: PosixPath is not JSON serializable``
        from ``canonical_json_size([bad_paths, ...])``; the guard turns that
        untyped escape into a precise, always-visible assertion.
        """
        original = self.globals["canonical_json_size"]
        charged = []

        def guarded(value):
            stack = [value]
            while stack:
                item = stack.pop()
                if isinstance(item, Path):
                    raise AssertionError(
                        f"canonical_json_size charged a Path value: {item!r}"
                    )
                if isinstance(item, (list, tuple)):
                    stack.extend(item)
                elif isinstance(item, dict):
                    stack.extend(item)
                    stack.extend(item.values())
            charged.append(value)
            return original(value)

        with self.patched(canonical_json_size=guarded):
            yield charged

    def assert_corrupt_object_report(self, report, target):
        """One structured unhealthy report with stable string subjects."""
        self.assertIs(report["ok"], False)
        self.assertEqual(report["object_hash_errors"], 1)
        self.assertEqual(report["bad_objects"], [str(target)])
        # Stable, JSON-serializable string projections - never Path values.
        for name in report["bad_objects"]:
            self.assertIsInstance(name, str)
        self.assertEqual(
            json.loads(json.dumps(report["bad_objects"])), [str(target)]
        )
        issues = report["issues"]
        # Typed diagnostics keep object-record precedence over sealed refs.
        self.assertEqual(issues[0]["kind"], "remote_object_invalid")
        self.assertEqual(issues[0]["subject"], str(target))
        self.assertEqual(
            issues[0]["message"], f"invalid object record: {target}"
        )
        kinds = [issue["kind"] for issue in issues]
        self.assertEqual(kinds.count("remote_object_invalid"), 1)
        self.assertIn("remote_sealed_commit_invalid", kinds)
        first_sealed = kinds.index("remote_sealed_commit_invalid")
        self.assertNotIn("remote_object_invalid", kinds[first_sealed:])
        self.assertGreater(report["checked_objects"], 0)

    def test_corrupted_object_record_is_a_typed_direct_report_not_an_exception(self):
        target = self.corrupt_one_object_record()
        with self.recorded_ledgers() as ledgers, self.guarded_json_accounting() as charged:
            try:
                payload = self.direct_payload()
            except TypeError as exc:  # the reproduced defect
                self.fail(f"corrupted-object Doctor raised an untyped error: {exc}")
            try:
                body = self.ns["canonical_json"](payload)
                report = dict(payload)
            finally:
                payload.close()
        self.assert_corrupt_object_report(report, target)
        self.assertEqual(json.loads(body)["bad_objects"], [str(target)])
        # The charged diagnostic projection IS the retained/published list.
        diagnostics = [
            value
            for value in charged
            if isinstance(value, list) and value and value[0] == [str(target)]
        ]
        self.assertEqual(len(diagnostics), 1)
        self.assertEqual(len(diagnostics[0]), 4)
        self.assertIs(diagnostics[0][0], report["bad_objects"])
        # Zero-debt cleanup on the deliberately local direct ledger.
        ledger = ledgers[0]
        self.assertEqual(ledger.command, "remote doctor")
        self.assertEqual(ledger.terminal_reason, "complete")
        self.assertTrue(
            all(value == 0 for value in ledger.current.values()),
            dict(ledger.current),
        )
        self.assertFalse(ledger.has_open_permits())

    def test_corrupted_object_record_is_a_typed_http_full_mode_report(self):
        target = self.corrupt_one_object_record()
        with self.recorded_ledgers() as ledgers, self.guarded_json_accounting():
            wire, observation = self.exchange()
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"), wire[:64])
        self.assertNotIn(b"Traceback", wire)
        self.assertNotIn(b"PosixPath", wire)
        ledger = self.assert_single_request_ledger(ledgers)
        self.assert_http_request_budget(ledger)
        body = self.body_of(wire)
        route_payload = json.loads(body)
        self.assert_corrupt_object_report(route_payload, target)
        # Byte-equivalent with the direct path, so keys, order and issues match.
        direct = self.direct_payload()
        try:
            self.assertEqual(body, self.ns["canonical_json"](direct))
            expected_issues = list(direct["issues"])
        finally:
            direct.close()
        self.assertEqual(route_payload["issues"], expected_issues)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assertGreater(observation["deadline"]["poll_count"], 0)
        self.assertIsNone(observation["deadline"]["expired_phase"])
        self.assert_no_debt(observation)

    def test_corrupted_object_retained_limit_exact_passes_and_minus_one_is_typed(self):
        """Exact/-1 retained accounting for the non-empty diagnostic."""
        target = self.corrupt_one_object_record()
        exact = self.observed_bounds()["retained_bytes"]
        self.assertGreater(exact, 1)
        wire, observation = self.exchange(
            profile=self.profile({"retained_bytes": ("concurrent", exact)})
        )
        self.assertTrue(wire.startswith(b"HTTP/1.0 200 OK\r\n"), wire[:64])
        self.assert_corrupt_object_report(json.loads(self.body_of(wire)), target)
        self.assertEqual(observation["snapshot"]["peak"]["retained_bytes"], exact)
        self.assertEqual(observation["snapshot"]["terminal_reason"], "complete")
        self.assert_no_debt(observation)
        self.assert_typed_budget_failure(
            "retained_bytes", exact - 1, override=("concurrent", exact - 1)
        )


if __name__ == "__main__":
    unittest.main()
