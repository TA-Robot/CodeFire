import unittest

import tests.test_python_existing_effect_input as effect_input_tests


class PythonExistingResponseCompositionFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        harness_type = effect_input_tests.PythonExistingEffectInputTests
        harness_type.setUpClass()
        cls.Harness = harness_type
        cls.ns = harness_type.ns
        cls.Response = cls.ns["_PreparedHTTPResponseOwner"]
        cls.Responses = cls.ns["_PreparedHTTPMutationResponses"]

    def setUp(self):
        self.harness = self.Harness(methodName="runTest")
        self.harness.setUp()

    def tearDown(self):
        self.harness.tearDown()

    def _unaccounted_response_shells(self, ledger):
        # Characterization-only stand-in after the preceding test proves that
        # normal response accounting cannot be added after the sealed baseline.
        # It isolates the two remaining ordering gaps without pretending this
        # is a production construction path.
        success = self.Response(ledger, b"success-header", 0, b"success", 0)
        durable = self.Response(ledger, b"durable-header", 0, b"durable", 0)
        return self.Responses(
            ledger,
            success,
            durable,
            None,
            None,
            len(success.body),
            len(durable.body),
            len(success.header),
            len(durable.header),
        )

    def _accounted_responses(self, ledger):
        success_header = b"success-header"
        success_body = b"success"
        durable_header = b"durable-header"
        durable_body = b"durable"
        retained = sum(
            map(
                len,
                (
                    success_header,
                    success_body,
                    durable_header,
                    durable_body,
                ),
            )
        )
        ledger.acquire("retained_bytes", retained, "response_body")
        success = self.Response(
            ledger,
            success_header,
            len(success_header),
            success_body,
            len(success_body),
        )
        durable = self.Response(
            ledger,
            durable_header,
            len(durable_header),
            durable_body,
            len(durable_body),
        )
        response_permit = ledger.reserve(
            "response_body",
            charges={"response_bytes": max(len(success_body), len(durable_body))},
        )
        header_permit = ledger.reserve(
            "response_headers",
            charges={
                "response_header_bytes": max(
                    len(success_header), len(durable_header)
                )
            },
        )
        responses = self.Responses(
            ledger,
            success,
            durable,
            response_permit,
            header_permit,
            len(success_body),
            len(durable_body),
            len(success_header),
            len(durable_header),
        )
        responses.seal_for_effect(ledger)
        return responses

    def test_accounted_response_after_sealed_baseline_is_rejected(self):
        ledger, _deadline, _request, carrier, _gate = self.harness._prepared()
        effect_input = self.harness.Prepare(carrier, ledger)
        responses = self._accounted_responses(ledger)

        with self.assertRaises(self.harness.CFError):
            carrier.finalize(ledger)

        responses.close()
        effect_input.close(ledger)
        self.harness._zero(ledger)

    def test_preterminal_response_can_change_in_the_last_callback(self):
        ledger, deadline, _request, carrier, _gate = self.harness._prepared()
        effect_input = self.harness.Prepare(carrier, ledger)
        responses = self._unaccounted_response_shells(ledger)
        original = responses.success.body

        def mutate_response(_phase):
            responses.success.body = b"changed-after-response-freeze"

        deadline.callback = mutate_response
        carrier.finalize(ledger)

        self.assertEqual(effect_input._state, effect_input.READY)
        self.assertNotEqual(responses.success.body, original)
        self.assertEqual(
            responses.success.body, b"changed-after-response-freeze"
        )
        effect_input.close(ledger)
        responses.close()
        self.harness._zero(ledger)

    def test_postterminal_response_callback_can_change_gate_subject(self):
        ledger, deadline, _request, carrier, gate = self.harness._prepared(
            gated=True
        )
        effect_input = self.harness.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        descriptor = self.harness.project / "remote_project.json"
        original = descriptor.read_bytes()

        def mutate_descriptor(_phase):
            descriptor.write_bytes(original + b" ")

        original_check = deadline.check
        deadline.check = mutate_descriptor
        deadline.check("upload mutation publication")
        deadline.check = original_check

        self.assertEqual(effect_input._state, effect_input.READY)
        self.assertNotEqual(descriptor.read_bytes(), original)
        self.assertFalse(gate._closed)
        descriptor.write_bytes(original)
        effect_input.close(ledger)
        self.harness._zero(ledger)

    def test_response_selection_has_no_effect_input_adoption_token(self):
        ledger, _deadline, _request, carrier, _gate = self.harness._prepared()
        effect_input = self.harness.Prepare(carrier, ledger)
        responses = self._unaccounted_response_shells(ledger)
        carrier.finalize(ledger)

        selected = responses.select_success(ledger)
        self.assertIsNone(responses.success)
        self.assertIsNone(responses.durable)
        self.assertEqual(selected.body, b"success")
        self.assertIs(ledger._upload_terminal_owner, effect_input)

        effect_input.close(ledger)
        self.assertEqual(selected.body, b"success")
        selected.close()
        responses.close()
        self.harness._zero(ledger)


if __name__ == "__main__":
    unittest.main()
