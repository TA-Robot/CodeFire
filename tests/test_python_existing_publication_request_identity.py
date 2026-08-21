import unittest
from tests import test_python_existing_effect_input as effect_harness


class PythonExistingPublicationRequestIdentityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        effect_harness.PythonExistingEffectInputTests.setUpClass()

    def setUp(self):
        self.harness = effect_harness.PythonExistingEffectInputTests(
            methodName=(
                "test_destination_first_transfer_is_opaque_and_closes_gate_owner_last"
            )
        )
        self.harness.setUp()

    def tearDown(self):
        self.harness.tearDown()

    def _prepared(self):
        ledger, deadline, request, carrier, gate = self.harness._prepared(
            gated=True,
            response_input=True,
            publication_input=True,
        )
        publication = self.harness.publication_input
        semantic = carrier._semantic_owner
        return ledger, deadline, request, carrier, gate, semantic, publication

    def _unit_identity(
        self,
        *,
        actor="alice",
        idempotency_key=None,
        roots=None,
        semantic_value=None,
        semantic_upper_adjustment=0,
    ):
        roots = [] if roots is None else roots
        ledger, _deadline, _request, semantic, _gate = self.harness._prepared(
            semantic_only=True
        )
        if semantic_value is not None:
            raise AssertionError("real semantic fixture has a fixed mapping")
        semantic_value = semantic._semantic
        plan = {
            "project_root": self.harness.project,
            "operation": "remote_upload",
            "actor": actor,
            "writes": [
                (
                    {"kind": "generation"},
                    None,
                    {"current_generation": 1},
                )
            ],
            "result": {},
            "idempotency_key": idempotency_key,
            "object_roots": roots,
        }
        canonical = self.harness.ns["canonical_json"]
        values = [
            plan["operation"],
            plan["actor"],
            plan["idempotency_key"],
            plan["object_roots"],
            plan["result"],
        ]
        for write in plan["writes"]:
            values.extend(write)
        upper = sum(len(canonical(value)) for value in values)
        semantic_upper = (
            len(canonical(semantic_value)) + semantic_upper_adjustment
        )
        publication = self.harness.ns[
            "prepare_http_upload_existing_publication_input"
        ](
            ledger,
            canonical_upper=upper,
            semantic_upper=semantic_upper,
        )
        publication.prepare(ledger, **plan)
        return ledger, publication, semantic, plan, semantic_value

    def _close_unit(self, ledger, publication, semantic):
        semantic.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_request_hash_matches_legacy_domain_without_payload_getter(self):
        ledger, _deadline, _request, carrier, _gate, semantic, publication = (
            self._prepared()
        )
        expected = self.harness.ns["remote_publication_request_hash"](
            "remote_upload",
            "alice",
            dict(semantic._semantic_view),
            None,
            [self.harness.head],
        )
        self.assertEqual(publication._request_hash, expected)
        self.assertEqual(len(publication._request_hash), 64)
        self.assertEqual(publication._state, publication.IDENTITY_BOUND)
        for name in ("request_hash", "payload", "semantic", "effect"):
            self.assertFalse(hasattr(publication, name))
        carrier.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_sealed_baseline_contains_exact_publication_transition(self):
        ledger, _deadline, _request, carrier, _gate, semantic, publication = (
            self._prepared()
        )
        pair = carrier._pair_owner
        proof = pair._sealed_owner._semantic_baseline_proof
        self.assertIs(proof[20], publication)
        self.assertIs(proof[21], publication._semantic_token)
        self.assertIs(proof[22], publication._transition)
        self.assertEqual(proof[23], publication._retained_exact)
        self.assertEqual(proof[25], publication._request_hash)
        self.assertTrue(
            self.harness.ns["AccountedHTTPUploadSealedValidation"]
            ._semantic_baseline_intact(proof, require_current=False)
        )
        carrier.close(ledger)
        self.harness._zero(ledger)

    def test_publication_alias_cannot_close_semantic_owned_identity(self):
        ledger, _deadline, _request, carrier, _gate, _semantic, publication = (
            self._prepared()
        )
        with self.assertRaisesRegex(
            self.harness.CFError, "semantic adopted publication"
        ):
            publication.close(ledger)
        carrier.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_last_callback_semantic_change_rejects_before_effect_adoption(self):
        ledger, deadline, _request, carrier, _gate, semantic, publication = (
            self._prepared()
        )
        effect = self.harness.Prepare(carrier, ledger)
        original = semantic._semantic["head"]
        replacement = original[:-1] + ("0" if original[-1] != "0" else "1")
        deadline.callback = lambda _phase: semantic._semantic.__setitem__(
            "head", replacement
        )
        with self.assertRaises(self.harness.CFError):
            carrier.finalize(ledger)
        self.assertIsNone(effect._effect_semantic)
        self.assertTrue(publication._closed)
        effect.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_foreign_semantic_owner_cannot_bind_identity(self):
        ledger, _deadline, _request, carrier, _gate, semantic, publication = (
            self._prepared()
        )
        foreign_ledger = self.harness.Ledger(
            self.harness.Profile.http_v1(),
            "foreign publication identity",
            command_kind="http_request",
        )
        with self.assertRaisesRegex(
            self.harness.CFError, "another ledger"
        ):
            publication._bind_request_identity(semantic, foreign_ledger)
        carrier.close(ledger)
        self.harness._zero(ledger)
        self.harness._zero(foreign_ledger)

    def test_unicode_actor_none_key_and_empty_roots_match_legacy_hash(self):
        ledger, publication, semantic, plan, semantic_value = (
            self._unit_identity(actor="アリス", roots=[])
        )
        publication._bind_request_identity(semantic, ledger)
        expected = self.harness.ns["remote_publication_request_hash"](
            plan["operation"],
            plan["actor"],
            semantic_value,
            None,
            [],
        )
        self.assertEqual(publication._request_hash, expected)
        self._close_unit(ledger, publication, semantic)

    def test_equal_length_actor_change_produces_a_distinct_hash(self):
        hashes = []
        for actor in ("alice", "carol"):
            ledger, publication, semantic, _plan, _semantic_value = (
                self._unit_identity(actor=actor)
            )
            publication._bind_request_identity(semantic, ledger)
            hashes.append(publication._request_hash)
            self._close_unit(ledger, publication, semantic)
        self.assertNotEqual(hashes[0], hashes[1])

    def test_semantic_upper_exact_minus_one_rejects_before_hash(self):
        ledger, publication, semantic, _plan, _semantic_value = (
            self._unit_identity(semantic_upper_adjustment=-1)
        )
        with self.assertRaisesRegex(
            self.harness.CFError, "exceeds its upper"
        ):
            publication._bind_request_identity(semantic, ledger)
        self.assertIsNone(publication._request_hash)
        publication.close(ledger)
        semantic.close(ledger)
        self.harness._zero(ledger)

    def test_uninitialized_exact_semantic_shell_is_rejected(self):
        ledger, publication, semantic, _plan, _semantic_value = (
            self._unit_identity()
        )
        fake = object.__new__(type(semantic))
        fake._ledger = ledger
        fake._ledger_identity = id(ledger)
        with self.assertRaisesRegex(
            self.harness.CFError, "semantic identity is incomplete"
        ):
            publication._bind_request_identity(fake, ledger)
        publication.close(ledger)
        semantic.close(ledger)
        self.harness._zero(ledger)

    def test_cloned_semantic_shell_cannot_replace_constructor_identity(self):
        ledger, publication, semantic, _plan, _semantic_value = (
            self._unit_identity()
        )
        fake = object.__new__(type(semantic))
        fake.__dict__.update(semantic.__dict__)
        source = semantic._source
        source._destination = fake
        try:
            with self.assertRaisesRegex(
                self.harness.CFError, "semantic identity is unavailable"
            ):
                publication._bind_request_identity(fake, ledger)
        finally:
            source._destination = semantic
            fake._pair_semantic_carrier_owner = object()
        publication.close(ledger)
        semantic.close(ledger)
        self.harness._zero(ledger)

    def test_last_callback_publication_plan_replacement_is_rejected(self):
        ledger, deadline, _request, carrier, _gate, _semantic, publication = (
            self._prepared()
        )
        effect = self.harness.Prepare(carrier, ledger)
        original = publication._identity_parts
        deadline.callback = lambda _phase: setattr(
            publication, "_identity_parts", tuple([*original])
        )
        with self.assertRaisesRegex(
            self.harness.CFError, "publication input changed"
        ):
            carrier.finalize(ledger)
        self.assertIsNone(effect._effect_semantic)
        effect.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_last_callback_publication_class_mutation_is_not_dispatched(self):
        ledger, deadline, _request, carrier, _gate, _semantic, publication = (
            self._prepared()
        )
        effect = self.harness.Prepare(carrier, ledger)
        owner_type = type(publication)
        original = owner_type.close
        replacement_called = False

        def replacement(*_args, **_kwargs):
            nonlocal replacement_called
            replacement_called = True

        deadline.callback = lambda _phase: setattr(
            owner_type, "close", replacement
        )
        try:
            with self.assertRaisesRegex(
                self.harness.CFError, "publication input changed"
            ):
                carrier.finalize(ledger)
        finally:
            owner_type.close = original
        self.assertFalse(replacement_called)
        effect.close(ledger)
        self.assertTrue(publication._closed)
        self.harness._zero(ledger)

    def test_work_upper_includes_fixed_request_envelope_for_empty_semantic(self):
        canonical = self.harness.ns["canonical_json"]
        plan_values = [
            "remote_upload",
            "alice",
            None,
            [],
            {},
            {"kind": "generation"},
            None,
            {"current_generation": 1},
        ]
        plan_upper = sum(len(canonical(value)) for value in plan_values)
        semantic_upper = len(canonical({}))
        Owner = self.harness.ns[
            "AccountedHTTPUploadExistingPublicationInput"
        ]
        work_upper = (
            Owner.WORK_BYTES_PER_CANONICAL_BYTE * plan_upper
            + Owner.REQUEST_IDENTITY_BYTES_PER_PLAN_BYTE * plan_upper
            + Owner.REQUEST_IDENTITY_BYTES_PER_SEMANTIC_BYTE * semantic_upper
            + Owner.REQUEST_IDENTITY_FIXED_WORK_BYTES
        )
        ledger = self.harness.Ledger(
            self.harness.Profile.http_v1(
                {"aggregate_bytes": work_upper - 1}
            ),
            "publication request identity work minus one",
            command_kind="http_request",
        )
        with self.assertRaises(self.harness.ns["ResourceBudgetExceeded"]):
            self.harness.ns[
                "prepare_http_upload_existing_publication_input"
            ](
                ledger,
                canonical_upper=plan_upper,
                semantic_upper=semantic_upper,
            )
        self.harness._zero(ledger)

        ledger = self.harness.Ledger(
            self.harness.Profile.http_v1(
                {"aggregate_bytes": work_upper}
            ),
            "publication request identity work exact",
            command_kind="http_request",
        )
        publication = self.harness.ns[
            "prepare_http_upload_existing_publication_input"
        ](
            ledger,
            canonical_upper=plan_upper,
            semantic_upper=semantic_upper,
        )
        publication.close(ledger)
        self.harness._zero(ledger)

    def test_identity_temporary_release_fault_retries_without_rehash_owner_loss(self):
        ledger, publication, semantic, _plan, _semantic_value = (
            self._unit_identity()
        )
        original = ledger.release
        failed = False

        def fail_once(dimension, amount):
            nonlocal failed
            if dimension == "temporary_bytes" and not failed:
                failed = True
                raise RuntimeError("identity temporary release fault")
            return original(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "temporary release fault"):
            publication._bind_request_identity(semantic, ledger)
        self.assertEqual(publication._state, publication.IDENTITY_BINDING)
        request_hash = publication._request_hash
        del ledger.release
        publication._bind_request_identity(semantic, ledger)
        self.assertEqual(publication._state, publication.IDENTITY_BOUND)
        self.assertEqual(publication._request_hash, request_hash)
        self.assertTrue(failed)
        self._close_unit(ledger, publication, semantic)


if __name__ == "__main__":
    unittest.main()
