import unittest

from tests import test_python_existing_effect_input as effect_harness


class PythonExistingResponseCompositionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        effect_harness.PythonExistingEffectInputTests.setUpClass()

    def setUp(self):
        self.harness = effect_harness.PythonExistingEffectInputTests(
            methodName="test_destination_first_transfer_is_opaque_and_closes_gate_owner_last"
        )
        self.harness.setUp()

    def tearDown(self):
        self.harness.tearDown()

    def _ready(self, response_values=None):
        ledger, deadline, request, carrier, gate = self.harness._prepared(
            gated=True,
            response_input=True,
            response_values=response_values,
        )
        response = self.harness.response_input
        owner = self.harness.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        self.assertEqual(owner._state, owner.READY)
        self.assertEqual(response._state, response.EFFECT_ADOPTED)
        self.assertIs(response._effect_input_owner, owner)
        self.assertIs(response._effect_input_token, owner._token)
        self.assertIsNotNone(response._semantic_owner)
        return ledger, owner, response, carrier, gate

    def test_success_selection_closes_gate_before_returning_prepared_bytes(self):
        ledger, owner, response, carrier, gate = self._ready()
        status, prepared = owner._take_prepared_response(
            ledger, durable=False
        )
        self.assertEqual(status, 200)
        self.assertEqual(prepared.header, b"success-header")
        self.assertEqual(prepared.body, b"success-body")
        self.assertTrue(owner._closed)
        self.assertTrue(response._closed)
        self.assertTrue(carrier._closed)
        self.assertTrue(gate._closed)
        self.assertEqual(
            ledger.current["retained_bytes"],
            len(b"success-header") + len(b"success-body"),
        )
        prepared.close()
        self.harness._zero(ledger)

    def test_durable_selection_discards_success_and_returns_503(self):
        ledger, owner, response, _carrier, _gate = self._ready()
        status, prepared = owner._take_prepared_response(
            ledger, durable=True
        )
        self.assertEqual(status, 503)
        self.assertEqual(prepared.header, b"durable-header")
        self.assertEqual(prepared.body, b"durable-body")
        self.assertTrue(response._closed)
        prepared.close()
        self.harness._zero(ledger)

    def test_abort_after_terminal_discards_both_responses(self):
        ledger, owner, response, carrier, gate = self._ready()
        owner.close(ledger)
        self.assertTrue(owner._closed)
        self.assertTrue(response._closed)
        self.assertTrue(carrier._closed)
        self.assertTrue(gate._closed)
        self.harness._zero(ledger)

    def test_release_fault_retries_only_the_original_selection(self):
        ledger, owner, response, carrier, gate = self._ready()
        original_release = ledger.release
        failed = [False]

        def fail_once(dimension, amount):
            if dimension == "retained_bytes" and not failed[0]:
                failed[0] = True
                raise RuntimeError("injected response selection release fault")
            return original_release(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "selection release fault"):
            owner._take_prepared_response(ledger, durable=True)
        self.assertEqual(response._state, response.SELECTING)
        self.assertFalse(carrier._closed)
        self.assertFalse(gate._closed)
        with self.assertRaisesRegex(
            self.harness.CFError, "selection is single-use"
        ):
            owner._take_prepared_response(ledger, durable=False)
        ledger.release = original_release
        status, prepared = owner._take_prepared_response(
            ledger, durable=True
        )
        self.assertEqual(status, 503)
        prepared.close()
        self.harness._zero(ledger)

    def test_response_alias_cannot_close_effect_owned_input(self):
        ledger, owner, response, _carrier, _gate = self._ready()
        with self.assertRaisesRegex(
            self.harness.CFError, "effect input adopted response"
        ):
            response.close(ledger)
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_abort_response_release_fault_is_retryable_by_same_owner(self):
        ledger, owner, response, carrier, _gate = self._ready()
        owner._carrier_close(
            carrier,
            ledger,
            _effect_input_owner=owner,
            _effect_input_token=owner._token,
        )
        self.assertTrue(carrier._closed)
        self.assertIsNone(response._semantic_owner)
        original_release = ledger.release
        failed = [False]
        response_retained = response._transition._retained_primary

        def fail_response_release(dimension, amount):
            if (
                dimension == "retained_bytes"
                and amount == response_retained
                and not failed[0]
            ):
                failed[0] = True
                raise RuntimeError("injected response close release fault")
            return original_release(dimension, amount)

        ledger.release = fail_response_release
        with self.assertRaisesRegex(RuntimeError, "close release fault"):
            owner.close(ledger)
        self.assertIs(response._effect_input_owner, owner)
        self.assertFalse(response._closed)
        ledger.release = original_release
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_selection_detects_state_change_during_scalar_shrink(self):
        ledger, owner, response, _carrier, _gate = self._ready()
        original_release = ledger.release
        changed = [False]

        def change_after_release(dimension, amount):
            result = original_release(dimension, amount)
            if dimension == "retained_bytes" and not changed[0]:
                changed[0] = True
                response._selected_status = 418
            return result

        ledger.release = change_after_release
        with self.assertRaisesRegex(
            self.harness.CFError, "changed during selection"
        ):
            owner._take_prepared_response(ledger, durable=True)
        ledger.release = original_release
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_transfer_detects_state_change_after_carrier_cleanup(self):
        ledger, owner, response, _carrier, _gate = self._ready()
        original_release = ledger.release
        changed = [False]

        def change_during_transfer(dimension, amount):
            result = original_release(dimension, amount)
            if (
                dimension == "retained_bytes"
                and amount == response.FIXED_RETAINED_BYTES
                and response._semantic_owner is None
                and response._state == response.SELECTED
                and not changed[0]
            ):
                changed[0] = True
                response._success_body = b"changed"
            return result

        ledger.release = change_during_transfer
        with self.assertRaisesRegex(
            self.harness.CFError, "changed during transfer"
        ):
            owner._take_prepared_response(ledger, durable=False)
        ledger.release = original_release
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_foreign_response_token_cannot_select(self):
        ledger, owner, response, _carrier, _gate = self._ready()
        retained_before = ledger.current["retained_bytes"]
        with self.assertRaisesRegex(
            self.harness.CFError, "selection is unavailable"
        ):
            owner._response_select(
                response, owner, object(), ledger, False
            )
        self.assertEqual(ledger.current["retained_bytes"], retained_before)
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_prepared_response_owns_the_only_remaining_scalar(self):
        ledger, owner, _response, _carrier, _gate = self._ready()
        _status, prepared = owner._take_prepared_response(
            ledger, durable=False
        )
        retained = prepared.retained_bytes
        self.assertGreater(retained, 0)
        self.assertEqual(ledger.current["retained_bytes"], retained)
        prepared.close()
        prepared.close()
        self.harness._zero(ledger)

    def test_empty_selected_response_transfers_without_a_scalar_handoff(self):
        ledger, owner, response, carrier, gate = self._ready(
            (200, b"", b"", b"", b"")
        )
        status, prepared = owner._take_prepared_response(
            ledger, durable=False
        )
        self.assertEqual(status, 200)
        self.assertEqual(prepared.header, b"")
        self.assertEqual(prepared.body, b"")
        self.assertEqual(prepared.retained_bytes, 0)
        self.assertTrue(response._closed)
        self.assertTrue(carrier._closed)
        self.assertTrue(gate._closed)
        prepared.close()
        self.harness._zero(ledger)


if __name__ == "__main__":
    unittest.main()
