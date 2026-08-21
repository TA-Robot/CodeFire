import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class EffectInputDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None
        self.calls = []

    def check(self, phase):
        self.calls.append(phase)
        callback = self.callback
        if callback is not None and "upload terminal pair probe" in phase:
            self.callback = None
            callback(phase)


class PythonExistingEffectInputTests(unittest.TestCase):
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
        cls.Owner = cls.ns["AccountedHTTPUploadExistingEffectInput"]
        cls.Prepare = staticmethod(
            cls.ns["prepare_http_upload_existing_effect_input"]
        )
        cls.PrepareResponse = staticmethod(
            cls.ns["prepare_http_upload_response_input"]
        )
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
        payload = self.ns["load_object_from_root"](
            self.objects, self.initial
        )
        child = json.loads(json.dumps(payload))
        child["message"] = "existing effect input"
        child["parents"] = [self.initial]
        self.key = "existing-effect-input-secret"
        child = self.ns["sign_commit_payload"](child, "alice", self.key)
        self.head = self.ns["store_object"](self.tmp, "commit", child)
        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        self.stable_id = "CF-PROJECT-0123456789abcdef0123456789abcdef"
        (self.project / "locks").mkdir()
        (self.project / "locks/project-gate.lock").write_bytes(b"")
        (self.project / "remote_project.json").write_bytes(
            self.ns["canonical_json"](
                {
                    "type": "codefire_remote_project",
                    "version": 1,
                    "stable_project_id": self.stable_id,
                    "protocol_generation": 1,
                    "organization": "org",
                    "application": "app",
                    "creation_provenance": "test",
                    "created_at": "2026-07-22T00:00:00Z",
                }
            )
            + b"\n"
        )
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

    def _prepared(
        self,
        profile=None,
        *,
        gated=False,
        effect_control=True,
        response_input=False,
        response_values=None,
        publication_input=False,
        semantic_only=False,
    ):
        ledger = self.Ledger(
            self.Profile.http_v1() if profile is None else profile,
            "existing effect input",
            command_kind="http_request",
        )
        deadline = EffectInputDeadline()
        gate = None
        if gated:
            gate = self.ns["prepare_http_upload_existing_project_gate"](
                self.project, "org", "app", ledger, deadline
            )
        records = [
            json.loads(path.read_text())
            for path in sorted(self.objects.glob("*/*.json"))
        ]
        request = {"branch": "main", "head": self.head, "objects": records}
        exact = len(self.ns["canonical_json"](request))
        response = None
        if response_input:
            response = self.PrepareResponse(
                ledger, body_upper=64, header_upper=64
            )
        scalar = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, exact)
        if response is not None:
            values = response_values or (
                200,
                b"success-header",
                b"success-body",
                b"durable-header",
                b"durable-body",
            )
            response.prepare(
                ledger,
                *values,
            )
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        if gate is None:
            semantic = self.ns["prepare_http_upload_semantic"](
                source,
                ledger,
                "unproved-stable-project-candidate",
                "main",
                deadline,
                _pair_semantic_carrier=True,
                _existing_effect_input=effect_control,
                _existing_response_input=response,
            )
        else:
            semantic = gate.prepare_semantic(
                source,
                ledger,
                "main",
                deadline,
                _existing_effect_input=effect_control,
                _existing_response_input=response,
            )
        semantic.prepare(ledger)
        if semantic_only:
            self.response_input = response
            self.publication_input = None
            return ledger, deadline, request, semantic, gate
        self.publication_input = None
        if publication_input:
            publication_plan = {
                "project_root": self.project,
                "operation": "remote_upload",
                "actor": "alice",
                "writes": [
                    (
                        {"kind": "generation"},
                        None,
                        {"current_generation": 1},
                    )
                ],
                "result": {"branch": "main", "head": self.head},
                "idempotency_key": None,
                "object_roots": [self.head],
            }
            values = [
                publication_plan["operation"],
                publication_plan["actor"],
                publication_plan["idempotency_key"],
                publication_plan["object_roots"],
                publication_plan["result"],
            ]
            for write in publication_plan["writes"]:
                values.extend(write)
            canonical = self.ns["canonical_json"]
            canonical_upper = sum(len(canonical(value)) for value in values)
            semantic_upper = len(canonical(semantic._semantic))
            publication = self.ns[
                "prepare_http_upload_existing_publication_input"
            ](
                ledger,
                canonical_upper=canonical_upper,
                semantic_upper=semantic_upper,
            )
            publication.prepare(ledger, **publication_plan)
            publication._bind_request_identity(semantic, ledger)
            self.publication_input = publication
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
        carrier = (
            self.Carrier(pair, semantic, ledger)
            if gate is None
            else gate.prepare_carrier(pair, semantic, ledger)
        )
        self.response_input = response
        return ledger, deadline, request, carrier, gate

    def test_destination_first_transfer_is_opaque_and_closes_gate_owner_last(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        expected = dict(carrier._semantic_owner._semantic_view)
        expected_control = (
            self.Carrier.FIXED_RETAINED_BYTES
            + self.Owner.FIXED_RETAINED_BYTES
        )
        self.assertEqual(
            carrier._semantic_owner._allocation._retained_secondary,
            expected_control,
        )
        retained_before = ledger.current["retained_bytes"]
        owner = self.Prepare(carrier, ledger)
        self.assertEqual(ledger.current["retained_bytes"], retained_before)
        with self.assertRaisesRegex(self.CFError, "adopted carrier"):
            carrier.close(ledger)
        carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.READY)
        self.assertIs(ledger._upload_terminal_owner, owner)
        self.assertIsNone(carrier._effect_semantic)
        self.assertIsNone(carrier._effect_copy_bytes)
        self.assertEqual(dict(owner._effect_semantic), expected)
        self.assertGreater(len(owner._effect_copy_bytes), 0)
        for public_name in ("payload", "semantic", "copy_bytes", "effect"):
            self.assertFalse(hasattr(owner, public_name))
        owner.close(ledger)
        owner.close(ledger)
        self._zero(ledger)

    def test_missing_destination_preaccount_is_rejected_without_consumption(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared(
            effect_control=False
        )
        with self.assertRaisesRegex(self.CFError, "semantic control"):
            self.Prepare(carrier, ledger)
        self.assertEqual(carrier._state, carrier.ADOPTED)
        carrier.close(ledger)
        self._zero(ledger)

    def test_close_before_terminal_cancels_every_parent_and_control(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        owner.close(ledger)
        self.assertTrue(carrier._closed)
        self.assertTrue(owner._closed)
        self._zero(ledger)

    def test_stable_gate_remains_live_until_effect_input_close(self):
        ledger, _deadline, _request, carrier, gate = self._prepared(
            gated=True
        )
        owner = self.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.READY)
        self.assertFalse(gate._closed)
        self.assertTrue(gate._lock_held)
        owner.close(ledger)
        self.assertTrue(gate._closed)
        self._zero(ledger)

    def test_foreign_ledger_and_direct_constructor_are_rejected(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        foreign = self.Ledger(
            self.Profile.http_v1(),
            "foreign effect input",
            command_kind="http_request",
        )
        with self.assertRaisesRegex(self.CFError, "another ledger"):
            self.Prepare(carrier, foreign)
        with self.assertRaisesRegex(self.CFError, "use prepare"):
            self.Owner(carrier, ledger)
        carrier.close(ledger)
        self._zero(ledger)
        self._zero(foreign)

    def test_last_callback_owner_mutation_mints_no_effect_input(self):
        ledger, deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        original_token = owner._token
        deadline.callback = lambda _phase: object.__setattr__(
            owner, "_token", object()
        )
        with self.assertRaisesRegex(self.CFError, "effect input changed"):
            carrier.finalize(ledger)
        self.assertIsNone(owner._effect_semantic)
        self.assertIsNone(owner._effect_copy_bytes)
        owner._token = original_token
        owner.close(ledger)
        self._zero(ledger)

    def test_last_callback_class_mutation_is_not_dispatched(self):
        ledger, deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        original = self.Owner.close
        replacement_called = False

        def replacement(*_args, **_kwargs):
            nonlocal replacement_called
            replacement_called = True

        deadline.callback = lambda _phase: setattr(
            self.Owner, "close", replacement
        )
        try:
            with self.assertRaisesRegex(self.CFError, "effect input changed"):
                carrier.finalize(ledger)
        finally:
            self.Owner.close = original
        self.assertFalse(replacement_called)
        owner.close(ledger)
        self._zero(ledger)

    def test_post_commit_cleanup_fault_retries_without_terminal_callback(self):
        ledger, deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        original = carrier._semantic_cleanup
        calls = 0

        def fail_once(semantic, selected_carrier):
            nonlocal calls
            calls += 1
            if calls == 1:
                raise RuntimeError("effect input cleanup fault")
            return original(semantic, selected_carrier)

        carrier._semantic_cleanup = fail_once
        with self.assertRaisesRegex(RuntimeError, "cleanup fault"):
            carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.STAGED)
        self.assertTrue(carrier._input_committed)
        callback_count = deadline.calls.count("upload terminal pair probe")
        carrier.finalize(ledger)
        self.assertEqual(
            deadline.calls.count("upload terminal pair probe"),
            callback_count,
        )
        self.assertEqual(calls, 2)
        self.assertEqual(owner._state, owner.READY)
        owner.close(ledger)
        self._zero(ledger)

    def test_finish_slot_mutation_is_retryable_after_restoration(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        finish = carrier._effect_input_finish
        original = carrier._semantic_cleanup

        def mutate_finish(semantic, selected_carrier):
            original(semantic, selected_carrier)
            carrier._effect_input_finish = lambda: None

        carrier._semantic_cleanup = mutate_finish
        with self.assertRaisesRegex(self.CFError, "changed before finish"):
            carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.STAGED)
        carrier._effect_input_finish = finish
        carrier._semantic_cleanup = original
        carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.READY)
        owner.close(ledger)
        self._zero(ledger)

    def test_finish_slot_replacement_before_finalize_is_not_dispatched(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        finish = carrier._effect_input_finish
        replacement_called = False

        def replacement():
            nonlocal replacement_called
            replacement_called = True

        carrier._effect_input_finish = replacement
        with self.assertRaisesRegex(self.CFError, "finish slot changed"):
            carrier.finalize(ledger)
        self.assertFalse(replacement_called)
        carrier._effect_input_finish = finish
        carrier.finalize(ledger)
        owner.close(ledger)
        self._zero(ledger)

    def test_control_release_fault_retries_after_carrier_is_closed(self):
        ledger, _deadline, _request, carrier, _gate = self._prepared()
        owner = self.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        original = ledger.release
        calls = 0

        def fail_once(dimension, amount):
            nonlocal calls
            if dimension == "retained_bytes":
                calls += 1
            if dimension == "retained_bytes" and calls == 1:
                raise RuntimeError("effect input release fault")
            return original(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "release fault"):
            owner.close(ledger)
        self.assertFalse(carrier._closed)
        self.assertFalse(owner._closed)
        owner.close(ledger)
        del ledger.release
        self.assertGreaterEqual(calls, 2)
        self._zero(ledger)


if __name__ == "__main__":
    unittest.main()
