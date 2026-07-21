import gc
import runpy
import sys
import unittest
import weakref
from collections import deque
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonAncestryOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Owner = cls.namespace["AccountedHTTPUploadAncestry"]
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]

    def ledger(self, overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            "HTTP upload ancestry",
            command_kind="http_request",
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    @staticmethod
    def graph_loader(graph, calls):
        def load(object_id):
            calls.append(object_id)
            value = graph[object_id]
            if isinstance(value, BaseException):
                raise value
            return value

        return load

    def test_bfs_full_walk_preserves_parent_and_late_error_order(self):
        graph = {
            "head": {"parents": ["left", "right"]},
            "left": {"parents": ["left-parent"]},
            "right": {"parents": ["right-parent"]},
            "left-parent": {"parents": []},
            "right-parent": {"parents": []},
        }
        calls = []
        ledger = self.ledger()
        owner = self.Owner(ledger, self.graph_loader(graph, calls))
        self.assertTrue(owner.contains("head", "head"))
        self.assertEqual(
            calls, ["head", "left", "right", "left-parent", "right-parent"]
        )
        self.assertEqual(ledger.cumulative.get("graph_nodes"), 5)
        self.assertEqual(ledger.cumulative.get("graph_edges"), 4)
        self.assertEqual(ledger.maximum.get("logical_depth"), 2)
        self.assert_closed(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "single-use"):
            owner.contains("head", "head")

        graph["right"] = RuntimeError("late right failure")
        calls = []
        failed_ledger = self.ledger()
        failed = self.Owner(failed_ledger, self.graph_loader(graph, calls))
        with self.assertRaisesRegex(RuntimeError, "late right failure"):
            failed.contains("head", "head")
        self.assertEqual(calls, ["head", "left", "right"])
        self.assert_closed(failed_ledger)

    def test_diamond_revisit_is_bfs_and_loads_each_commit_once(self):
        graph = {
            "head": {"parents": ["left", "right"]},
            "left": {"parents": ["shared"]},
            "right": {"parents": ["shared"]},
            "shared": {"parents": []},
        }
        calls = []
        ledger = self.ledger()
        owner = self.Owner(ledger, self.graph_loader(graph, calls))
        self.assertTrue(owner.contains("shared", "head"))
        self.assertEqual(calls, ["head", "left", "right", "shared"])
        self.assertEqual(ledger.counters.get("revisits"), 1)
        self.assertEqual(ledger.counters.get("memo_hits"), 1)
        self.assertEqual(ledger.counters.get("unique_nodes_discovered"), 4)
        self.assert_closed(ledger)

    def test_collection_and_graph_limits_precede_allocators_and_loaders(self):
        seen_allocator = self.Owner._allocate_seen
        frontier_allocator = self.Owner._allocate_frontier
        allocator_calls = []

        def observe_seen():
            allocator_calls.append("seen")
            return seen_allocator()

        def observe_frontier():
            allocator_calls.append("frontier")
            return frontier_allocator()

        self.Owner._allocate_seen = staticmethod(observe_seen)
        self.Owner._allocate_frontier = staticmethod(observe_frontier)
        rejected = self.ledger(
            {
                "retained_bytes": (
                    "concurrent",
                    self.Owner.COLLECTION_BASE_BYTES
                    + self.Owner.TRANSITION_BYTES
                    - 1,
                )
            }
        )
        adopted = self.namespace["_AncestryPermitTransition"].adopt_acquired
        durable_transition_calls = []

        def observe_adoption(transition, *args, **kwargs):
            durable_transition_calls.append(transition)
            return adopted(transition, *args, **kwargs)

        self.namespace["_AncestryPermitTransition"].adopt_acquired = observe_adoption
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.Owner(rejected, lambda _object_id: {"parents": []})
        finally:
            self.namespace["_AncestryPermitTransition"].adopt_acquired = adopted
            self.Owner._allocate_seen = staticmethod(seen_allocator)
            self.Owner._allocate_frontier = staticmethod(frontier_allocator)
        self.assertEqual(allocator_calls, [])
        self.assertEqual(durable_transition_calls, [])
        self.assert_closed(rejected)

        graph = {
            "head": {"parents": ["parent"]},
            "parent": {"parents": ["root"]},
            "root": {"parents": []},
        }
        calls = []
        node_limited = self.ledger({"graph_nodes": ("cumulative", 2)})
        owner = self.Owner(node_limited, self.graph_loader(graph, calls))
        with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
            owner.contains("missing", "head")
        self.assertEqual(calls, ["head", "parent"])
        self.assert_closed(node_limited)

        calls = []
        edge_limited = self.ledger({"graph_edges": ("cumulative", 0)})
        owner = self.Owner(edge_limited, self.graph_loader(graph, calls))
        with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
            owner.contains("missing", "head")
        self.assertEqual(calls, ["head"])
        self.assert_closed(edge_limited)

    def test_frame_growth_depth_and_frontier_limits_precede_append(self):
        append = self.Owner._append_frontier
        append_calls = []

        def observe_append(*args):
            append_calls.append(args[1])
            append(*args)

        self.Owner._append_frontier = staticmethod(observe_append)
        head = "head"
        root_retained = (
            self.Owner.COLLECTION_BASE_BYTES
            + self.Owner.FRAME_BASE_BYTES
            + len(head)
            + self.Owner.FRONTIER_SLOT_BYTES
            + 2 * self.Owner.TRANSITION_BYTES
        )
        root_temporary = self.Owner.FRONTIER_SLOT_BYTES
        for override in [
            {"retained_bytes": ("concurrent", root_retained - 1)},
            {"temporary_bytes": ("concurrent", root_temporary - 1)},
            {"maximum_frontier": ("maximum", 0)},
        ]:
            with self.subTest(override=override):
                append_calls.clear()
                ledger = self.ledger(override)
                owner = self.Owner(
                    ledger, lambda _object_id: {"parents": []}
                )
                with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                    owner.contains(head, head)
                self.assertEqual(append_calls, [])
                self.assert_closed(ledger)
        self.Owner._append_frontier = staticmethod(append)

        append_calls.clear()
        ledger = self.ledger({"logical_depth": ("maximum", 0)})
        owner = self.Owner(
            ledger,
            lambda object_id: {"parents": ["parent"]}
            if object_id == head
            else {"parents": []},
        )
        self.Owner._append_frontier = staticmethod(observe_append)
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                owner.contains("missing", head)
        finally:
            self.Owner._append_frontier = staticmethod(append)
        self.assertEqual(append_calls, [head])
        self.assert_closed(ledger)

    def test_child_depth_is_materialized_only_after_frame_acquisition(self):
        observations = []
        ledger = self.ledger()

        class DepthProbe(int):
            def __add__(self, value):
                observations.append(ledger.current.get("retained_bytes", 0))
                return super().__add__(value)

        append = self.Owner._append_frontier

        def inject_root_depth(frontier, object_id, depth, transition):
            if object_id == "head":
                depth = DepthProbe(depth)
            append(frontier, object_id, depth, transition)

        self.Owner._append_frontier = staticmethod(inject_root_depth)
        owner = self.Owner(
            ledger,
            lambda object_id: {"parents": ["parent"]}
            if object_id == "head"
            else {"parents": []},
        )
        retained_before_child = ledger.current.get("retained_bytes", 0)
        try:
            self.assertTrue(owner.contains("parent", "head"))
        finally:
            self.Owner._append_frontier = staticmethod(append)
        self.assertEqual(len(observations), 1)
        self.assertGreater(observations[0], retained_before_child)
        self.assert_closed(ledger)

    def test_collection_growth_uppers_cover_cpython_through_100k(self):
        fixtures = [
            (
                set(),
                lambda collection, index: collection.add(str(index)),
                self.Owner.SEEN_SLOT_BYTES,
            ),
            (
                deque(),
                lambda collection, index: collection.append((str(index), index)),
                self.Owner.FRONTIER_SLOT_BYTES,
            ),
        ]
        for collection, insert, slot_bytes in fixtures:
            with self.subTest(collection=type(collection).__name__):
                base = sys.getsizeof(collection)
                before = base
                for index in range(100_000):
                    insert(collection, index)
                    after = sys.getsizeof(collection)
                    count = index + 1
                    self.assertLessEqual(after - base, count * slot_bytes)
                    self.assertLessEqual(
                        (before - base) + (after - base),
                        (2 * index + 1) * slot_bytes,
                    )
                    before = after

        # popleft can leave a partially consumed deque block. Its upper is
        # therefore keyed by historical insertions, as the owner does, rather
        # than only by the live queue length.
        collection = deque()
        base = sys.getsizeof(collection)
        for index in range(100_000):
            if collection and index % 3:
                collection.popleft()
            before = sys.getsizeof(collection)
            collection.append((str(index), index))
            after = sys.getsizeof(collection)
            self.assertLessEqual(
                after - base,
                (index + 1) * self.Owner.FRONTIER_SLOT_BYTES,
            )
            self.assertLessEqual(
                (before - base) + (after - base),
                (2 * index + 1) * self.Owner.FRONTIER_SLOT_BYTES,
            )

    def test_deep_chain_is_iterative(self):
        count = 12_000
        calls = 0

        def load(object_id):
            nonlocal calls
            calls += 1
            index = int(object_id[1:])
            return {"parents": [] if index == 0 else [f"n{index - 1}"]}

        ledger = self.ledger()
        owner = self.Owner(ledger, load)
        self.assertTrue(owner.contains("n0", f"n{count - 1}"))
        self.assertEqual(calls, count)
        self.assertEqual(ledger.maximum.get("logical_depth"), count - 1)
        self.assert_closed(ledger)

    def test_deadline_polls_before_loader_and_between_nodes(self):
        now = [0.0]
        deadline = self.namespace["HTTPRequestDeadline"](1, lambda: now[0])
        calls = []
        ledger = self.ledger()
        owner = self.Owner(
            ledger,
            self.graph_loader({"head": {"parents": []}}, calls),
            deadline,
        )
        now[0] = 0.001
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]) as failure:
            owner.contains("head", "head")
        self.assertEqual(failure.exception.phase, "upload ancestry")
        self.assertEqual(calls, [])
        self.assert_closed(ledger)

        now[0] = 0.0
        deadline = self.namespace["HTTPRequestDeadline"](1, lambda: now[0])

        def expire_after_head(object_id):
            calls.append(object_id)
            now[0] = 0.001
            return {"parents": ["parent"]}

        calls = []
        ledger = self.ledger()
        owner = self.Owner(ledger, expire_after_head, deadline)
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            owner.contains("missing", "head")
        self.assertEqual(calls, ["head"])
        self.assert_closed(ledger)

    def test_invalid_subject_closes_without_calling_loader(self):
        calls = []
        ledger = self.ledger()
        owner = self.Owner(
            ledger,
            lambda object_id: calls.append(object_id) or {"parents": []},
        )
        with self.assertRaisesRegex(self.namespace["CFError"], "ASCII"):
            owner.contains("ancestor", "h\N{SNOWMAN}")
        self.assertEqual(calls, [])
        self.assert_closed(ledger)

    def test_allocation_and_post_effect_faults_reconcile(self):
        original_append = self.Owner._append_frontier
        append_calls = 0

        def fail_before_append(*_args):
            nonlocal append_calls
            append_calls += 1
            raise MemoryError("frontier allocation")

        self.Owner._append_frontier = staticmethod(fail_before_append)
        ledger = self.ledger()
        owner = self.Owner(ledger, lambda _object_id: {"parents": []})
        try:
            with self.assertRaisesRegex(MemoryError, "frontier allocation"):
                owner.contains("head", "head")
        finally:
            self.Owner._append_frontier = staticmethod(original_append)
        self.assertEqual(append_calls, 1)
        self.assert_closed(ledger)

        original_seen = self.Owner._insert_seen

        def append_then_raise(*args):
            original_append(*args)
            raise MemoryError("after frontier append")

        def insert_then_raise(*args):
            original_seen(*args)
            raise MemoryError("after seen insertion")

        self.Owner._append_frontier = staticmethod(append_then_raise)
        self.Owner._insert_seen = staticmethod(insert_then_raise)
        ledger = self.ledger()
        owner = self.Owner(
            ledger,
            lambda object_id: {"parents": ["parent"]}
            if object_id == "head"
            else {"parents": []},
        )
        try:
            self.assertTrue(owner.contains("parent", "head"))
        finally:
            self.Owner._append_frontier = staticmethod(original_append)
            self.Owner._insert_seen = staticmethod(original_seen)
        self.assert_closed(ledger)

    def test_transition_handoff_faults_release_the_fixed_gate(self):
        transition_type = self.namespace["_AncestryPermitTransition"]
        adopt = transition_type.adopt_acquired

        def fail_before_adoption(*_args, **_kwargs):
            raise MemoryError("before transition adoption")

        transition_type.adopt_acquired = fail_before_adoption
        ledger = self.ledger()
        try:
            with self.assertRaisesRegex(MemoryError, "before transition adoption"):
                self.Owner(ledger, lambda _object_id: {"parents": []})
        finally:
            transition_type.adopt_acquired = adopt
        self.assert_closed(ledger)

        def adopt_then_raise(transition, *args, **kwargs):
            adopt(transition, *args, **kwargs)
            raise MemoryError("after transition adoption")

        transition_type.adopt_acquired = adopt_then_raise
        ledger = self.ledger()
        try:
            with self.assertRaisesRegex(MemoryError, "after transition adoption"):
                self.Owner(ledger, lambda _object_id: {"parents": []})
        finally:
            transition_type.adopt_acquired = adopt
        self.assert_closed(ledger)

    def test_registration_consume_release_and_cancel_post_effect_faults(self):
        ledger = self.ledger()
        register = ledger._register_reservation
        consume = ledger.consume_reserved
        release = ledger.release

        def register_then_raise(permit, reservation):
            register(permit, reservation)
            raise RuntimeError("after register")

        def consume_then_raise(permit, **actual):
            consume(permit, **actual)
            raise RuntimeError("after consume")

        def release_then_raise(dimension, amount):
            release(dimension, amount)
            raise RuntimeError("after release")

        ledger._register_reservation = register_then_raise
        ledger.consume_reserved = consume_then_raise
        ledger.release = release_then_raise
        owner = self.Owner(
            ledger,
            lambda object_id: {"parents": ["parent"]}
            if object_id == "head"
            else {"parents": []},
        )
        self.assertTrue(owner.contains("parent", "head"))
        self.assert_closed(ledger)

        now = [0.0]
        deadline = self.namespace["HTTPRequestDeadline"](1, lambda: now[0])
        ledger = self.ledger()
        cancel = ledger.cancel_permit

        def cancel_then_raise(permit):
            cancel(permit)
            raise RuntimeError("after cancel")

        owner = self.Owner(
            ledger, lambda _object_id: {"parents": []}, deadline
        )
        ledger.cancel_permit = cancel_then_raise
        now[0] = 0.001
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            owner.contains("head", "head")
        self.assert_closed(ledger)

    def test_payload_precedes_retryable_release_and_foreign_close_is_rejected(self):
        class Payload(dict):
            pass

        class Parents(list):
            pass

        parents = Parents()
        parents_ref = weakref.ref(parents)
        payload = Payload(parents=parents)
        payload_ref = weakref.ref(payload)
        source = {"head": payload}
        parents = None
        payload = None

        def load(object_id):
            return source.pop(object_id)

        ledger = self.ledger()
        owner = self.Owner(ledger, load)
        release = ledger.release
        allow_release = [False]
        observations = []

        def fail_retained_release(dimension, amount):
            if dimension == "retained_bytes" and not allow_release[0]:
                gc.collect()
                observations.append(
                    (payload_ref() is None, parents_ref() is None)
                )
                raise RuntimeError("retained release blocked")
            release(dimension, amount)

        ledger.release = fail_retained_release
        with self.assertRaisesRegex(RuntimeError, "retained release blocked"):
            owner.contains("head", "head")
        self.assertTrue(observations)
        self.assertTrue(
            all(
                payload_dead and parents_dead
                for payload_dead, parents_dead in observations
            )
        )
        self.assertTrue(any(value > 0 for value in ledger.current.values()))
        allow_release[0] = True
        owner.close()
        self.assert_closed(ledger)

        now = [0.0]
        deadline = self.namespace["HTTPRequestDeadline"](1, lambda: now[0])
        parents = Parents(["parent"])
        parents_ref = weakref.ref(parents)
        payload = Payload(parents=parents)
        payload_ref = weakref.ref(payload)
        source = {"head": payload}
        parents = None
        payload = None
        loader_returned = [False]

        def expire_with_payload(object_id):
            now[0] = 0.001
            result = source.pop(object_id)
            loader_returned[0] = True
            return result

        ledger = self.ledger()
        owner = self.Owner(ledger, expire_with_payload, deadline)
        release = ledger.release
        observations = []

        def observe_expired_release(dimension, amount):
            if loader_returned[0]:
                gc.collect()
                observations.append(
                    (payload_ref() is None, parents_ref() is None)
                )
            release(dimension, amount)

        ledger.release = observe_expired_release
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            owner.contains("missing", "head")
        self.assertTrue(observations)
        self.assertTrue(
            all(
                payload_dead and parents_dead
                for payload_dead, parents_dead in observations
            )
        )
        self.assert_closed(ledger)

        own_ledger = self.ledger()
        foreign_ledger = self.ledger()
        foreign_owner = self.Owner(
            own_ledger, lambda _object_id: {"parents": []}
        )
        with self.assertRaisesRegex(self.namespace["CFError"], "different ledger"):
            foreign_owner.close(foreign_ledger)
        foreign_owner.close()
        self.assert_closed(own_ledger)
        self.assert_closed(foreign_ledger)


if __name__ == "__main__":
    unittest.main()
