import copy
import runpy
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonUploadSemanticOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.CFError = cls.namespace["CFError"]
        cls.Digest = cls.namespace["AccountedRemoteObjectSetDigest"]
        cls.Semantic = cls.namespace["AccountedHTTPUploadSemantic"]

    def ledger(self, label="upload semantic", overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            label,
            command_kind="http_request",
        )

    def records(self):
        records = []
        for generation, payload in [
            (2, {"text": "second", "nested": [2, True]}),
            (1, {"text": "first", "nested": [1, None]}),
        ]:
            records.append(
                {
                    "object_id": self.namespace["object_id"]("blob", payload),
                    "type": "blob",
                    "hash": self.namespace["object_digest"]("blob", payload),
                    "payload": payload,
                    "remote": {"generation": generation},
                }
            )
        return records

    def request(self):
        records = self.records()
        return {
            "actor": "alice",
            "branch": "local-main",
            "head": "CF-COMMIT-" + "1" * 64,
            "idempotency_key": "request-1",
            "objects": records,
            "historical_extension": {"preserve": ["all", "records"]},
        }

    def source_bytes(self, request):
        return self.namespace["canonical_json_size"](request)

    def bind_source(self, request, ledger, amount=None, upper=None):
        selected = self.source_bytes(request) if amount is None else amount
        selected_upper = selected if upper is None else upper
        scalar = self.namespace["prepare_http_upload_parser_scalar"](
            ledger, selected_upper
        )
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, selected)
        source = self.namespace["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        return source, selected

    def allocate_owner(self, source, ledger, deadline=None):
        return self.namespace["prepare_http_upload_semantic"](
            source,
            ledger,
            "CF-PROJECT-" + "2" * 32,
            "remote-main",
            deadline,
        )

    def expected_semantic(self, request):
        return {
            "stable_project_id": "CF-PROJECT-" + "2" * 32,
            "remote_branch": "remote-main",
            "local_branch": "local-main",
            "head": "CF-COMMIT-" + "1" * 64,
            "object_set_digest": self.namespace[
                "merge_request_object_set_digest"
            ](request["objects"]),
        }

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_two_stage_adopts_exact_scalar_and_detaches_private_inputs(self):
        request = self.request()
        ledger = self.ledger()
        source, parser_bytes = self.bind_source(request, ledger)
        before_adoption = dict(ledger.current)
        owner = self.allocate_owner(source, ledger)
        self.assertEqual(dict(ledger.current), before_adoption)
        self.assertIs(owner._source, source)
        self.assertIs(source._destination, owner)
        with self.assertRaisesRegex(self.CFError, "adopted"):
            source.close(ledger)

        owner.prepare(ledger)
        expected = self.expected_semantic(request)
        self.assertTrue(owner.matches_semantic(expected))
        self.assertFalse(hasattr(owner, "digest"))
        self.assertFalse(hasattr(owner, "object_set_digest"))
        semantic_identity = id(owner._semantic_view)
        copy_identity = id(owner._copy_bytes)
        self.assertEqual(
            list(owner._copy_bytes),
            [
                self.namespace["_serialize_object_record_core"](record)
                for record in sorted(
                    request["objects"], key=lambda record: record["object_id"]
                )
            ],
        )
        pinned_copies = list(owner._copy_bytes)

        owner.terminal_detach(ledger)
        self.assertTrue(source._closed)
        self.assertEqual(source._parser_canonical_bytes, 0)
        self.assertIsNone(owner._semantic)
        self.assertIsNone(owner._copy_bytes)
        self.assertEqual(id(owner._effect_semantic), semantic_identity)
        self.assertEqual(id(owner._effect_copy_bytes), copy_identity)
        self.assertTrue(owner.matches_semantic(expected))
        with self.assertRaises(TypeError):
            owner._effect_semantic["head"] = "CF-COMMIT-mutated"
        request["objects"][0]["remote"]["generation"] = 999
        request["objects"][0]["payload"]["nested"].append("ambient")
        self.assertEqual(list(owner._effect_copy_bytes), pinned_copies)
        self.assertGreater(ledger.current.get("retained_bytes", 0), 0)
        self.assertEqual(ledger.current.get("temporary_bytes", 0), 0)
        self.assertGreater(parser_bytes, 0)
        owner.close(ledger)
        self.assert_closed(ledger)

    def test_foreign_ledger_rejected_before_source_adoption(self):
        request = self.request()
        ledger = self.ledger("source")
        foreign = self.ledger("foreign")
        source, _amount = self.bind_source(request, ledger)
        scalar = source._scalar
        with self.assertRaisesRegex(self.CFError, "already adopted"):
            self.namespace["bind_http_upload_request_source"](
                request, ledger, scalar
            )
        with self.assertRaisesRegex(self.CFError, "different ledger"):
            self.allocate_owner(source, foreign)
        self.assertIsNone(source._destination)
        owner = self.allocate_owner(source, ledger)
        with self.assertRaisesRegex(self.CFError, "already adopted"):
            self.allocate_owner(source, ledger)
        owner.close(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

    def test_parser_token_acquire_callback_close_is_typed_and_leak_free(self):
        request = self.request()
        amount = self.source_bytes(request)
        ledger = self.ledger("parser token callback")
        token = self.namespace["prepare_http_upload_parser_scalar"](
            ledger, amount
        )
        consume = ledger.consume_reserved
        fired = False

        def callback_consume(*args, **kwargs):
            nonlocal fired
            if not fired:
                fired = True
                token.close(ledger)
                with self.assertRaisesRegex(self.CFError, "already active"):
                    token.acquire(ledger)
            return consume(*args, **kwargs)

        ledger.consume_reserved = callback_consume
        try:
            with self.assertRaisesRegex(self.CFError, "cancelled by close"):
                token.acquire(ledger)
        finally:
            ledger.consume_reserved = consume
        self.assertTrue(fired)
        self.assertTrue(token._closed)
        self.assert_closed(ledger)

    def test_parser_upper_reconciles_exact_with_retry_safe_shrink(self):
        request = self.request()
        exact = self.source_bytes(request)
        for post_effect in (False, True):
            with self.subTest(post_effect=post_effect):
                ledger = self.ledger(f"parser reconcile {post_effect}")
                upper = exact + 97
                token = self.namespace["prepare_http_upload_parser_scalar"](
                    ledger, upper
                )
                token.acquire(ledger)
                self.assertEqual(ledger.current["retained_bytes"], upper)
                with self.assertRaisesRegex(self.CFError, "not reconciled"):
                    self.namespace["bind_http_upload_request_source"](
                        request, ledger, token
                    )
                release = ledger.release
                fired = False

                def fail_shrink_once(dimension, amount):
                    nonlocal fired
                    if not fired and amount == upper - exact:
                        fired = True
                        if post_effect:
                            release(dimension, amount)
                        raise RuntimeError("parser reconcile release fault")
                    return release(dimension, amount)

                ledger.release = fail_shrink_once
                if post_effect:
                    token.reconcile_exact(ledger, exact)
                else:
                    with self.assertRaisesRegex(RuntimeError, "release fault"):
                        token.reconcile_exact(ledger, exact)
                    self.assertEqual(ledger.current["retained_bytes"], upper)
                    ledger.release = release
                    token.reconcile_exact(ledger, exact)
                ledger.release = release
                self.assertTrue(fired)
                self.assertEqual(ledger.current["retained_bytes"], exact)
                source = self.namespace["bind_http_upload_request_source"](
                    request, ledger, token
                )
                with self.assertRaisesRegex(self.CFError, "adopted"):
                    self.namespace["bind_http_upload_request_source"](
                        request, ledger, token
                    )
                source.close(ledger)
                self.assert_closed(ledger)

    def test_parser_reconcile_callback_close_is_typed_and_leak_free(self):
        request = self.request()
        exact = self.source_bytes(request)
        upper = exact + 31
        ledger = self.ledger("parser reconcile callback")
        token = self.namespace["prepare_http_upload_parser_scalar"](
            ledger, upper
        )
        token.acquire(ledger)
        release = ledger.release
        observations = []
        fired = False

        def callback_release(dimension, amount):
            nonlocal fired
            if not fired and amount == upper - exact:
                fired = True
                try:
                    token.reconcile_exact(ledger, exact)
                except BaseException as error:
                    observations.append(str(error))
                token.close(ledger)
            return release(dimension, amount)

        ledger.release = callback_release
        try:
            with self.assertRaisesRegex(self.CFError, "cancelled by close"):
                token.reconcile_exact(ledger, exact)
        finally:
            ledger.release = release
        self.assertTrue(fired)
        self.assertEqual(observations, [
            "HTTP upload parser reconciliation is already active"
        ])
        self.assertTrue(token._closed)
        self.assert_closed(ledger)

    def test_stage_like_record_bound_is_rejected_before_digest(self):
        request = self.request()
        record_bound = len(
            self.namespace["canonical_json"](
                self.namespace["canonical_remote_object_records"](
                    request["objects"]
                )
            )
        )
        self.assertLess(record_bound, self.source_bytes(request))
        ledger = self.ledger("false stage bound")
        source, _amount = self.bind_source(request, ledger, record_bound)
        owner = self.allocate_owner(source, ledger)
        with self.assertRaisesRegex(self.CFError, "full request"):
            owner.prepare(ledger)
        self.assertIsNone(owner._digest_owner)
        owner.close(ledger)
        self.assert_closed(ledger)

        request = self.request()
        ledger = self.ledger("false overbound parser scalar")
        source, _amount = self.bind_source(
            request, ledger, self.source_bytes(request) + 1
        )
        owner = self.allocate_owner(source, ledger)
        with self.assertRaisesRegex(self.CFError, "full request"):
            owner.prepare(ledger)
        owner.close(ledger)
        self.assert_closed(ledger)

    def test_destination_limits_precede_digest_and_exact_combined_accepts(self):
        request = self.request()
        source_bytes = self.source_bytes(request)
        count = len(request["objects"])
        probe_ledger = self.ledger("upper probe")
        source, _amount = self.bind_source(request, probe_ledger)
        probe = self.allocate_owner(source, probe_ledger)
        retained, temporary, work = probe._allocation_upper(
            source_bytes, count
        )
        probe.close(probe_ledger)
        self.assert_closed(probe_ledger)

        digest_retained = (
            self.Digest.ORDERED_LIST_BASE_BYTES
            + self.Digest.ORDERED_LIST_SLOT_BYTES * count
            + self.Digest.DIGEST_RETAINED_BYTES
        )
        digest_temporary = (
            self.Digest.ENCODER_BASE_BYTES
            + self.Digest.ENCODER_BYTES_PER_SOURCE_BYTE * source_bytes
        )
        cases = [
            (
                "retained_bytes",
                source_bytes + retained - 1,
                "concurrent",
            ),
            ("temporary_bytes", temporary - 1, "concurrent"),
            ("aggregate_bytes", work - 1, "cumulative"),
        ]
        for dimension, limit, measurement in cases:
            with self.subTest(dimension=dimension):
                ledger = self.ledger(
                    f"reject {dimension}",
                    {dimension: (measurement, limit)},
                )
                source, _amount = self.bind_source(request, ledger)
                owner = self.allocate_owner(source, ledger)
                with self.assertRaises(
                    self.namespace["ResourceBudgetExceeded"]
                ):
                    owner.prepare(ledger)
                self.assertIsNone(owner._digest_owner)
                owner.close(ledger)
                self.assert_closed(ledger)

        exact = self.ledger(
            "exact semantic upper",
            {
                "retained_bytes": (
                    "concurrent",
                    source_bytes + retained + digest_retained,
                ),
                "temporary_bytes": (
                    "concurrent",
                    temporary + digest_temporary,
                ),
                "aggregate_bytes": (
                    "cumulative",
                    work + source_bytes,
                ),
            },
        )
        source, _amount = self.bind_source(request, exact)
        owner = self.allocate_owner(source, exact)
        owner.prepare(exact)
        owner.terminal_detach(exact)
        owner.close(exact)
        self.assert_closed(exact)

    def test_terminal_rejects_identity_content_and_request_mutation(self):
        mutations = [
            lambda request: request.__setitem__("branch", "other-main"),
            lambda request: request["objects"].reverse(),
            lambda request: request["objects"].__setitem__(
                0, copy.deepcopy(request["objects"][0])
            ),
            lambda request: request["objects"][0]["payload"]["nested"].append(
                "changed"
            ),
            lambda request: request.__setitem__(
                "historical_extension", {"preserve": ["changed"]}
            ),
        ]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                request = self.request()
                ledger = self.ledger(f"mutation {index}")
                source, _amount = self.bind_source(request, ledger)
                owner = self.allocate_owner(source, ledger)
                owner.prepare(ledger)
                mutate(request)
                with self.assertRaisesRegex(
                    self.CFError, "changed|full request"
                ):
                    owner.terminal_detach(ledger)
                with self.assertRaisesRegex(self.CFError, "abort-only"):
                    owner.terminal_detach(ledger)
                owner.close(ledger)
                self.assert_closed(ledger)

    def test_prepare_callback_rejects_partial_surfaces_and_cancels(self):
        request = self.request()

        class Deadline:
            def __init__(self):
                self.owner = None
                self.armed = False
                self.errors = []

            def check(self, _phase):
                if not self.armed or self.owner is None:
                    return
                self.armed = False
                operations = [
                    lambda: self.owner.matches_semantic({}),
                    lambda: self.owner.prepare(ledger),
                    lambda: self.owner.terminal_detach(ledger),
                ]
                for operation in operations:
                    try:
                        operation()
                    except BaseException as error:
                        self.errors.append(str(error))
                self.owner.close(ledger)

        deadline = Deadline()
        ledger = self.ledger("prepare callback")
        source, _amount = self.bind_source(request, ledger)
        owner = self.allocate_owner(source, ledger, deadline)
        deadline.owner = owner
        deadline.armed = True
        with self.assertRaisesRegex(self.CFError, "cancelled by close"):
            owner.prepare(ledger)
        self.assertEqual(len(deadline.errors), 3)
        self.assertIn("active", deadline.errors[0])
        self.assertIn("already active", deadline.errors[1])
        self.assertIn("not prepared", deadline.errors[2])
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)

    def test_last_deadline_callback_mutation_is_rehashed_and_aborts(self):
        request = self.request()

        class Deadline:
            def __init__(self):
                self.armed = False

            def check(self, phase):
                if self.armed and phase == "upload semantic terminal":
                    self.armed = False
                    request["objects"][0]["remote"]["generation"] = 9

        deadline = Deadline()
        ledger = self.ledger("last callback mutation")
        source, _amount = self.bind_source(request, ledger)
        owner = self.allocate_owner(source, ledger, deadline)
        owner.prepare(ledger)
        deadline.armed = True
        with self.assertRaisesRegex(self.CFError, "changed"):
            owner.terminal_detach(ledger)
        self.assertTrue(owner._terminal_failed)
        owner.close(ledger)
        self.assert_closed(ledger)

    def test_terminal_reentry_close_returns_typed_cancellation(self):
        request = self.request()
        ledger = self.ledger("terminal callback")
        source, _amount = self.bind_source(request, ledger)
        owner = self.allocate_owner(source, ledger)
        owner.prepare(ledger)
        globals_map = self.namespace["prepare_http_upload_semantic"].__globals__
        canonical_json = globals_map["canonical_json"]
        fired = False
        observations = []

        def callback_canonical(value):
            nonlocal fired
            if not fired:
                fired = True
                for operation in [
                    lambda: owner.matches_semantic({}),
                    lambda: owner.terminal_detach(ledger),
                ]:
                    try:
                        operation()
                    except BaseException as error:
                        observations.append(str(error))
                owner.close(ledger)
            return canonical_json(value)

        globals_map["canonical_json"] = callback_canonical
        try:
            with self.assertRaisesRegex(self.CFError, "cancelled by close"):
                owner.terminal_detach(ledger)
        finally:
            globals_map["canonical_json"] = canonical_json
        self.assertEqual(len(observations), 2)
        self.assertTrue(all("active" in item for item in observations))
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)

    def test_repeated_digest_constructor_cleanup_remains_caller_owned(self):
        request = self.request()
        ledger = self.ledger("digest constructor cleanup")
        source, _amount = self.bind_source(request, ledger)
        owner = self.allocate_owner(source, ledger)
        heap_sort = self.Digest._heap_sort
        close_digest = self.Digest.close
        close_calls = 0

        def fail_sort(_digest_owner, _deadline):
            raise MemoryError("semantic digest construction fault")

        def fail_twice_close(digest_owner):
            nonlocal close_calls
            close_calls += 1
            if close_calls <= 2:
                raise RuntimeError("semantic digest cleanup fault")
            return close_digest(digest_owner)

        self.Digest._heap_sort = fail_sort
        self.Digest.close = fail_twice_close
        try:
            with self.assertRaisesRegex(RuntimeError, "digest cleanup fault"):
                owner.prepare(ledger)
            exact_child = owner._digest_owner
            self.assertIsNotNone(exact_child)
            with self.assertRaisesRegex(RuntimeError, "digest cleanup fault"):
                owner.close(ledger)
            self.assertIs(owner._digest_owner, exact_child)
            self.assertIs(owner._source, source)
        finally:
            self.Digest._heap_sort = heap_sort
            self.Digest.close = close_digest
        owner.close(ledger)
        self.assertEqual(close_calls, 2)
        self.assertTrue(exact_child._closed)
        self.assert_closed(ledger)

    def test_digest_reserve_and_consume_post_effect_faults_reconcile(self):
        for operation in ("reserve", "consume"):
            with self.subTest(operation=operation):
                request = self.request()
                ledger = self.ledger(f"digest post-effect {operation}")
                source, _amount = self.bind_source(request, ledger)
                owner = self.allocate_owner(source, ledger)
                fired = False
                if operation == "reserve":
                    original = ledger.reserve_into

                    def post_effect_reserve(permit, phase, **upper):
                        nonlocal fired
                        result = original(permit, phase, **upper)
                        if (
                            not fired
                            and upper.get("counters", {}).get(
                                "object_records"
                            )
                            == len(request["objects"])
                        ):
                            fired = True
                            raise RuntimeError("digest reserve post-effect fault")
                        return result

                    ledger.reserve_into = post_effect_reserve
                else:
                    original = ledger.consume_reserved

                    def post_effect_consume(permit, **actual):
                        nonlocal fired
                        reservation = ledger._reserved.get(permit, {})
                        is_digest = reservation.get("counters", {}).get(
                            "object_records"
                        ) == len(request["objects"])
                        result = original(permit, **actual)
                        if not fired and is_digest:
                            fired = True
                            raise RuntimeError("digest consume post-effect fault")
                        return result

                    ledger.consume_reserved = post_effect_consume
                try:
                    owner.prepare(ledger)
                finally:
                    if operation == "reserve":
                        ledger.reserve_into = original
                    else:
                        ledger.consume_reserved = original
                self.assertTrue(fired)
                owner.terminal_detach(ledger)
                owner.close(ledger)
                self.assert_closed(ledger)

    def test_parser_scalar_release_pre_and_post_faults_are_retry_safe(self):
        for post_effect in (False, True):
            with self.subTest(post_effect=post_effect):
                request = self.request()
                ledger = self.ledger(f"scalar release {post_effect}")
                source, source_bytes = self.bind_source(request, ledger)
                owner = self.allocate_owner(source, ledger)
                owner.prepare(ledger)
                release = ledger.release
                fired = False

                def fail_source_once(dimension, amount):
                    nonlocal fired
                    if (
                        dimension == "retained_bytes"
                        and amount == source_bytes
                        and not fired
                    ):
                        fired = True
                        self.assertIsNone(owner._record_refs)
                        self.assertIsNone(owner._sorted_refs)
                        self.assertIsNone(source._request)
                        self.assertIsNone(source._records)
                        if post_effect:
                            release(dimension, amount)
                        raise RuntimeError("parser scalar release fault")
                    return release(dimension, amount)

                ledger.release = fail_source_once
                if post_effect:
                    owner.terminal_detach(ledger)
                else:
                    with self.assertRaisesRegex(RuntimeError, "release fault"):
                        owner.terminal_detach(ledger)
                    self.assertTrue(owner._terminal_revalidated)
                    self.assertIs(owner._source, source)
                    self.assertIsNone(source._request)
                    self.assertEqual(
                        source._parser_canonical_bytes, source_bytes
                    )
                    self.assertIsNotNone(source._scalar)
                    ledger.release = release
                    owner.terminal_detach(ledger)
                ledger.release = release
                self.assertTrue(fired)
                self.assertTrue(source._closed)
                owner.close(ledger)
                self.assert_closed(ledger)

    def test_parser_release_callback_reentry_closes_with_cancellation(self):
        request = self.request()
        ledger = self.ledger("scalar release callback")
        source, source_bytes = self.bind_source(request, ledger)
        owner = self.allocate_owner(source, ledger)
        owner.prepare(ledger)
        release = ledger.release
        observations = []
        fired = False

        def callback_release(dimension, amount):
            nonlocal fired
            if (
                dimension == "retained_bytes"
                and amount == source_bytes
                and not fired
            ):
                fired = True
                for operation in [
                    lambda: owner.matches_semantic({}),
                    lambda: owner.terminal_detach(ledger),
                ]:
                    try:
                        operation()
                    except BaseException as error:
                        observations.append(str(error))
                owner.close(ledger)
            return release(dimension, amount)

        ledger.release = callback_release
        try:
            with self.assertRaisesRegex(self.CFError, "cancelled by close"):
                owner.terminal_detach(ledger)
        finally:
            ledger.release = release
        self.assertTrue(fired)
        self.assertEqual(len(observations), 2)
        self.assertTrue(all("active" in item for item in observations))
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
