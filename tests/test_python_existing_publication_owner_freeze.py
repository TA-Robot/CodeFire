import unittest
from pathlib import Path

from tests import test_python_existing_effect_input as effect_harness


class PythonExistingPublicationOwnerFreezeTests(unittest.TestCase):
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
        self.PreparePublication = self.harness.ns[
            "prepare_http_upload_existing_publication_input"
        ]

    def tearDown(self):
        self.harness.tearDown()

    def _ready(self):
        ledger, _deadline, _request, carrier, gate = self.harness._prepared(
            gated=True,
            response_input=True,
        )
        effect = self.harness.Prepare(carrier, ledger)
        carrier.finalize(ledger)
        return ledger, effect, gate

    def _publication(self, ledger, project_root=None):
        plan = {
            "project_root": project_root or self.harness.project,
            "operation": "remote_upload",
            "actor": "alice",
            "writes": [
                (
                    {"kind": "generation"},
                    None,
                    {"current_generation": 1},
                )
            ],
            "result": {"branch": "main", "head": self.harness.head},
            "idempotency_key": None,
            "object_roots": [self.harness.head],
        }
        values = [
            plan["operation"],
            plan["actor"],
            plan["idempotency_key"],
            plan["object_roots"],
            plan["result"],
        ]
        for write in plan["writes"]:
            values.extend(write)
        canonical = self.harness.ns["canonical_json"]
        upper = sum(len(canonical(value)) for value in values)
        publication = self.PreparePublication(
            ledger, canonical_upper=upper
        )
        publication.prepare(ledger, **plan)
        return publication

    def test_late_publication_input_does_not_adopt_terminal_registry(self):
        ledger, effect, _gate = self._ready()
        publication = self._publication(ledger)
        self.assertEqual(publication._state, publication.PREPARED)
        self.assertIs(ledger._upload_terminal_owner, effect)
        self.assertFalse(hasattr(publication, "_effect_input_owner"))
        self.assertFalse(hasattr(publication, "_effect_input_token"))
        publication.close(ledger)
        effect.close(ledger)
        self.harness._zero(ledger)

    def test_effect_response_transfer_destroys_inputs_before_publication_adoption(self):
        ledger, effect, gate = self._ready()
        publication = self._publication(ledger)
        _status, prepared = effect._take_prepared_response(
            ledger, durable=False
        )
        self.assertTrue(effect._closed)
        self.assertTrue(gate._closed)
        self.assertIsNone(effect._effect_semantic)
        self.assertIsNone(effect._effect_copy_bytes)
        self.assertEqual(publication._state, publication.PREPARED)
        self.assertIsNone(ledger._upload_terminal_owner)
        prepared.close()
        publication.close(ledger)
        self.harness._zero(ledger)

    def test_publication_input_is_not_bound_to_the_stable_project_subject(self):
        ledger, effect, _gate = self._ready()
        publication = self._publication(
            ledger, Path("/different/unpinned/project")
        )
        self.assertEqual(publication._state, publication.PREPARED)
        self.assertFalse(hasattr(publication, "_stable_gate_owner"))
        self.assertFalse(hasattr(publication, "_stable_gate_token"))
        publication.close(ledger)
        effect.close(ledger)
        self.harness._zero(ledger)

    def test_preimage_change_is_not_observed_by_prepared_publication_input(self):
        ledger, effect, _gate = self._ready()
        publication = self._publication(ledger)
        generation = self.harness.project / "gc_state.json"
        generation.write_bytes(b'{"current_generation":99}\n')
        self.assertEqual(publication._state, publication.PREPARED)
        self.assertFalse(hasattr(publication, "validate_fresh_preimages"))
        publication.close(ledger)
        effect.close(ledger)
        self.harness._zero(ledger)


if __name__ == "__main__":
    unittest.main()
