import unittest

from tests import test_python_existing_effect_input as effect_harness


class PythonExistingPublicationCompositionFreezeTests(unittest.TestCase):
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

    def _ready(self):
        ledger, _deadline, _request, carrier, _gate = self.harness._prepared(
            gated=True,
            response_input=True,
        )
        owner = self.harness.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        return ledger, owner

    def _plan(self, owner):
        semantic = dict(owner._effect_semantic)
        writes = [
            (
                {"kind": "generation"},
                None,
                {"current_generation": 1},
            )
        ]
        result = {"branch": "main", "head": self.harness.head}
        return semantic, writes, result

    def _preflight(self, semantic, writes, result):
        return self.harness.ns["preflight_remote_json_publication"](
            self.harness.project,
            "remote_upload",
            "alice",
            semantic,
            writes,
            result,
            None,
            [self.harness.head],
            validation_objects_root=self.harness.objects,
        )

    def test_effect_semantic_mapping_cannot_enter_legacy_preflight_directly(self):
        ledger, owner = self._ready()
        _semantic, writes, result = self._plan(owner)
        with self.assertRaisesRegex(TypeError, "JSON serializable"):
            self._preflight(owner._effect_semantic, writes, result)
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_legacy_preflight_has_no_ledger_token_or_cleanup_owner(self):
        ledger, owner = self._ready()
        semantic, writes, result = self._plan(owner)
        preflight = self._preflight(semantic, writes, result)
        self.assertFalse(hasattr(preflight, "_ledger"))
        self.assertFalse(hasattr(preflight, "_effect_input_token"))
        self.assertFalse(hasattr(preflight, "close"))
        owner.close(ledger)
        self.harness._zero(ledger)

    def test_effect_close_does_not_revoke_sealed_legacy_preflight(self):
        ledger, owner = self._ready()
        semantic, writes, result = self._plan(owner)
        preflight = self._preflight(semantic, writes, result)
        preflight.seal_for_effect(
            self.harness.project,
            "remote_upload",
            "alice",
            semantic,
            writes,
            result,
            None,
            (self.harness.head,),
        )
        owner.close(ledger)
        self.harness._zero(ledger)
        preflight.consume_for_effect(
            self.harness.project,
            "remote_upload",
            "alice",
            semantic,
            writes,
            result,
            None,
            (self.harness.head,),
        )
        self.assertTrue(preflight._consumed)

    def test_response_abort_does_not_change_publication_preflight_state(self):
        ledger, owner = self._ready()
        semantic, writes, result = self._plan(owner)
        preflight = self._preflight(semantic, writes, result)
        self.assertFalse(preflight._sealed)
        self.assertFalse(preflight._consumed)
        owner.close(ledger)
        self.assertFalse(preflight._sealed)
        self.assertFalse(preflight._consumed)
        self.harness._zero(ledger)


if __name__ == "__main__":
    unittest.main()
