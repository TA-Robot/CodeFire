import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class CarrierDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None
        self.match = None
        self.calls = []

    def check(self, phase):
        self.calls.append(phase)
        callback = self.callback
        if callback is not None and (
            self.match is None or self.match in phase
        ):
            self.callback = None
            callback(phase)


class PythonPairSemanticCarrierTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Authority = cls.ns["AccountedRemoteSignaturePolicyAuthority"]
        cls.Pair = cls.ns["AccountedHTTPUploadExistingTerminalOwner"]
        cls.Carrier = cls.ns[
            "AccountedHTTPUploadExistingPairSemanticCarrier"
        ]
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
        self.initial = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text()
        )["head"]
        payload = self.ns["load_object_from_root"](self.objects, self.initial)
        child = json.loads(json.dumps(payload))
        child["message"] = "pair semantic carrier"
        child["parents"] = [self.initial]
        self.key = "pair-semantic-secret"
        child = self.ns["sign_commit_payload"](child, "alice", self.key)
        self.head = self.ns["store_object"](self.tmp, "commit", child)
        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        (self.project / "server_policy.json").write_bytes(
            json.dumps(
                {
                    "commit_signatures": {
                        "required": True,
                        "keys": {"alice": self.key},
                    },
                    "verification": {"required": []},
                },
                separators=(",", ":"),
            ).encode()
        )

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertIsNone(ledger._upload_terminal_owner)

    def _semantic_owner(
        self,
        *,
        semantic_branch="main",
        semantic_head=None,
        profile=None,
        carrier_control=True,
    ):
        ledger = self.Ledger(
            self.Profile.http_v1() if profile is None else profile,
            "pair semantic carrier",
            command_kind="http_request",
        )
        deadline = CarrierDeadline()
        records = [
            json.loads(path.read_text())
            for path in sorted(self.objects.glob("*/*.json"))
        ]
        request = {
            "branch": "main",
            "head": self.head if semantic_head is None else semantic_head,
            "objects": records,
        }
        exact = len(self.ns["canonical_json"](request))
        scalar = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, exact)
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        semantic = self.ns["prepare_http_upload_semantic"](
            source,
            ledger,
            "unproved-stable-project-candidate",
            semantic_branch,
            deadline,
            _pair_semantic_carrier=carrier_control,
        )
        return ledger, deadline, request, semantic

    def _prepared(self, *, semantic_branch="main", semantic_head=None):
        ledger, deadline, request, semantic = self._semantic_owner(
            semantic_branch=semantic_branch,
            semantic_head=semantic_head,
        )
        semantic.prepare(ledger)

        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.ns["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
            _semantic_baseline_owner=semantic,
        )
        sealed.validate(ledger)
        policy = self.Authority(self.project, ledger, deadline)
        pair = self.Pair(sealed, policy, ledger)
        pair.verify_signature_and_server("main", self.head, ledger)
        self.assertTrue(pair.contains_ancestor(self.initial, self.head, ledger))
        return ledger, deadline, pair, request, semantic

    def test_ready_is_opaque_uses_pair_callback_and_fresh_mapping(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        source_backing = semantic._semantic
        expected = dict(source_backing)
        carrier = self.Carrier(pair, semantic, ledger)
        carrier.finalize(ledger)
        self.assertEqual(carrier._state, carrier.READY)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        self.assertNotIn("upload semantic terminal", deadline.calls)
        self.assertEqual(dict(carrier._effect_semantic), expected)
        self.assertIsNot(carrier._effect_semantic, semantic._semantic_view)
        source_backing["stable_project_id"] = "mutated-after-ready"
        self.assertEqual(dict(carrier._effect_semantic), expected)
        self.assertTrue(pair._pair_resources_released)
        self.assertTrue(semantic._terminal_complete)
        self.assertIsNone(semantic._source)
        self.assertIs(ledger._upload_terminal_owner, carrier)
        for public_name in ("payload", "semantic", "copy_bytes", "effect"):
            self.assertFalse(hasattr(carrier, public_name))
        with self.assertRaisesRegex(self.CFError, "single-use"):
            carrier.finalize(ledger)
        carrier.close(ledger)
        carrier.close(ledger)
        self._zero(ledger)

    def test_adoption_blocks_every_stale_pair_and_semantic_alias(self):
        ledger, _deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        calls = (
            lambda: pair.contains_ancestor(self.initial, self.head, ledger),
            lambda: pair.finalize_pair(ledger),
            lambda: pair.close(ledger),
            lambda: semantic.matches_semantic({}),
            lambda: semantic.terminal_detach(ledger),
            lambda: semantic.close(ledger),
        )
        for call in calls:
            with self.assertRaisesRegex(self.CFError, "adopted"):
                call()
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_semantic_mutation_aborts_without_payload(self):
        ledger, deadline, pair, request, semantic = self._prepared()
        sealed = pair._sealed_owner
        policy = pair._policy_authority
        carrier = self.Carrier(pair, semantic, ledger)
        deadline.callback = lambda _phase: request.__setitem__("branch", "mAin")
        deadline.match = "upload terminal pair probe"
        with self.assertRaises(self.CFError):
            carrier.finalize(ledger)
        self.assertIsNone(carrier._effect_semantic)
        self.assertIsNone(carrier._effect_copy_bytes)
        self.assertFalse(sealed._pair_detached)
        self.assertFalse(policy._pair_detached)
        self.assertTrue(carrier._closed)
        self._zero(ledger)

    def test_callback_cannot_replace_captured_semantic_probe(self):
        ledger, deadline, _pair, request, semantic = self._prepared()
        carrier = self.Carrier(_pair, semantic, ledger)

        def replace_and_mutate(_phase):
            carrier._semantic_probe = lambda: None
            carrier._semantic_commit = lambda: None
            request["branch"] = "mAin"

        deadline.callback = replace_and_mutate
        deadline.match = "upload terminal pair probe"
        with self.assertRaises(self.CFError):
            carrier.finalize(ledger)
        self.assertTrue(carrier._closed)
        self._zero(ledger)

    def test_post_commit_cleanup_fault_retries_without_callback_or_probe(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        transition_type = type(semantic._allocation)
        original = transition_type.release_temporary
        calls = 0

        def fail_once(selected):
            nonlocal calls
            if selected is not semantic._allocation:
                return original(selected)
            calls += 1
            if calls == 1:
                raise RuntimeError("semantic temporary cleanup fault")
            return original(selected)

        transition_type.release_temporary = fail_once
        try:
            with self.assertRaisesRegex(RuntimeError, "temporary cleanup fault"):
                carrier.finalize(ledger)
            self.assertEqual(carrier._state, carrier.CLEANUP_PENDING)
            self.assertTrue(carrier._input_committed)
            self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
            carrier.finalize(ledger)
        finally:
            transition_type.release_temporary = original
        self.assertEqual(carrier._state, carrier.READY)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        self.assertEqual(calls, 2)
        carrier.close(ledger)
        self._zero(ledger)

    def test_subject_mismatch_mints_no_carrier(self):
        for branch, head in (("other", None), ("main", self.initial)):
            with self.subTest(branch=branch, head=head):
                ledger, _deadline, pair, _request, semantic = self._prepared(
                    semantic_branch=branch,
                    semantic_head=head,
                )
                with self.assertRaisesRegex(self.CFError, "subject mismatch"):
                    self.Carrier(pair, semantic, ledger)
                pair.close(ledger)
                semantic.close(ledger)
                self._zero(ledger)

    def test_callback_registry_and_mirror_replacement_invoke_no_hooks(self):
        class Trap:
            calls = 0

            def __getattribute__(self, _name):
                type(self).calls += 1
                raise AssertionError("trap hook must not run")

        for target in ("registry", "pair_mirror"):
            with self.subTest(target=target):
                ledger, deadline, pair, _request, semantic = self._prepared()
                carrier = self.Carrier(pair, semantic, ledger)
                trap = Trap()

                def mutate(_phase):
                    if target == "registry":
                        object.__setattr__(ledger, "_upload_terminal_owner", trap)
                    else:
                        object.__setattr__(
                            pair, "_pair_semantic_carrier_owner", trap
                        )

                deadline.callback = mutate
                deadline.match = "upload terminal pair probe"
                with self.assertRaises(self.CFError):
                    carrier.finalize(ledger)
                self.assertFalse(carrier._input_committed)
                self.assertEqual(Trap.calls, 0)
                object.__setattr__(ledger, "_upload_terminal_owner", carrier)
                object.__setattr__(
                    pair, "_pair_semantic_carrier_owner", carrier
                )
                carrier.close(ledger)
                self._zero(ledger)

    def test_callback_proof_identity_replacement_mints_no_payload(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        sealed = pair._sealed_owner
        proof = sealed._semantic_baseline_proof
        deadline.callback = lambda _phase: object.__setattr__(
            sealed,
            "_semantic_baseline_proof",
            tuple(list(proof)),
        )
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.CFError, "input changed"):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        self.assertIsNone(carrier._effect_semantic)
        self.assertIsNone(carrier._effect_copy_bytes)
        self.assertTrue(carrier._closed)
        self._zero(ledger)

    def test_pair_cleanup_fault_keeps_semantic_baseline_live_until_retry(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        policy = pair._policy_authority
        original_close = policy.close
        calls = []

        def fail_once(*args, **kwargs):
            calls.append(True)
            if len(calls) == 1:
                raise KeyboardInterrupt("carrier pair cleanup")
            return original_close(*args, **kwargs)

        policy.close = fail_once
        with self.assertRaisesRegex(KeyboardInterrupt, "carrier pair cleanup"):
            carrier.finalize(ledger)
        self.assertEqual(carrier._state, carrier.CLEANUP_PENDING)
        self.assertTrue(carrier._input_committed)
        self.assertFalse(pair._pair_resources_released)
        self.assertFalse(semantic._terminal_complete)
        self.assertFalse(semantic._allocation._retained_primary_released)
        self.assertFalse(semantic._allocation._retained_secondary_released)
        self.assertFalse(semantic._allocation._temporary_released)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        carrier.finalize(ledger)
        self.assertEqual(carrier._state, carrier.READY)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        carrier.close(ledger)
        self._zero(ledger)

    def test_control_scalar_release_is_retryable_before_and_after_effect(self):
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger, _deadline, pair, _request, semantic = self._prepared()
                carrier = self.Carrier(pair, semantic, ledger)
                carrier.finalize(ledger)
                transition = semantic._allocation
                transition_type = type(transition)
                original = transition_type.release_retained_secondary
                calls = []

                def fail_once(selected):
                    if selected is not transition:
                        return original(selected)
                    calls.append(True)
                    if after_effect:
                        original(selected)
                    raise KeyboardInterrupt("carrier control release")

                transition_type.release_retained_secondary = fail_once
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "carrier control release"
                    ):
                        carrier.close(ledger)
                finally:
                    transition_type.release_retained_secondary = original
                self.assertTrue(semantic._carrier_payload_released)
                self.assertIsNone(carrier._pair_owner)
                self.assertIs(carrier._semantic_owner, semantic)
                self.assertIs(ledger._upload_terminal_owner, carrier)
                self.assertEqual(
                    ledger.current.get("retained_bytes", 0),
                    0 if after_effect else carrier.FIXED_RETAINED_BYTES,
                )
                carrier.close(ledger)
                self.assertEqual(len(calls), 1)
                self._zero(ledger)

    def test_semantic_baseline_rejects_foreign_scalar_before_reservation(self):
        ledger, _deadline, _request, semantic = self._semantic_owner()
        semantic.prepare(ledger)
        ledger.acquire("retained_bytes", 1, "query")
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        with self.assertRaisesRegex(self.CFError, "foreign scalars"):
            self.ns["prepare_http_upload_sealed_validation"](
                ledger,
                self.objects,
                self.head,
                record_count_upper=len(sizes),
                aggregate_bytes_upper=sum(sizes),
                maximum_record_bytes_upper=max(sizes),
                deadline=None,
                _semantic_baseline_owner=semantic,
            )
        ledger.release("retained_bytes", 1)
        semantic.close(ledger)
        self._zero(ledger)

    def test_carrier_control_requires_exact_runtime_upper_budget(self):
        self.assertEqual(
            self.Carrier.retained_upper(),
            self.Carrier.FIXED_RETAINED_BYTES,
        )
        self.assertGreater(self.Carrier.FIXED_RETAINED_BYTES, 4096)
        ledger, _deadline, _request, semantic = self._semantic_owner(
            carrier_control=False
        )
        semantic.prepare(ledger)
        normal_peak = ledger.peak["retained_bytes"]
        semantic.close(ledger)
        self._zero(ledger)

        ledger, _deadline, _request, semantic = self._semantic_owner()
        semantic.prepare(ledger)
        carrier_peak = ledger.peak["retained_bytes"]
        self.assertEqual(
            carrier_peak - normal_peak,
            self.Carrier.FIXED_RETAINED_BYTES,
        )
        semantic.close(ledger)
        self._zero(ledger)

        exact_profile = self.Profile.http_v1(
            {"retained_bytes": carrier_peak}
        )
        ledger, _deadline, _request, semantic = self._semantic_owner(
            profile=exact_profile
        )
        semantic.prepare(ledger)
        self.assertEqual(ledger.peak["retained_bytes"], carrier_peak)
        semantic.close(ledger)
        self._zero(ledger)

        short_profile = self.Profile.http_v1(
            {"retained_bytes": carrier_peak - 1}
        )
        ledger, _deadline, _request, semantic = self._semantic_owner(
            profile=short_profile
        )
        with self.assertRaises(self.ns["ResourceBudgetExceeded"]):
            semantic.prepare(ledger)
        semantic.close(ledger)
        self._zero(ledger)

    def test_callback_equal_net_foreign_scalar_cannot_hide_proof_flag_loss(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        transition = semantic._allocation
        amount = transition._retained_secondary

        def mutate(_phase):
            ledger.release("retained_bytes", amount)
            object.__setattr__(
                transition, "_retained_secondary_released", True
            )
            ledger.acquire("retained_bytes", amount, "query")

        deadline.callback = mutate
        deadline.match = "upload terminal pair probe"
        with self.assertRaises(self.CFError):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        self.assertEqual(carrier._state, carrier.ABORT_ONLY)
        object.__setattr__(transition, "_retained_secondary_released", False)
        carrier.close(ledger)
        self._zero(ledger)

    def test_partial_bind_failure_restores_pair_registry_and_both_aliases(self):
        ledger, _deadline, pair, _request, semantic = self._prepared()
        carrier_type = self.Carrier
        class_mapping = type.__getattribute__(carrier_type, "__dict__")
        original_descriptor = class_mapping["_bind_sources"]

        def fail_partial(
            selected_ledger, selected_pair, _semantic, owner, _token
        ):
            object.__setattr__(
                selected_ledger, "_upload_terminal_owner", owner
            )
            object.__setattr__(
                selected_pair, "_pair_semantic_carrier_owner", owner
            )
            raise KeyboardInterrupt("partial carrier bind")

        carrier_type._bind_sources = staticmethod(fail_partial)
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "partial carrier bind"
            ):
                carrier_type(pair, semantic, ledger)
        finally:
            carrier_type._bind_sources = original_descriptor
        self.assertIs(ledger._upload_terminal_owner, pair)
        self.assertIsNone(pair._pair_semantic_carrier_owner)
        self.assertIsNone(semantic._pair_semantic_carrier_owner)
        pair.close(ledger)
        semantic.close(ledger)
        self._zero(ledger)

    def test_carrier_rejects_plain_semantic_wrong_ledger_and_wrong_proof(self):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        ledger, _deadline, _request, semantic = self._semantic_owner(
            carrier_control=False
        )
        semantic.prepare(ledger)
        with self.assertRaisesRegex(self.CFError, "ownership is unavailable"):
            self.ns["prepare_http_upload_sealed_validation"](
                ledger,
                self.objects,
                self.head,
                record_count_upper=len(sizes),
                aggregate_bytes_upper=sum(sizes),
                maximum_record_bytes_upper=max(sizes),
                deadline=None,
                _semantic_baseline_owner=semantic,
            )
        semantic.close(ledger)
        self._zero(ledger)

        ledger, _deadline, pair, _request, semantic = self._prepared()
        foreign = self.Ledger(
            self.Profile.http_v1(),
            "foreign pair semantic carrier",
            command_kind="http_request",
        )
        with self.assertRaisesRegex(self.CFError, "different ledger"):
            self.Carrier(pair, semantic, foreign)
        sealed = pair._sealed_owner
        original_owner = sealed._semantic_baseline_owner
        object.__setattr__(sealed, "_semantic_baseline_owner", object())
        try:
            with self.assertRaisesRegex(
                self.CFError, "ancestry-complete pair"
            ):
                self.Carrier(pair, semantic, ledger)
        finally:
            object.__setattr__(
                sealed, "_semantic_baseline_owner", original_owner
            )
        pair.close(ledger)
        semantic.close(ledger)
        self._zero(ledger)
        self._zero(foreign)

    def test_sealed_before_semantic_proof_cannot_mint_carrier(self):
        ledger = self.Ledger(
            self.Profile.http_v1(),
            "sealed before semantic",
            command_kind="http_request",
        )
        deadline = CarrierDeadline()
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.ns["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
        )
        sealed.validate(ledger)
        records = [json.loads(path.read_text()) for path in sorted(paths)]
        request = {
            "branch": "main",
            "head": self.head,
            "objects": records,
        }
        exact = len(self.ns["canonical_json"](request))
        scalar = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, exact)
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        semantic = self.ns["prepare_http_upload_semantic"](
            source,
            ledger,
            "unproved-stable-project-candidate",
            "main",
            deadline,
            _pair_semantic_carrier=True,
        )
        semantic.prepare(ledger)
        policy = self.Authority(self.project, ledger, deadline)
        pair = self.Pair(sealed, policy, ledger)
        pair.verify_signature_and_server("main", self.head, ledger)
        self.assertTrue(pair.contains_ancestor(self.initial, self.head, ledger))
        with self.assertRaisesRegex(self.CFError, "ancestry-complete pair"):
            self.Carrier(pair, semantic, ledger)
        semantic.close(ledger)
        pair.close(ledger)
        self._zero(ledger)

    def test_foreign_scalar_after_sealed_commit_stops_pair_cleanup_then_retries(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        ledger.acquire("retained_bytes", 1, "query")
        with self.assertRaises(self.CFError):
            carrier.finalize(ledger)
        self.assertTrue(carrier._input_committed)
        self.assertEqual(carrier._state, carrier.CLEANUP_PENDING)
        self.assertFalse(pair._pair_resources_released)
        self.assertFalse(semantic._terminal_complete)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        ledger.release("retained_bytes", 1)
        carrier.finalize(ledger)
        self.assertEqual(carrier._state, carrier.READY)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        carrier.close(ledger)
        self._zero(ledger)

    def test_semantic_carrier_transition_reconciles_reserve_and_consume_faults(self):
        for operation in ("reserve", "consume"):
            for after_effect in (False, True):
                with self.subTest(operation=operation, after_effect=after_effect):
                    ledger, _deadline, _request, semantic = self._semantic_owner()
                    method_name = (
                        "reserve_into"
                        if operation == "reserve"
                        else "consume_reserved"
                    )
                    original = getattr(ledger, method_name)
                    fired = False

                    def fail_once(*args, **kwargs):
                        nonlocal fired
                        if fired:
                            return original(*args, **kwargs)
                        fired = True
                        if after_effect:
                            original(*args, **kwargs)
                        raise RuntimeError(
                            f"semantic carrier {operation} fault"
                        )

                    setattr(ledger, method_name, fail_once)
                    try:
                        if after_effect:
                            semantic.prepare(ledger)
                        else:
                            with self.assertRaisesRegex(
                                RuntimeError,
                                f"semantic carrier {operation} fault",
                            ):
                                semantic.prepare(ledger)
                    finally:
                        setattr(ledger, method_name, original)
                    self.assertTrue(fired)
                    semantic.close(ledger)
                    semantic.close(ledger)
                    self._zero(ledger)

    def test_semantic_secondary_amount_tamper_is_not_a_baseline_proof(self):
        ledger, _deadline, _request, semantic = self._semantic_owner()
        semantic.prepare(ledger)
        transition = semantic._allocation
        original_secondary = transition._retained_secondary
        object.__setattr__(transition, "_retained_secondary", 1)
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        try:
            with self.assertRaisesRegex(
                self.CFError, "ownership is unavailable"
            ):
                self.ns["prepare_http_upload_sealed_validation"](
                    ledger,
                    self.objects,
                    self.head,
                    record_count_upper=len(sizes),
                    aggregate_bytes_upper=sum(sizes),
                    maximum_record_bytes_upper=max(sizes),
                    deadline=None,
                    _semantic_baseline_owner=semantic,
                )
        finally:
            object.__setattr__(
                transition, "_retained_secondary", original_secondary
            )
        semantic.close(ledger)
        self._zero(ledger)

    def test_same_total_release_acquire_in_last_callback_cannot_mint_ready(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        amount = semantic._allocation._retained_secondary

        def replace_scalar(_phase):
            ledger.release("retained_bytes", amount)
            ledger.acquire("retained_bytes", amount, "query")

        deadline.callback = replace_scalar
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.CFError, "ledger changed"):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        self.assertNotEqual(carrier._state, carrier.READY)
        self.assertEqual(deadline.calls.count("upload terminal pair probe"), 1)
        if not carrier._closed:
            carrier.close(ledger)
        self._zero(ledger)

    def test_charge_only_open_permit_in_last_callback_blocks_ready(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        carrier = self.Carrier(pair, semantic, ledger)
        permit = None

        def reserve_only(_phase):
            nonlocal permit
            permit = ledger.reserve(
                "query",
                charges={"aggregate_bytes": 1},
            )

        deadline.callback = reserve_only
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.CFError, "ledger changed"):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        self.assertTrue(ledger.has_open_permits())
        self.assertIsNotNone(permit)
        ledger.cancel_permit(permit)
        if not carrier._closed:
            carrier.close(ledger)
        self._zero(ledger)

    def test_terminal_owner_class_hook_replacement_invokes_no_hook(self):
        for target_name in (
            "pair",
            "policy",
            "sealed",
            "semantic",
            "carrier",
            "ledger",
        ):
            with self.subTest(target=target_name):
                ledger, deadline, pair, _request, semantic = self._prepared()
                policy = pair._policy_authority
                sealed = pair._sealed_owner
                carrier = self.Carrier(pair, semantic, ledger)
                target_type = {
                    "pair": type(pair),
                    "policy": type(policy),
                    "sealed": type(sealed),
                    "semantic": type(semantic),
                    "carrier": type(carrier),
                    "ledger": type(ledger),
                }[target_name]
                target_mapping = type.__getattribute__(target_type, "__dict__")
                original = target_mapping.get("__getattribute__")
                carrier_mapping = object.__getattribute__(carrier, "__dict__")
                abort_state = carrier.ABORT_ONLY
                calls = []

                def trap(_selected, _name):
                    calls.append(True)
                    raise AssertionError("terminal owner hook must not run")

                deadline.callback = lambda _phase: setattr(
                    target_type, "__getattribute__", trap
                )
                deadline.match = "upload terminal pair probe"
                try:
                    with self.assertRaisesRegex(self.CFError, "ledger changed"):
                        carrier.finalize(ledger)
                    self.assertEqual(calls, [])
                    self.assertEqual(carrier_mapping["_state"], abort_state)
                    self.assertFalse(carrier_mapping["_closed"])
                finally:
                    if original is None:
                        delattr(target_type, "__getattribute__")
                    else:
                        target_type.__getattribute__ = original
                carrier.close(ledger)
                self._zero(ledger)

    def test_private_pair_finalize_rejects_fake_and_replaced_carrier_inputs(self):
        ledger, _deadline, pair, _request, semantic = self._prepared()
        fake = object()
        object.__setattr__(ledger, "_upload_terminal_owner", fake)
        object.__setattr__(pair, "_pair_semantic_carrier_owner", fake)
        object.__setattr__(pair, "_pair_semantic_carrier_token", object())
        with self.assertRaisesRegex(self.CFError, "semantic carrier"):
            pair.finalize_pair(
                ledger,
                _pair_semantic_carrier_owner=fake,
                _pair_semantic_carrier_token=pair._pair_semantic_carrier_token,
                _semantic_probe=lambda: None,
                _semantic_commit=lambda: None,
            )
        object.__setattr__(ledger, "_upload_terminal_owner", pair)
        object.__setattr__(pair, "_pair_semantic_carrier_owner", None)
        object.__setattr__(pair, "_pair_semantic_carrier_token", None)

        carrier = self.Carrier(pair, semantic, ledger)
        carrier._state = carrier.FINALIZING
        with self.assertRaisesRegex(self.CFError, "semantic carrier"):
            pair.finalize_pair(
                ledger,
                _pair_semantic_carrier_owner=carrier,
                _pair_semantic_carrier_token=carrier._pair_token,
                _semantic_probe=lambda: None,
                _semantic_commit=lambda: None,
            )
        carrier._state = carrier.ADOPTED
        carrier.close(ledger)
        self._zero(ledger)


if __name__ == "__main__":
    unittest.main()
