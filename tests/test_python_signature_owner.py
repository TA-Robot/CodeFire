import json
import gc
import runpy
import sys
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NAMESPACE = runpy.run_path(str(ROOT / "codefire"))


class PhaseDeadline:
    def __init__(self, phase, allowed):
        self.phase = phase
        self.allowed = allowed
        self.matches = 0

    def check(self, phase):
        if phase != self.phase:
            return
        self.matches += 1
        if self.matches > self.allowed:
            raise NAMESPACE["RequestDeadlineExceeded"](phase, 1)


class AccountedCommitSignaturePolicyTests(unittest.TestCase):
    KEY = "signing-secret"
    SIGNER = "alice"

    def policy_raw(self, *, history=True, keys=None, required=True):
        return json.dumps(
            {
                "commit_signatures": {
                    "required": required,
                    "require_history": history,
                    "keys": {self.SIGNER: self.KEY} if keys is None else keys,
                }
            },
            separators=(",", ":"),
        ).encode("utf-8")

    def signed(self, parents=(), *, label="node", signer=None, key=None, key_id=None):
        payload = {
            "type": "commit",
            "version": 1,
            "parents": list(parents),
            "label": label,
        }
        return NAMESPACE["sign_commit_payload"](
            payload,
            signer or self.SIGNER,
            key or self.KEY,
            key_id,
        )

    def loader(self, graph, calls=None):
        def load(object_id):
            if calls is not None:
                calls.append(object_id)
            payload = graph[object_id]
            return payload, NAMESPACE["canonical_json_size"](payload)

        return load

    def ledger(self, overrides=None):
        profile = NAMESPACE["DecisionWorkProfile"].local_v1(overrides)
        return NAMESPACE["DecisionWorkLedger"](profile, "signature fixture")

    def assert_released(self, ledger):
        self.assertEqual(ledger.current.get("retained_bytes", 0), 0)
        self.assertEqual(ledger.current.get("temporary_bytes", 0), 0)
        self.assertFalse(ledger.has_open_permits())

    def run_owner(self, raw, graph, head, *, ledger=None, calls=None, deadline=None):
        ledger = ledger or self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            raw,
            ledger,
            self.loader(graph, calls),
            deadline,
        )
        try:
            owner.verify(head)
        finally:
            owner.close()
        self.assert_released(ledger)
        return ledger

    def test_head_and_history_required_preserve_legacy_unsigned_rules(self):
        unsigned_leaf = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "label": "unsigned-leaf",
        }
        unsigned_nonleaf = {
            "type": "commit",
            "version": 1,
            "parents": ["CF-COMMIT-leaf"],
            "label": "unsigned-nonleaf",
        }
        unsigned_head = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "label": "unsigned-head",
        }
        signed_head = self.signed(["CF-COMMIT-parent"], label="head")

        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(history=True),
            ledger,
            self.loader({"CF-COMMIT-head": unsigned_head}),
        )
        with self.assertRaisesRegex(NAMESPACE["CFError"], "signature is required"):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assert_released(ledger)

        no_history_calls = []
        self.run_owner(
            self.policy_raw(history=False),
            {
                "CF-COMMIT-head": signed_head,
                "CF-COMMIT-parent": unsigned_nonleaf,
                "CF-COMMIT-leaf": unsigned_leaf,
            },
            "CF-COMMIT-head",
            calls=no_history_calls,
        )
        self.assertEqual(no_history_calls, ["CF-COMMIT-head"])

        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(history=True),
            ledger,
            self.loader(
                {
                    "CF-COMMIT-head": signed_head,
                    "CF-COMMIT-parent": unsigned_nonleaf,
                    "CF-COMMIT-leaf": unsigned_leaf,
                }
            ),
        )
        with self.assertRaisesRegex(NAMESPACE["CFError"], "signature is required"):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assert_released(ledger)

        leaf_head = self.signed(["CF-COMMIT-leaf"], label="leaf-head")
        self.run_owner(
            self.policy_raw(history=True),
            {"CF-COMMIT-head": leaf_head, "CF-COMMIT-leaf": unsigned_leaf},
            "CF-COMMIT-head",
        )

    def test_one_three_and_ten_node_curves_charge_actual_work(self):
        observations = []
        for count in (1, 3, 10):
            graph = {}
            parent = None
            expected_hashed = 0
            for index in range(count):
                object_id = f"CF-COMMIT-{index}"
                commit = self.signed(
                    [] if parent is None else [parent], label=f"node-{index}"
                )
                graph[object_id] = commit
                expected_hashed += len(
                    NAMESPACE["commit_signature_input"](
                        commit, commit["signature"]
                    )
                )
                parent = object_id
            ledger = self.run_owner(self.policy_raw(), graph, parent)
            self.assertEqual(ledger.cumulative.get("graph_nodes"), count)
            self.assertEqual(ledger.cumulative.get("graph_edges", 0), count - 1)
            self.assertEqual(ledger.counters.get("bytes_hashed"), expected_hashed)
            observations.append(expected_hashed)
        self.assertLess(observations[1], observations[0] * 4)
        self.assertLess(observations[2], observations[0] * 12)

    def test_thousand_node_chain_retires_scalar_free_transitions_immediately(self):
        graph = {}
        parent = None
        for index in range(1000):
            object_id = f"CF-COMMIT-{index}"
            graph[object_id] = self.signed(
                [] if parent is None else [parent],
                label=f"retire-{index}",
            )
            parent = object_id
        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(),
            ledger,
            self.loader(graph),
        )
        owner.verify(parent)

        chain_count = 0
        fully_released_count = 0
        fully_released_shallow = 0
        retained_upper = 0
        temporary_upper = 0
        previous = None
        transition = owner._transition_head
        while transition is not None:
            self.assertIs(transition._prev, previous)
            owns_retained = transition._retained_acquired and (
                not transition._retained_primary_released
                or not transition._retained_secondary_released
            )
            owns_temporary = (
                transition._temporary_acquired
                and not transition._temporary_released
            )
            if not owns_retained and not owns_temporary:
                fully_released_count += 1
                fully_released_shallow += sys.getsizeof(transition)
            if transition._retained_acquired:
                if not transition._retained_primary_released:
                    retained_upper += transition._retained_primary
                if not transition._retained_secondary_released:
                    retained_upper += transition._retained_secondary
            if owns_temporary:
                temporary_upper += transition._temporary
            chain_count += 1
            previous = transition
            transition = transition._next

        self.assertEqual(fully_released_count, 0)
        self.assertEqual(fully_released_shallow, 0)
        self.assertEqual(chain_count, 1002)
        self.assertEqual(
            ledger.current.get("retained_bytes", 0), retained_upper
        )
        self.assertEqual(
            ledger.current.get("temporary_bytes", 0), temporary_upper
        )
        owner.close()
        self.assert_released(ledger)

    def test_streaming_hmac_matches_canonical_json_for_nested_escaped_payload(self):
        commit = self.signed(label="quote:\" slash:\\ line:\n snowman:☃")
        commit["nested"] = {
            "z": [None, True, False, -12, 1.5, {"β": "火"}],
            "a": "\u0001",
        }
        commit = NAMESPACE["sign_commit_payload"](
            {key: value for key, value in commit.items() if key != "signature"},
            self.SIGNER,
            self.KEY,
        )
        ledger = self.run_owner(
            self.policy_raw(),
            {"CF-COMMIT-head": commit},
            "CF-COMMIT-head",
        )
        self.assertEqual(
            ledger.counters.get("bytes_hashed"),
            len(NAMESPACE["commit_signature_input"](commit, commit["signature"])),
        )

    def test_diamond_is_lifo_and_counts_revisited_edge_once(self):
        graph = {
            "CF-COMMIT-base": self.signed(label="base"),
            "CF-COMMIT-left": self.signed(["CF-COMMIT-base"], label="left"),
            "CF-COMMIT-right": self.signed(["CF-COMMIT-base"], label="right"),
            "CF-COMMIT-head": self.signed(
                ["CF-COMMIT-left", "CF-COMMIT-right"], label="head"
            ),
        }
        calls = []
        ledger = self.run_owner(
            self.policy_raw(), graph, "CF-COMMIT-head", calls=calls
        )
        self.assertEqual(
            calls,
            ["CF-COMMIT-head", "CF-COMMIT-left", "CF-COMMIT-base", "CF-COMMIT-right"],
        )
        self.assertEqual(ledger.cumulative.get("graph_nodes"), 4)
        self.assertEqual(ledger.cumulative.get("graph_edges"), 4)
        self.assertEqual(ledger.counters.get("revisits"), 1)

    def test_invalid_revoked_and_expired_key_keep_first_parent_error(self):
        invalid = self.signed(label="invalid", signer="invalid", key_id="invalid")
        invalid["signature"]["signature"] = "0" * 64
        revoked = self.signed(label="revoked", signer="revoked", key_id="revoked")
        expired = self.signed(label="expired", signer="expired", key_id="expired")
        graph = {
            "CF-COMMIT-invalid": invalid,
            "CF-COMMIT-revoked": revoked,
            "CF-COMMIT-expired": expired,
        }
        keys = {
            self.SIGNER: self.KEY,
            "invalid": self.KEY,
            "revoked": {"secret": self.KEY, "status": "revoked"},
            "expired": {"secret": self.KEY, "not_after": "2000-01-01T00:00:00Z"},
        }
        cases = [
            (["invalid", "revoked", "expired"], "invalid signature"),
            (["revoked", "expired", "invalid"], "key is revoked"),
            (["expired", "invalid", "revoked"], "key is expired"),
        ]
        for order, expected in cases:
            with self.subTest(order=order):
                head = self.signed(
                    [f"CF-COMMIT-{name}" for name in order], label="head"
                )
                case_graph = dict(graph)
                case_graph["CF-COMMIT-head"] = head
                ledger = self.ledger()
                owner = NAMESPACE["AccountedCommitSignaturePolicy"](
                    self.policy_raw(keys=keys), ledger, self.loader(case_graph)
                )
                with self.assertRaisesRegex(NAMESPACE["CFError"], expected):
                    owner.verify("CF-COMMIT-head")
                owner.close()
                self.assert_released(ledger)

    def test_policy_retained_and_hmac_temporary_upper_exact_and_minus_one(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw(required=False)
        retained_upper = Owner.policy_retained_upper(len(raw))
        exact = self.ledger({"retained_bytes": retained_upper})
        owner = Owner(raw, exact, lambda _object_id: self.fail("loader called"))
        owner.close()
        self.assert_released(exact)

        plus_one = self.ledger({"retained_bytes": retained_upper + 1})
        owner = Owner(raw, plus_one, lambda _object_id: self.fail("loader called"))
        owner.close()
        self.assert_released(plus_one)

        rejected = self.ledger({"retained_bytes": retained_upper - 1})
        with self.assertRaises(NAMESPACE["ResourceBudgetExceeded"]):
            Owner(raw, rejected, lambda _object_id: self.fail("loader called"))
        self.assert_released(rejected)

        commit = self.signed(label="temporary")
        commit_upper = NAMESPACE["canonical_json_size"](commit)
        hmac_upper = Owner.hmac_temporary_upper(
            commit_upper, len(self.policy_raw())
        )
        policy_temp = Owner.policy_temporary_upper(len(self.policy_raw()))
        self.assertGreater(hmac_upper, policy_temp)
        exact = self.ledger({"temporary_bytes": hmac_upper})
        self.run_owner(
            self.policy_raw(),
            {"CF-COMMIT-head": commit},
            "CF-COMMIT-head",
            ledger=exact,
        )
        self.run_owner(
            self.policy_raw(),
            {"CF-COMMIT-head": commit},
            "CF-COMMIT-head",
            ledger=self.ledger({"temporary_bytes": hmac_upper + 1}),
        )
        rejected = self.ledger({"temporary_bytes": hmac_upper - 1})
        owner = Owner(
            self.policy_raw(),
            rejected,
            self.loader({"CF-COMMIT-head": commit}),
        )
        with self.assertRaises(NAMESPACE["ResourceBudgetExceeded"]):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assert_released(rejected)

    def test_policy_ast_and_collection_upper_is_current_before_parse(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        cases = {
            "dense": json.dumps(
                {
                    "commit_signatures": {"required": False},
                    "extension": [{} for _ in range(10_000)],
                },
                separators=(",", ":"),
            ).encode("utf-8"),
            "deep": (
                '{"commit_signatures":{"required":false},"extension":'
                + "[" * 500
                + "0"
                + "]" * 500
                + "}"
            ).encode("utf-8"),
        }
        original_loads = Owner.__init__.__globals__["json"].loads
        for label, raw in cases.items():
            with self.subTest(label=label):
                ledger = self.ledger()
                observations = []

                def inspect_current(value):
                    observations.append(
                        (
                            ledger.current.get("retained_bytes", 0),
                            ledger.current.get("temporary_bytes", 0),
                        )
                    )
                    return original_loads(value)

                Owner.__init__.__globals__["json"].loads = inspect_current
                try:
                    owner = Owner(
                        raw,
                        ledger,
                        lambda _object_id: self.fail("loader called"),
                    )
                finally:
                    Owner.__init__.__globals__["json"].loads = original_loads
                self.assertEqual(
                    observations,
                    [
                        (
                            Owner.policy_retained_upper(len(raw)),
                            Owner.policy_temporary_upper(len(raw)),
                        )
                    ],
                )
                owner.close()
                self.assert_released(ledger)

    def test_dense_and_deep_policy_upper_exact_and_minus_one(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        cases = [
            json.dumps(
                {
                    "commit_signatures": {"required": False},
                    "extension": [{} for _ in range(10_000)],
                },
                separators=(",", ":"),
            ).encode("utf-8"),
            (
                '{"commit_signatures":{"required":false},"extension":'
                + "[" * 500
                + "0"
                + "]" * 500
                + "}"
            ).encode("utf-8"),
        ]
        for raw in cases:
            for dimension, upper in (
                ("retained_bytes", Owner.policy_retained_upper(len(raw))),
                ("temporary_bytes", Owner.policy_temporary_upper(len(raw))),
            ):
                with self.subTest(raw_bytes=len(raw), dimension=dimension):
                    exact = self.ledger({dimension: upper})
                    owner = Owner(
                        raw,
                        exact,
                        lambda _object_id: self.fail("loader called"),
                    )
                    owner.close()
                    self.assert_released(exact)

                    rejected = self.ledger({dimension: upper - 1})
                    with self.assertRaises(NAMESPACE["ResourceBudgetExceeded"]):
                        Owner(
                            raw,
                            rejected,
                            lambda _object_id: self.fail("loader called"),
                        )
                    self.assert_released(rejected)

    def test_policy_failure_tracebacks_drop_parser_payload_before_release(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        globals_ = Owner.__init__.__globals__
        original_loads = globals_["json"].loads
        original_size = globals_["canonical_json_size_up_to"]

        class Probe:
            pass

        for failure_site in ("parse", "size"):
            with self.subTest(failure_site=failure_site):
                ledger = self.ledger()
                references = []
                releases = []
                original_release = ledger.release

                def fail_loads(value):
                    probe = Probe()
                    references.append(weakref.ref(probe))
                    if failure_site == "parse":
                        raise NAMESPACE["CFError"]("injected policy parse failure")
                    return {
                        "commit_signatures": {"required": False},
                        "extension": probe,
                    }

                def fail_size(_value, _limit, _poll):
                    raise NAMESPACE["CFError"]("injected policy size failure")

                def observe_release(dimension, amount):
                    if not releases:
                        gc.collect()
                        self.assertEqual(len(references), 1)
                        self.assertIsNone(references[0]())
                    releases.append((dimension, amount))
                    return original_release(dimension, amount)

                globals_["json"].loads = fail_loads
                if failure_site == "size":
                    globals_["canonical_json_size_up_to"] = fail_size
                ledger.release = observe_release
                try:
                    with self.assertRaisesRegex(
                        NAMESPACE["CFError"], f"policy {failure_site} failure"
                    ):
                        Owner(
                            self.policy_raw(required=False),
                            ledger,
                            lambda _object_id: self.fail("loader called"),
                        )
                finally:
                    globals_["json"].loads = original_loads
                    globals_["canonical_json_size_up_to"] = original_size
                    ledger.release = original_release
                self.assertTrue(releases)
                self.assert_released(ledger)

    def test_active_frontier_frame_stays_owned_through_node_loader(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        commit = self.signed(label="active-frame")
        ledger = self.ledger()
        owner = None
        baseline = 0
        observations = []

        def load(object_id):
            observations.append(ledger.current.get("retained_bytes", 0))
            self.assertEqual(object_id, "CF-COMMIT-head")
            return commit, NAMESPACE["canonical_json_size"](commit)

        owner = Owner(raw, ledger, load)
        baseline = ledger.current.get("retained_bytes", 0)
        owner.verify("CF-COMMIT-head")
        self.assertGreaterEqual(
            observations[0],
            baseline + Owner.frontier_frame_bytes("CF-COMMIT-head"),
        )
        owner.close()
        self.assert_released(ledger)

    def test_frontier_and_seen_growth_hold_temporary_overlap_before_insert(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        commit = self.signed(label="collection-growth")
        ledger = self.ledger()
        owner = Owner(
            self.policy_raw(),
            ledger,
            self.loader({"CF-COMMIT-head": commit}),
        )
        original_append = owner._append_frontier
        original_insert = owner._insert_seen
        observations = []

        def inspect_append(frontier, object_id, is_head, transition):
            observations.append(
                ("frontier", ledger.current.get("temporary_bytes", 0))
            )
            return original_append(frontier, object_id, is_head, transition)

        def inspect_insert(seen, object_id):
            observations.append(("seen", ledger.current.get("temporary_bytes", 0)))
            return original_insert(seen, object_id)

        owner._append_frontier = inspect_append
        owner._insert_seen = inspect_insert
        owner.verify("CF-COMMIT-head")
        self.assertEqual([label for label, _ in observations], ["frontier", "seen"])
        self.assertTrue(all(amount > 0 for _, amount in observations))
        owner.close()
        self.assert_released(ledger)

    def test_frontier_and_seen_allocation_faults_rollback_and_retry_cleanup(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        commit = self.signed(label="allocation-fault")
        graph = {"CF-COMMIT-head": commit}

        ledger = self.ledger()
        owner = Owner(self.policy_raw(), ledger, self.loader(graph))
        baseline = ledger.current.get("retained_bytes", 0)

        def fail_append(_frontier, _object_id, _is_head, _transition):
            raise MemoryError("injected frontier allocation failure")

        owner._append_frontier = fail_append
        with self.assertRaisesRegex(MemoryError, "frontier allocation"):
            owner.verify("CF-COMMIT-head")
        self.assertEqual(owner._frontier_capacity, 0)
        self.assertEqual(owner._frontier, [])
        self.assertEqual(ledger.current.get("retained_bytes", 0), baseline)
        self.assertEqual(ledger.current.get("temporary_bytes", 0), 0)
        self.assertFalse(ledger.has_open_permits())
        owner.close()
        self.assert_released(ledger)

        ledger = self.ledger()
        owner = Owner(self.policy_raw(), ledger, self.loader(graph))

        def fail_insert(_seen, _object_id):
            raise MemoryError("injected seen allocation failure")

        owner._insert_seen = fail_insert
        with self.assertRaisesRegex(MemoryError, "seen allocation"):
            owner.verify("CF-COMMIT-head")
        self.assertEqual(owner._seen_capacity, 0)
        self.assertEqual(owner._seen, set())
        self.assertEqual(ledger.current.get("temporary_bytes", 0), 0)
        self.assertFalse(ledger.has_open_permits())
        owner.close()
        self.assert_released(ledger)

        ledger = self.ledger()
        owner = Owner(self.policy_raw(), ledger, self.loader(graph))
        owner._append_frontier = fail_append
        original_release = ledger.release
        failures = {"temporary": 1}

        def fail_rollback_release(dimension, amount):
            if dimension == "temporary_bytes" and failures["temporary"]:
                failures["temporary"] -= 1
                raise NAMESPACE["CFError"]("injected collection release failure")
            return original_release(dimension, amount)

        ledger.release = fail_rollback_release
        with self.assertRaisesRegex(NAMESPACE["CFError"], "collection release"):
            owner.verify("CF-COMMIT-head")
        self.assertEqual(owner._frontier_capacity, 0)
        self.assertEqual(owner._frontier, [])
        self.assertGreater(ledger.current.get("temporary_bytes", 0), 0)
        ledger.release = original_release
        owner.close()
        self.assert_released(ledger)

    def test_hmac_key_sort_is_deadline_polled(self):
        payload = {
            "type": "commit",
            "version": 1,
            "parents": [],
        }
        for index in range(300):
            payload[f"field-{index:04d}"] = index
        commit = NAMESPACE["sign_commit_payload"](
            payload, self.SIGNER, self.KEY
        )
        deadline = PhaseDeadline("commit signature HMAC", 330)
        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(),
            ledger,
            self.loader({"CF-COMMIT-head": commit}),
            deadline,
        )
        with self.assertRaises(NAMESPACE["RequestDeadlineExceeded"]):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assert_released(ledger)

    def test_hmac_failure_traceback_drops_serializer_payload_before_release(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        commit = self.signed(label="traceback")
        commit_upper = NAMESPACE["canonical_json_size"](commit)
        raw = self.policy_raw()
        hmac_temporary = Owner.hmac_temporary_upper(commit_upper, len(raw))
        ledger = self.ledger()
        owner = Owner(
            raw,
            ledger,
            self.loader({"CF-COMMIT-head": commit}),
        )
        original_stream = owner._stream_signature_input
        original_release = ledger.release
        references = []
        observed = []

        class Probe:
            pass

        def chained_failure(depth):
            probe = Probe()
            references.append(weakref.ref(probe))
            if depth == 0:
                raise NAMESPACE["CFError"]("injected HMAC chain leaf")
            try:
                chained_failure(depth - 1)
            except BaseException as inner:
                raise NAMESPACE["CFError"](
                    f"injected HMAC chain {depth}"
                ) from inner

        def fail_stream(_digest, _permit, _commit, _signature):
            chained_failure(70)

        def observe_release(dimension, amount):
            if dimension == "temporary_bytes" and amount == hmac_temporary:
                gc.collect()
                self.assertEqual(len(references), 71)
                self.assertTrue(all(reference() is None for reference in references))
                observed.append(True)
                original_release(dimension, amount)
                raise NAMESPACE["CFError"]("post-HMAC-release")
            return original_release(dimension, amount)

        owner._stream_signature_input = fail_stream
        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(NAMESPACE["CFError"], "HMAC chain 70"):
                owner.verify("CF-COMMIT-head")
        finally:
            owner._stream_signature_input = original_stream
            ledger.release = original_release
        self.assertEqual(observed, [True])
        owner.close()
        self.assert_released(ledger)

    def test_false_loader_upper_rejects_before_serializer_collections(self):
        signature = {
            "type": "commit_signature",
            "version": 1,
            "algorithm": "hmac-sha256",
            "signer": self.SIGNER,
            "key_id": self.SIGNER,
            "signed_at": "2026-07-21T00:00:00Z",
            "signature": "0" * 64,
        }
        dense = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "signature": signature,
        }
        for index in range(50_000):
            dense[f"field-{index:05d}"] = index
        nested = 0
        for _ in range(500):
            nested = [nested]
        deep = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "nested": nested,
            "signature": signature,
        }
        large_string = {
            "type": "commit",
            "version": 1,
            "parents": [],
            "text": "x" * (1024 * 1024),
            "signature": signature,
        }
        for label, commit in (
            ("dense", dense),
            ("deep", deep),
            ("string", large_string),
        ):
            with self.subTest(label=label):
                ledger = self.ledger()
                owner = NAMESPACE["AccountedCommitSignaturePolicy"](
                    self.policy_raw(),
                    ledger,
                    lambda _object_id, commit=commit: (commit, 64),
                )
                serializer_calls = []

                def forbidden_serializer(*_args):
                    serializer_calls.append(True)
                    self.fail("serializer allocated before loader upper rejection")

                owner._stream_signature_input = forbidden_serializer
                with self.assertRaisesRegex(
                    NAMESPACE["CFError"], "understated its canonical size upper"
                ):
                    owner.verify("CF-COMMIT-head")
                self.assertEqual(serializer_calls, [])
                owner.close()
                self.assert_released(ledger)

    def test_graph_node_and_edge_limits_are_exact(self):
        graph = {
            "CF-COMMIT-parent": self.signed(label="parent"),
            "CF-COMMIT-head": self.signed(["CF-COMMIT-parent"], label="head"),
        }
        self.run_owner(
            self.policy_raw(),
            graph,
            "CF-COMMIT-head",
            ledger=self.ledger({"graph_nodes": 2, "graph_edges": 1}),
        )
        self.run_owner(
            self.policy_raw(),
            graph,
            "CF-COMMIT-head",
            ledger=self.ledger({"graph_nodes": 3, "graph_edges": 2}),
        )
        for overrides in (
            {"graph_nodes": 1, "graph_edges": 1},
            {"graph_nodes": 2, "graph_edges": 0},
        ):
            with self.subTest(overrides=overrides):
                ledger = self.ledger(overrides)
                owner = NAMESPACE["AccountedCommitSignaturePolicy"](
                    self.policy_raw(), ledger, self.loader(graph)
                )
                with self.assertRaises(NAMESPACE["ResourceBudgetExceeded"]):
                    owner.verify("CF-COMMIT-head")
                owner.close()
                self.assert_released(ledger)

    def test_mid_history_and_hmac_deadlines_release_every_owner(self):
        graph = {}
        parent = None
        for index in range(10):
            object_id = f"CF-COMMIT-{index}"
            graph[object_id] = self.signed(
                [] if parent is None else [parent], label=f"node-{index}"
            )
            parent = object_id
        deadline = PhaseDeadline("commit signature history load", 5)
        calls = []
        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(), ledger, self.loader(graph, calls), deadline
        )
        with self.assertRaises(NAMESPACE["RequestDeadlineExceeded"]):
            owner.verify(parent)
        self.assertLess(len(calls), len(graph))
        owner.close()
        self.assert_released(ledger)

        large = self.signed(label="x" * (256 * 1024))
        expected = len(
            NAMESPACE["commit_signature_input"](large, large["signature"])
        )
        deadline = PhaseDeadline("commit signature HMAC", 1100)
        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(),
            ledger,
            self.loader({"CF-COMMIT-head": large}),
            deadline,
        )
        with self.assertRaises(NAMESPACE["RequestDeadlineExceeded"]):
            owner.verify("CF-COMMIT-head")
        hashed = ledger.counters.get("bytes_hashed", 0)
        self.assertGreater(hashed, 0)
        self.assertLess(hashed, expected)
        owner.close()
        self.assert_released(ledger)

    def test_cleanup_retry_foreign_ledger_and_zero_permits(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        commit = self.signed(label="cleanup")
        graph = {"CF-COMMIT-head": commit}
        ledger = self.ledger()
        owner = Owner(self.policy_raw(), ledger, self.loader(graph))
        owner.verify("CF-COMMIT-head")
        foreign = self.ledger()
        retained_before = ledger.current.get("retained_bytes", 0)
        with self.assertRaisesRegex(NAMESPACE["CFError"], "different ledger"):
            owner.close(foreign)
        self.assertEqual(ledger.current.get("retained_bytes", 0), retained_before)

        original_release = ledger.release
        failures = {"retained": 1}

        def fail_retained_once(dimension, amount):
            if dimension == "retained_bytes" and failures["retained"]:
                failures["retained"] -= 1
                raise NAMESPACE["CFError"]("injected retained release failure")
            return original_release(dimension, amount)

        ledger.release = fail_retained_once
        with self.assertRaisesRegex(NAMESPACE["CFError"], "injected retained"):
            owner.close()
        ledger.release = original_release
        owner.close()
        self.assert_released(ledger)

    def test_permit_consume_and_cancel_failures_keep_retry_ownership(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        commit = self.signed(label="permit-retry")
        graph = {"CF-COMMIT-head": commit}

        ledger = self.ledger()
        original_consume = ledger.consume_reserved
        original_cancel = ledger.cancel_permit
        state = {"consume_failed": False, "cancel_failures": 1}

        def fail_policy_consume(_permit, **_kwargs):
            state["consume_failed"] = True
            raise NAMESPACE["CFError"]("injected policy consume failure")

        def fail_cancel_once(permit):
            if state["consume_failed"] and state["cancel_failures"]:
                state["cancel_failures"] -= 1
                raise NAMESPACE["CFError"]("injected policy cancel failure")
            return original_cancel(permit)

        ledger.consume_reserved = fail_policy_consume
        ledger.cancel_permit = fail_cancel_once
        with self.assertRaisesRegex(NAMESPACE["CFError"], "policy cancel failure"):
            Owner(raw, ledger, self.loader(graph))
        ledger.consume_reserved = original_consume
        ledger.cancel_permit = original_cancel
        gc.collect()
        self.assert_released(ledger)

        for target in ("frontier", "seen", "hmac"):
            with self.subTest(target=target):
                ledger = self.ledger()
                owner = Owner(raw, ledger, self.loader(graph))
                original_consume = ledger.consume_reserved
                original_cancel = ledger.cancel_permit
                state = {"consume_failed": False, "cancel_failures": 1}

                def fail_target_consume(permit, **kwargs):
                    phase = kwargs.get("phase")
                    charges = kwargs.get("charges") or {}
                    should_fail = (
                        target == "frontier"
                        and phase == "graph_discovery"
                        and not charges
                    ) or (
                        target == "seen"
                        and phase == "graph_discovery"
                        and charges.get("graph_nodes") == 1
                    ) or (
                        target == "hmac"
                        and phase == "semantic_validation"
                    )
                    if should_fail:
                        state["consume_failed"] = True
                        raise NAMESPACE["CFError"](
                            f"injected {target} consume failure"
                        )
                    return original_consume(permit, **kwargs)

                def fail_target_cancel(permit):
                    if state["consume_failed"] and state["cancel_failures"]:
                        state["cancel_failures"] -= 1
                        raise NAMESPACE["CFError"](
                            f"injected {target} cancel failure"
                        )
                    return original_cancel(permit)

                ledger.consume_reserved = fail_target_consume
                ledger.cancel_permit = fail_target_cancel
                with self.assertRaisesRegex(
                    NAMESPACE["CFError"], f"{target} cancel failure"
                ):
                    owner.verify("CF-COMMIT-head")
                self.assertTrue(ledger.has_open_permits())
                slot = (
                    "_hmac_work_permit"
                    if target == "hmac"
                    else "_collection_permit"
                )
                self.assertIsNotNone(getattr(owner, slot))
                ledger.consume_reserved = original_consume
                ledger.cancel_permit = original_cancel
                owner.close()
                self.assert_released(ledger)

        ledger = self.ledger()
        owner = Owner(self.policy_raw(), ledger, self.loader(graph))
        original_release = ledger.release
        failures = {"temporary": 1}

        def fail_temporary_once(dimension, amount):
            if dimension == "temporary_bytes" and failures["temporary"]:
                failures["temporary"] -= 1
                raise NAMESPACE["CFError"]("injected temporary release failure")
            return original_release(dimension, amount)

        ledger.release = fail_temporary_once
        with self.assertRaisesRegex(NAMESPACE["CFError"], "injected temporary"):
            owner.verify("CF-COMMIT-head")
        self.assertGreater(ledger.current.get("temporary_bytes", 0), 0)
        ledger.release = original_release
        owner.close()
        self.assert_released(ledger)

    def test_original_consume_then_raise_reconciles_every_permit_path(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        commit = self.signed(label="post-consume")
        graph = {"CF-COMMIT-head": commit}

        ledger = self.ledger()
        original_consume = ledger.consume_reserved
        injected = []

        def post_policy_consume(permit, **kwargs):
            original_consume(permit, **kwargs)
            if kwargs.get("phase") == "validation" and not injected:
                injected.append("policy")
                raise NAMESPACE["CFError"]("post-policy-consume")

        ledger.consume_reserved = post_policy_consume
        owner = Owner(raw, ledger, self.loader(graph))
        owner.close()
        self.assertEqual(injected, ["policy"])
        self.assert_released(ledger)

        def matches(target, kwargs):
            phase = kwargs.get("phase")
            charges = kwargs.get("charges") or {}
            acquires = kwargs.get("acquires") or {}
            counters = kwargs.get("counters") or {}
            if target == "frontier":
                return phase == "graph_discovery" and counters.get(
                    "frontier_insertion_attempts"
                ) == 1
            if target == "seen":
                return charges.get("graph_nodes") == 1
            if target == "hmac_storage":
                return phase == "semantic_validation" and bool(acquires)
            if target == "hmac_counter":
                return counters.get("semantic_validations") == 1
            return bool(charges.get("aggregate_bytes"))

        for target in (
            "frontier",
            "seen",
            "hmac_storage",
            "hmac_counter",
            "hmac_chunk",
        ):
            with self.subTest(target=target):
                ledger = self.ledger()
                owner = Owner(raw, ledger, self.loader(graph))
                original_consume = ledger.consume_reserved
                injected = []

                def post_consume(permit, **kwargs):
                    original_consume(permit, **kwargs)
                    if not injected and matches(target, kwargs):
                        injected.append(target)
                        raise NAMESPACE["CFError"](f"post-{target}-consume")

                ledger.consume_reserved = post_consume
                owner.verify("CF-COMMIT-head")
                owner.close()
                self.assertEqual(injected, [target])
                self.assert_released(ledger)

    def test_post_consume_memory_error_is_recovered_by_transition_scalar_owner(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        Transition = NAMESPACE["_StablePermitTransition"]
        raw = self.policy_raw()
        commit = self.signed(label="durable-scalar")
        graph = {"CF-COMMIT-head": commit}
        original_consume = Transition.consume

        def matches(target, actual):
            phase = actual.get("phase")
            charges = actual.get("charges") or {}
            acquires = actual.get("acquires") or {}
            counters = actual.get("counters") or {}
            if target == "policy":
                return phase == "validation"
            if target == "frontier":
                return counters.get("frontier_insertion_attempts") == 1
            if target == "seen":
                return charges.get("graph_nodes") == 1
            return phase == "semantic_validation" and bool(acquires)

        for target in ("policy", "frontier", "seen", "hmac_storage"):
            with self.subTest(target=target):
                ledger = self.ledger()
                owner = None
                baseline = 0
                injected = []

                def post_consume(transition, **actual):
                    original_consume(transition, **actual)
                    if not injected and matches(target, actual):
                        injected.append(target)
                        raise MemoryError(f"post-{target}-scalar handoff")

                Transition.consume = post_consume
                try:
                    if target == "policy":
                        with self.assertRaisesRegex(MemoryError, "scalar handoff"):
                            Owner(raw, ledger, self.loader(graph))
                    else:
                        Transition.consume = original_consume
                        owner = Owner(raw, ledger, self.loader(graph))
                        baseline = ledger.current.get("retained_bytes", 0)
                        Transition.consume = post_consume
                        with self.assertRaisesRegex(MemoryError, "scalar handoff"):
                            owner.verify("CF-COMMIT-head")
                        if target == "frontier":
                            frame = Owner.frontier_frame_bytes("CF-COMMIT-head")
                            growth = Owner.INITIAL_CAPACITY * Owner.FRONTIER_SLOT_BYTES
                            scratch = Owner.INITIAL_CAPACITY * Owner.FRONTIER_SLOT_BYTES
                            self.assertEqual(
                                ledger.current.get("retained_bytes", 0),
                                baseline + frame + growth,
                            )
                            self.assertEqual(
                                ledger.current.get("temporary_bytes", 0), scratch
                            )
                        owner.close()
                finally:
                    Transition.consume = original_consume
                self.assertEqual(injected, [target])
                self.assert_released(ledger)

    def test_release_state_does_not_depend_on_fallible_integer_subtraction(self):
        class RaiseOnSubtract(int):
            def __sub__(self, _other):
                raise MemoryError("legacy scalar subtraction")

        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(),
            ledger,
            self.loader({"CF-COMMIT-head": self.signed(label="fixed-release")}),
        )
        # This recreates the removed aggregate-scalar failure surface.  The
        # durable transition chain must never read either legacy-named value.
        owner._retained_bytes = RaiseOnSubtract(
            ledger.current.get("retained_bytes", 0)
        )
        owner._temporary_bytes = RaiseOnSubtract(
            ledger.current.get("temporary_bytes", 0)
        )
        original_release = ledger.release

        def post_release(dimension, amount):
            original_release(dimension, amount)
            raise MemoryError(f"post-{dimension}-release")

        ledger.release = post_release
        owner.verify("CF-COMMIT-head")
        owner.close()
        owner.close()
        self.assert_released(ledger)

    def test_post_registration_fault_keeps_every_caller_owned_permit_recoverable(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        commit = self.signed(label="post-register")
        graph = {"CF-COMMIT-head": commit}

        def matches(target, reservation):
            phase = reservation["phase"]
            charges = reservation["charges"]
            acquires = reservation["acquires"]
            counters = reservation["counters"]
            if target == "policy":
                return phase == "validation"
            if target == "frontier":
                return counters.get("frontier_insertion_attempts") == 1
            if target == "seen":
                return charges.get("graph_nodes") == 1
            if target == "hmac_storage":
                return phase == "semantic_validation" and bool(acquires)
            return phase == "semantic_validation" and bool(
                charges.get("aggregate_bytes")
            )

        for target in (
            "policy",
            "frontier",
            "seen",
            "hmac_storage",
            "hmac_work",
        ):
            with self.subTest(target=target):
                ledger = self.ledger()
                original_register = ledger._register_reservation
                injected = []

                def post_register(permit, reservation):
                    original_register(permit, reservation)
                    if not injected and matches(target, reservation):
                        injected.append(target)
                        raise NAMESPACE["CFError"](f"post-{target}-register")

                if target == "policy":
                    ledger._register_reservation = post_register
                    owner = Owner(raw, ledger, self.loader(graph))
                else:
                    owner = Owner(raw, ledger, self.loader(graph))
                    ledger._register_reservation = post_register
                    owner.verify("CF-COMMIT-head")
                owner.close()
                self.assertEqual(injected, [target])
                self.assert_released(ledger)

    def test_pre_registration_failure_retires_every_published_transition(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        graph = {"CF-COMMIT-head": self.signed(label="pre-register-retire")}
        original_reserve_owned = Owner._reserve_owned_permit

        def matches(target, phase, charges, acquires, counters):
            if target == "policy":
                return phase == "validation"
            if target == "frontier":
                return counters.get("frontier_insertion_attempts") == 1
            if target == "seen":
                return charges.get("graph_nodes") == 1
            if target == "hmac_storage":
                return phase == "semantic_validation" and bool(acquires)
            return phase == "semantic_validation" and bool(
                charges.get("aggregate_bytes")
            )

        for target in (
            "policy",
            "frontier",
            "seen",
            "hmac_storage",
            "hmac_work",
        ):
            with self.subTest(target=target):
                ledger = self.ledger()
                original_register = ledger._register_reservation
                captured = []
                captured_owners = []

                def capture_transition(
                    owner, attribute, transition, phase, **upper
                ):
                    if matches(
                        target,
                        phase,
                        upper.get("charges") or {},
                        upper.get("acquires") or {},
                        upper.get("counters") or {},
                    ):
                        captured.append(transition)
                        captured_owners.append(owner)
                    return original_reserve_owned(
                        owner, attribute, transition, phase, **upper
                    )

                def fail_before_register(permit, reservation):
                    if matches(
                        target,
                        reservation["phase"],
                        reservation["charges"],
                        reservation["acquires"],
                        reservation["counters"],
                    ):
                        raise MemoryError(f"pre-{target}-registration")
                    return original_register(permit, reservation)

                Owner._reserve_owned_permit = capture_transition
                ledger._register_reservation = fail_before_register
                owner = None
                try:
                    if target == "policy":
                        with self.assertRaisesRegex(
                            MemoryError, f"pre-{target}-registration"
                        ):
                            Owner(raw, ledger, self.loader(graph))
                        owner = captured_owners[0]
                    else:
                        owner = Owner(raw, ledger, self.loader(graph))
                        with self.assertRaisesRegex(
                            MemoryError, f"pre-{target}-registration"
                        ):
                            owner.verify("CF-COMMIT-head")

                    self.assertEqual(len(captured), 1)
                    failed = captured[0]
                    self.assertFalse(failed._linked)
                    self.assertIsNone(failed._prev)
                    self.assertIsNone(failed._next)
                    transition = owner._transition_head
                    while transition is not None:
                        fully_released = (
                            transition._permit is None
                            and transition._retained_primary_released
                            and transition._retained_secondary_released
                            and transition._temporary_released
                        )
                        self.assertFalse(fully_released)
                        transition = transition._next
                    owner.close()
                finally:
                    Owner._reserve_owned_permit = original_reserve_owned
                    ledger._register_reservation = original_register
                self.assert_released(ledger)

    def test_original_cancel_then_raise_preserves_business_error_every_path(self):
        Owner = NAMESPACE["AccountedCommitSignaturePolicy"]
        raw = self.policy_raw()
        commit = self.signed(label="post-cancel")
        graph = {"CF-COMMIT-head": commit}

        def exercise(target):
            ledger = self.ledger()
            original_consume = ledger.consume_reserved
            original_cancel = ledger.cancel_permit
            cancel_faults = []

            def should_fail(kwargs):
                phase = kwargs.get("phase")
                charges = kwargs.get("charges") or {}
                acquires = kwargs.get("acquires") or {}
                counters = kwargs.get("counters") or {}
                if target == "policy":
                    return phase == "validation"
                if target == "frontier":
                    return counters.get("frontier_insertion_attempts") == 1
                if target == "seen":
                    return charges.get("graph_nodes") == 1
                if target == "hmac_storage":
                    return phase == "semantic_validation" and bool(acquires)
                if target == "hmac_counter":
                    return counters.get("semantic_validations") == 1
                return bool(charges.get("aggregate_bytes"))

            def business_consume(_permit, **kwargs):
                if should_fail(kwargs):
                    raise NAMESPACE["CFError"](f"{target} business error")
                return original_consume(_permit, **kwargs)

            def post_cancel(permit):
                original_cancel(permit)
                cancel_faults.append(target)
                raise NAMESPACE["CFError"](f"post-{target}-cancel")

            ledger.consume_reserved = business_consume
            ledger.cancel_permit = post_cancel
            if target == "policy":
                with self.assertRaisesRegex(
                    NAMESPACE["CFError"], f"{target} business error"
                ):
                    Owner(raw, ledger, self.loader(graph))
            else:
                owner = Owner(raw, ledger, self.loader(graph))
                with self.assertRaisesRegex(
                    NAMESPACE["CFError"], f"{target} business error"
                ):
                    owner.verify("CF-COMMIT-head")
                owner.close()
            self.assertTrue(cancel_faults)
            self.assert_released(ledger)

        for target in (
            "policy",
            "frontier",
            "seen",
            "hmac_storage",
            "hmac_counter",
            "hmac_chunk",
        ):
            with self.subTest(target=target):
                exercise(target)

    def test_original_release_then_raise_reconciles_and_keeps_business_error(self):
        invalid = self.signed(label="invalid-parent")
        invalid["signature"]["signature"] = "0" * 64
        graph = {
            "CF-COMMIT-invalid": invalid,
            "CF-COMMIT-pending": self.signed(label="pending-parent"),
            "CF-COMMIT-head": self.signed(
                ["CF-COMMIT-invalid", "CF-COMMIT-pending"], label="head"
            ),
        }
        ledger = self.ledger()
        original_release = ledger.release
        release_faults = []

        def post_release(dimension, amount):
            original_release(dimension, amount)
            release_faults.append((dimension, amount))
            raise NAMESPACE["CFError"](f"post-{dimension}-release")

        ledger.release = post_release
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(), ledger, self.loader(graph)
        )
        with self.assertRaisesRegex(NAMESPACE["CFError"], "invalid signature"):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assertGreater(len(release_faults), 6)
        self.assertTrue(any(dimension == "temporary_bytes" for dimension, _ in release_faults))
        self.assertTrue(any(dimension == "retained_bytes" for dimension, _ in release_faults))
        self.assert_released(ledger)

    def test_original_release_then_raise_preserves_collection_rollback_error(self):
        owner_class = NAMESPACE["AccountedCommitSignaturePolicy"]
        commit = self.signed(label="post-rollback-release")
        ledger = self.ledger()
        owner = owner_class(
            self.policy_raw(),
            ledger,
            self.loader({"CF-COMMIT-head": commit}),
        )
        original_release = ledger.release
        release_faults = []

        def fail_append(_frontier, _object_id, _is_head, _transition):
            raise MemoryError("frontier allocation business error")

        def post_release(dimension, amount):
            original_release(dimension, amount)
            release_faults.append((dimension, amount))
            raise NAMESPACE["CFError"](f"post-rollback-{dimension}")

        owner._append_frontier = fail_append
        ledger.release = post_release
        with self.assertRaisesRegex(MemoryError, "frontier allocation business error"):
            owner.verify("CF-COMMIT-head")
        owner.close()
        self.assertTrue(any(dimension == "temporary_bytes" for dimension, _ in release_faults))
        self.assertTrue(any(dimension == "retained_bytes" for dimension, _ in release_faults))
        self.assert_released(ledger)

    def test_disabled_policy_has_zero_graph_work(self):
        ledger = self.ledger()
        owner = NAMESPACE["AccountedCommitSignaturePolicy"](
            self.policy_raw(required=False),
            ledger,
            lambda _object_id: self.fail("disabled policy loaded a commit"),
        )
        owner.verify("CF-COMMIT-head")
        owner.close()
        self.assertEqual(ledger.cumulative.get("graph_nodes", 0), 0)
        self.assertEqual(ledger.cumulative.get("graph_edges", 0), 0)
        self.assert_released(ledger)


if __name__ == "__main__":
    unittest.main()
