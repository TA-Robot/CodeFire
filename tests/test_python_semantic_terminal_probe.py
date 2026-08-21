import json
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class ProbeDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None

    def check(self, _phase):
        callback = self.callback
        if callback is not None:
            self.callback = None
            callback()


class HookDict(dict):
    armed = False
    calls = 0

    def items(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("dict hook ran")
        return super().items()


class HookList(list):
    armed = False
    calls = 0

    def __iter__(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("list hook ran")
        return super().__iter__()


class HookStr(str):
    armed = False
    calls = 0

    def __lt__(self, other):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("string hook ran")
        return super().__lt__(other)

    def __eq__(self, other):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("string equality hook ran")
        return super().__eq__(other)

    def __hash__(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("string hash hook ran")
        return super().__hash__()


class HookInt(int):
    armed = False
    calls = 0

    def __repr__(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("int hook ran")
        return super().__repr__()


class HookFloat(float):
    armed = False
    calls = 0

    def __repr__(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("float hook ran")
        return super().__repr__()


class PythonSemanticTerminalProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Probe = cls.ns["_HTTPUploadSemanticTerminalProbe"]
        cls.CFError = cls.ns["CFError"]

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
        self.objects = self.tmp / ".codefire/objects"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _owned_probe_scratch(self, probe):
        pins = probe._pins
        measured = sys.getsizeof(pins) + sys.getsizeof(probe._primitives)
        measured += sum(sys.getsizeof(array) for array in pins[12])
        plan_bundle = pins[13]
        plan_table = plan_bundle[0]
        measured += sys.getsizeof(plan_bundle) + sys.getsizeof(plan_table)
        measured += sys.getsizeof(plan_bundle[2])
        owned_ints = {id(plan_bundle[1]): plan_bundle[1]}
        for value in plan_bundle[2]:
            owned_ints[id(value)] = value
        for descriptor in plan_table:
            measured += sys.getsizeof(descriptor)
            kind = descriptor[0]
            selected = (descriptor[2], descriptor[3]) if kind == 2 else (
                descriptor[2],
                descriptor[3],
                descriptor[4],
                descriptor[6],
            )
            for item in selected:
                measured += sys.getsizeof(item)
            if kind == 1:
                owned_ints[id(descriptor[5])] = descriptor[5]
                child_indices = descriptor[6]
            else:
                child_indices = descriptor[3]
            for value in child_indices:
                owned_ints[id(value)] = value
        measured += sum(sys.getsizeof(value) for value in owned_ints.values())
        measured += sys.getsizeof(pins[7])
        measured += sum(sys.getsizeof(view) for view in pins[7])
        for index in (4, 5, 6, 14, 15, 16):
            measured += sys.getsizeof(pins[index])
        for state in pins[17:20]:
            measured += sys.getsizeof(state)
            measured += sys.getsizeof(state[4]) + sys.getsizeof(state[5])
        measured += 12 * pins[11]
        return measured

    def _retained_frame_integer_bytes(self, arrays):
        retained = {}
        for array in arrays:
            for value in array:
                if type(value) is int:
                    retained[id(value)] = value
        return sum(sys.getsizeof(value) for value in retained.values())

    def _prepared(self, extra=None, records_mutator=None):
        ledger = self.Ledger(
            self.Profile.http_v1(),
            "semantic terminal probe",
            command_kind="http_request",
        )
        records = [
            json.loads(path.read_text())
            for path in sorted(self.objects.glob("*/*.json"))
        ]
        if records_mutator is not None:
            records_mutator(records)
        head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text()
        )["head"]
        request = {"branch": "main", "head": head, "objects": records}
        if extra is not None:
            request["extra"] = extra
        exact = len(self.ns["canonical_json"](request))
        scalar = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, exact)
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        deadline = ProbeDeadline()
        semantic = self.ns["prepare_http_upload_semantic"](
            source,
            ledger,
            "candidate-only",
            "main",
            deadline,
        )
        semantic.prepare(ledger)
        return ledger, deadline, request, semantic

    def _close(self, ledger, semantic, probe):
        probe.close()
        semantic.close(ledger)
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_exact_probe_covers_all_four_parities_and_is_single_use(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        exact()
        with self.assertRaisesRegex(self.CFError, "single-use"):
            exact()
        self._close(ledger, semantic, probe)

    def test_equal_length_reorder_replace_and_private_tamper_reject(self):
        mutations = (
            lambda request, semantic: request.__setitem__("branch", "mAin"),
            lambda request, semantic: request["objects"].reverse(),
            lambda request, semantic: request["objects"].__setitem__(
                0, json.loads(json.dumps(request["objects"][0]))
            ),
            lambda _request, semantic: semantic._semantic.__setitem__(
                "remote_branch", "mAin"
            ),
            lambda _request, semantic: setattr(
                semantic, "_copy_bytes", tuple(reversed(semantic._copy_bytes))
            ),
            lambda _request, semantic: setattr(
                semantic, "_sorted_refs", tuple(reversed(semantic._sorted_refs))
            ),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                ledger, _deadline, request, semantic = self._prepared()
                probe = self.Probe(semantic)
                exact = probe.capture()
                mutation(request, semantic)
                with self.assertRaises(self.CFError):
                    exact()
                self._close(ledger, semantic, probe)

    def test_last_callback_subclasses_never_execute_hooks(self):
        def insert_subclass_key(request):
            HookStr.armed = False
            dict.__setitem__(request, HookStr("bad"), None)
            HookStr.armed = True

        mutations = (
            lambda request: request.__setitem__("bad", HookDict()),
            lambda request: request.__setitem__("bad", HookList()),
            insert_subclass_key,
            lambda request: request.__setitem__("bad", HookStr("value")),
            lambda request: request.__setitem__("bad", HookInt(7)),
            lambda request: request.__setitem__("bad", HookFloat(1.5)),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                ledger, deadline, request, semantic = self._prepared()
                probe = self.Probe(semantic)
                exact = probe.capture()
                deadline.callback = lambda: mutation(request)
                HookDict.calls = HookList.calls = HookStr.calls = 0
                HookInt.calls = HookFloat.calls = 0
                HookDict.armed = HookList.armed = HookStr.armed = True
                HookInt.armed = HookFloat.armed = True
                try:
                    deadline.check("last")
                    with self.assertRaisesRegex(
                        self.CFError, "exact|string keys|length changed"
                    ):
                        exact()
                finally:
                    HookDict.armed = HookList.armed = HookStr.armed = False
                    HookInt.armed = HookFloat.armed = False
                self.assertEqual(
                    (
                        HookDict.calls,
                        HookList.calls,
                        HookStr.calls,
                        HookInt.calls,
                        HookFloat.calls,
                    ),
                    (0, 0, 0, 0, 0),
                )
                self._close(ledger, semantic, probe)

    def test_last_callback_huge_and_cyclic_inputs_are_bounded(self):
        mutations = (
            lambda request: request.__setitem__("extra", [None] * 1_000_000),
            lambda request: request.__setitem__("extra", 1 << 1_000_000),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                ledger, deadline, request, semantic = self._prepared()
                probe = self.Probe(semantic)
                exact = probe.capture()
                deadline.callback = lambda: mutation(request)
                deadline.check("last")
                with self.assertRaises(self.CFError):
                    exact()
                self._close(ledger, semantic, probe)

        ledger, deadline, request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()

        def cycle():
            value = []
            value.append(value)
            request["extra"] = value

        deadline.callback = cycle
        deadline.check("last")
        with self.assertRaises(self.CFError):
            exact()
        self._close(ledger, semantic, probe)

        ledger, deadline, request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()

        def deep():
            value = None
            for _ in range(3000):
                value = [value]
            request["extra"] = value

        deadline.callback = deep
        deadline.check("last")
        with self.assertRaises(self.CFError):
            exact()
        self._close(ledger, semantic, probe)

    def test_scalar_canonical_parity_and_same_length_mutation(self):
        values = [
            "escape\b\n\U0001f600",
            -0.0,
            float("nan"),
            float("inf"),
            float("-inf"),
            1 << 4096,
        ]
        ledger, _deadline, _request, semantic = self._prepared(values)
        probe = self.Probe(semantic)
        probe.capture()()
        self._close(ledger, semantic, probe)

        for before, after in ((10, 11), (1.25, 1.26), ("same", "sAme")):
            with self.subTest(before=before, after=after):
                ledger, deadline, request, semantic = self._prepared(before)
                probe = self.Probe(semantic)
                exact = probe.capture()
                deadline.callback = lambda: request.__setitem__("extra", after)
                deadline.check("last")
                with self.assertRaises(self.CFError):
                    exact()
                self._close(ledger, semantic, probe)

    def test_captured_functions_ignore_class_and_global_replacement(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        originals = (
            self.Probe._probe,
            self.Probe._walk,
            self.Probe.close,
            self.ns["canonical_json"],
        )
        self.Probe._probe = staticmethod(lambda *_args: None)
        self.Probe._walk = staticmethod(lambda *_args: None)
        self.Probe.close = lambda *_args: None
        probe._pins = None
        probe._primitives = None
        probe._revoke = lambda: None
        self.ns["canonical_json"] = lambda *_args: (_ for _ in ()).throw(
            AssertionError("global canonical helper ran")
        )
        try:
            exact()
        finally:
            self.Probe._probe = staticmethod(originals[0])
            self.Probe._walk = staticmethod(originals[1])
            self.Probe.close = originals[2]
            self.ns["canonical_json"] = originals[3]
        self._close(ledger, semantic, probe)

    def test_equal_subclass_keys_never_run_lookup_hooks(self):
        targets = (
            lambda request, semantic, probe: (request, "objects"),
            lambda _request, semantic, _probe: (
                semantic._record_refs[0],
                "object_id",
            ),
            lambda _request, semantic, _probe: (
                semantic._semantic,
                "stable_project_id",
            ),
            lambda _request, semantic, _probe: (semantic.__dict__, "_source"),
            lambda _request, semantic, _probe: (
                semantic._source.__dict__,
                "_request",
            ),
            lambda _request, _semantic, probe: (probe.__dict__, "_pins"),
        )
        for select_target in targets:
            with self.subTest(select_target=select_target):
                ledger, _deadline, request, semantic = self._prepared()
                probe = self.Probe(semantic)
                exact = probe.capture()
                mapping, selected_key = select_target(request, semantic, probe)
                original_items = list(dict.items(mapping))
                selected_value = dict.__getitem__(mapping, selected_key)
                dict.__delitem__(mapping, selected_key)
                subclass_key = HookStr(selected_key)
                dict.__setitem__(mapping, subclass_key, selected_value)
                HookStr.calls = 0
                HookStr.armed = True
                try:
                    with self.assertRaises(self.CFError):
                        exact()
                finally:
                    HookStr.armed = False
                    dict.clear(mapping)
                    for key, value in original_items:
                        dict.__setitem__(mapping, key, value)
                self.assertEqual(HookStr.calls, 0)
                self._close(ledger, semantic, probe)

    def test_captured_instance_dict_descriptors_ignore_class_replacement(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        selected_types = (type(semantic), type(semantic._source), type(probe))
        originals = []
        hook_calls = 0

        def fail_getattribute(_owner, _name):
            nonlocal hook_calls
            hook_calls += 1
            raise AssertionError("instance attribute hook ran")

        for selected in selected_types:
            namespace = type.__getattribute__(selected, "__dict__")
            originals.append((
                selected,
                "__getattribute__" in namespace,
                namespace.get("__getattribute__"),
            ))
            type.__setattr__(selected, "__getattribute__", fail_getattribute)
        try:
            exact()
        finally:
            for selected, had_value, original in originals:
                if had_value:
                    type.__setattr__(selected, "__getattribute__", original)
                else:
                    type.__delattr__(selected, "__getattribute__")
        self.assertEqual(hook_calls, 0)
        self._close(ledger, semantic, probe)

    def test_compatible_instance_class_change_is_normalized(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        original_type = type(semantic)

        class CompatibleSemantic(original_type):
            pass

        semantic.__class__ = CompatibleSemantic
        try:
            with self.assertRaises(self.CFError):
                exact()
        finally:
            semantic.__class__ = original_type
        self._close(ledger, semantic, probe)

    def test_huge_extra_instance_state_is_constant_time_reject(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        semantic_mapping = semantic.__dict__
        original_items = list(dict.items(semantic_mapping))
        for index in range(100_000):
            dict.__setitem__(semantic_mapping, f"_extra_{index}", index)
        try:
            with self.assertRaisesRegex(self.CFError, "private state changed"):
                exact()
        finally:
            dict.clear(semantic_mapping)
            for key, value in original_items:
                dict.__setitem__(semantic_mapping, key, value)
        self._close(ledger, semantic, probe)

    def test_plan_is_immutable_and_has_no_post_callback_identity_map(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        plan_bundle = probe._pins[13]
        self.assertIs(type(plan_bundle), tuple)
        self.assertIs(type(plan_bundle[0]), tuple)
        with self.assertRaises(TypeError):
            plan_bundle[0][plan_bundle[1]] = HookDict()
        probe.capture()()
        self._close(ledger, semantic, probe)

    def test_deep_valid_tree_fits_formal_step_and_frame_bounds(self):
        value = None
        for _ in range(3000):
            value = [value]
        previous_limit = sys.getrecursionlimit()
        sys.setrecursionlimit(10000)
        try:
            ledger, _deadline, _request, semantic = self._prepared(value)
        finally:
            sys.setrecursionlimit(previous_limit)
        probe = self.Probe(semantic)
        limit = semantic._source._parser_canonical_bytes
        arrays = probe._pins[12]
        measured = self._owned_probe_scratch(probe)
        probe.capture()()
        measured += self._retained_frame_integer_bytes(arrays)
        self.assertLessEqual(
            measured,
            semantic.SEMANTIC_TEMPORARY_BYTES_PER_SOURCE_BYTE * (limit + 1),
        )
        self.assertEqual(
            len(arrays), probe.FRAME_ARRAYS + probe.KEY_ARENAS
        )
        self._close(ledger, semantic, probe)

    def test_nested_remote_is_preserved_in_core_parity(self):
        def add_remote(records):
            records[0]["remote"] = {"root": "omitted"}
            records[0]["extensions"] = [
                {
                    "namespace": "probe",
                    "name": "nested-remote",
                    "version": 1,
                    "critical": False,
                    "references": [],
                    "remote": {"nested": True},
                }
            ]

        ledger, _deadline, _request, semantic = self._prepared(
            records_mutator=add_remote
        )
        probe = self.Probe(semantic)
        probe.capture()()
        self._close(ledger, semantic, probe)

    def test_shared_alias_and_many_empty_plans_remain_bounded(self):
        shared = {"nested": [1, 2, 3]}
        ledger, _deadline, _request, semantic = self._prepared(
            {"left": shared, "right": shared}
        )
        probe = self.Probe(semantic)
        probe.capture()()
        self._close(ledger, semantic, probe)

        ledger, _deadline, _request, semantic = self._prepared(
            [{} for _ in range(1000)]
        )
        probe = self.Probe(semantic)
        limit = semantic._source._parser_canonical_bytes
        plan = probe._pins[13]
        self.assertIs(type(plan), tuple)
        self.assertIs(type(plan[0]), tuple)
        self.assertTrue(all(type(item) is tuple for item in plan[0]))
        measured = self._owned_probe_scratch(probe)
        self.assertLess(
            measured,
            4096 + semantic.SEMANTIC_TEMPORARY_BYTES_PER_SOURCE_BYTE * limit,
        )
        probe.capture()()
        self._close(ledger, semantic, probe)

    def test_captured_probe_is_revoked_by_close(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        exact = probe.capture()
        probe.close()
        with self.assertRaisesRegex(self.CFError, "single-use"):
            exact()
        semantic.close(ledger)

    def test_single_owner_partial_construction_and_semantic_close(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        captured = probe.capture()
        with self.assertRaisesRegex(self.CFError, "already owned"):
            self.Probe(semantic)
        with self.assertRaisesRegex(self.CFError, "still live"):
            semantic.terminal_detach(ledger)
        semantic.close(ledger)
        with self.assertRaisesRegex(self.CFError, "single-use"):
            captured()
        self.assertIsNone(semantic._terminal_probe_owner)
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

        ledger, _deadline, _request, semantic = self._prepared()
        globals_map = self.Probe.__init__.__globals__
        had_memoryview = "memoryview" in globals_map
        original = globals_map.get("memoryview")
        built_memoryviews = []
        calls = 0
        builtin_memoryview = memoryview

        def fail_later(value):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise RuntimeError("probe allocation fault")
            view = builtin_memoryview(value)
            built_memoryviews.append(view)
            return view

        globals_map["memoryview"] = fail_later
        try:
            with self.assertRaisesRegex(RuntimeError, "allocation fault"):
                self.Probe(semantic)
        finally:
            if had_memoryview:
                globals_map["memoryview"] = original
            else:
                del globals_map["memoryview"]
        self.assertIsNone(semantic._terminal_probe_owner)
        for view in built_memoryviews:
            with self.assertRaises(ValueError):
                _ = view.obj
        semantic.close(ledger)

    def test_all_copy_views_release_on_late_publication_fault(self):
        ledger, _deadline, _request, semantic = self._prepared()
        globals_map = self.Probe.__init__.__globals__
        had_memoryview = "memoryview" in globals_map
        original_memoryview = globals_map.get("memoryview")
        builtin_memoryview = memoryview
        built_memoryviews = []

        def capture_view(value):
            selected = builtin_memoryview(value)
            built_memoryviews.append(selected)
            return selected

        semantic_type = type(semantic)
        semantic_namespace = type.__getattribute__(semantic_type, "__dict__")
        had_setattr = "__setattr__" in semantic_namespace
        original_setattr = semantic_namespace.get("__setattr__")

        def fail_publication(owner, name, value):
            if name == "_terminal_probe_owner":
                object.__setattr__(owner, name, value)
                raise RuntimeError("late probe publication fault")
            object.__setattr__(owner, name, value)

        globals_map["memoryview"] = capture_view
        type.__setattr__(semantic_type, "__setattr__", fail_publication)
        try:
            with self.assertRaisesRegex(RuntimeError, "late probe publication"):
                self.Probe(semantic)
        finally:
            if had_setattr:
                type.__setattr__(semantic_type, "__setattr__", original_setattr)
            else:
                type.__delattr__(semantic_type, "__setattr__")
            if had_memoryview:
                globals_map["memoryview"] = original_memoryview
            else:
                del globals_map["memoryview"]
        self.assertGreater(len(built_memoryviews), 1)
        for selected in built_memoryviews:
            with self.assertRaises(ValueError):
                _ = selected.obj
        self.assertIsNone(semantic._terminal_probe_owner)
        semantic.close(ledger)

    def test_preallocated_pointer_peak_is_below_semantic_envelope(self):
        ledger, _deadline, _request, semantic = self._prepared()
        probe = self.Probe(semantic)
        limit = semantic._source._parser_canonical_bytes
        arrays = probe._pins[12]
        measured = self._owned_probe_scratch(probe)
        documented = (
            4096
            + semantic.SEMANTIC_TEMPORARY_BYTES_PER_SOURCE_BYTE * limit
        )
        self.assertLess(measured, documented)
        self._close(ledger, semantic, probe)


if __name__ == "__main__":
    unittest.main()
