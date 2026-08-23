import unittest

import tests.test_python_existing_effect_input as effect_input_tests


class PythonResponseBaselineTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        harness_type = effect_input_tests.PythonExistingEffectInputTests
        harness_type.setUpClass()
        cls.Harness = harness_type

    def setUp(self):
        self.harness = self.Harness(methodName="runTest")
        self.harness.setUp()

    def tearDown(self):
        self.harness.tearDown()

    def prepared(self, *, gated=False):
        result = self.harness._prepared(
            gated=gated, response_input=True
        )
        return (*result, self.harness.response_input)

    def test_response_exact_scalar_joins_sealed_baseline_and_terminal(self):
        ledger, _deadline, _request, carrier, _gate, response = self.prepared()
        effect_input = self.harness.Prepare(carrier, ledger)
        proof = carrier._pair_owner._sealed_owner._semantic_baseline_proof
        self.assertIs(response._transition, proof[-3])
        self.assertEqual(response._retained_exact, proof[-2])
        with self.assertRaisesRegex(self.harness.CFError, "adopted response"):
            response.close(ledger)
        carrier.finalize(ledger)
        self.assertEqual(effect_input._state, effect_input.READY)
        effect_input.close(ledger)
        self.assertTrue(response._closed)
        self.harness._zero(ledger)

    def test_parser_predecessor_is_rebased_from_upper_to_exact(self):
        ledger, _deadline, _request, carrier, _gate, response = self.prepared()
        semantic = carrier._semantic_owner
        scalar = semantic._source._scalar
        predecessor = response._transition._baseline_retained
        self.assertEqual(
            scalar._allocation._baseline_retained,
            predecessor + response._retained_exact,
        )
        self.assertIs(scalar._response_input, response)
        self.harness.Prepare(carrier, ledger).close(ledger)
        self.harness._zero(ledger)

    def test_last_callback_response_replacement_is_rejected(self):
        ledger, deadline, _request, carrier, _gate, response = self.prepared()
        effect_input = self.harness.Prepare(carrier, ledger)

        def mutate_response(_phase):
            response._success_body = bytes(bytearray(response._success_body))

        deadline.callback = mutate_response
        with self.assertRaisesRegex(
            self.harness.CFError, "response input changed"
        ):
            carrier.finalize(ledger)
        self.assertIsNone(effect_input._effect_semantic)
        effect_input.close(ledger)
        self.harness._zero(ledger)

    def test_gate_and_response_predecessors_share_one_exact_baseline(self):
        ledger, _deadline, _request, carrier, gate, response = self.prepared(
            gated=True
        )
        semantic = carrier._semantic_owner
        scalar = semantic._source._scalar
        self.assertEqual(
            response._transition._baseline_retained,
            gate._retained_upper,
        )
        self.assertEqual(
            scalar._allocation._baseline_retained,
            gate._retained_upper + response._retained_exact,
        )
        effect_input = self.harness.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        effect_input.close(ledger)
        self.harness._zero(ledger)

    def test_last_callback_response_class_mutation_is_not_dispatched(self):
        ledger, deadline, _request, carrier, _gate, response = self.prepared()
        effect_input = self.harness.Prepare(carrier, ledger)
        owner_type = type(response)
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
                self.harness.CFError, "response input changed"
            ):
                carrier.finalize(ledger)
        finally:
            owner_type.close = original
        self.assertFalse(replacement_called)
        effect_input.close(ledger)
        self.harness._zero(ledger)

    def test_last_callback_prepared_response_shell_mutation_is_rejected(self):
        ledger, deadline, _request, carrier, _gate, response = self.prepared()
        effect_input = self.harness.Prepare(carrier, ledger)

        def mutate_prepared_shell(_phase):
            response._prepared_response.header = b"unaccounted"

        deadline.callback = mutate_prepared_shell
        with self.assertRaisesRegex(
            self.harness.CFError, "response input changed"
        ):
            carrier.finalize(ledger)
        effect_input.close(ledger)
        self.harness._zero(ledger)


if __name__ == "__main__":
    unittest.main()
