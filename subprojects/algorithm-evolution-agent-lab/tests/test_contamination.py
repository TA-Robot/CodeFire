import unittest

from evoagent.contamination import ContaminationSource, ContaminationSourceRegistry


# cf-atom: TEST-contamination-source-registry-flags-dataset-and-artifact-overlap
class ContaminationSourceRegistryTests(unittest.TestCase):
    def test_contamination_source_registry_flags_dataset_and_artifact_overlap(self):
        registry = ContaminationSourceRegistry()
        registry.add(
            ContaminationSource(
                "src1",
                "hidden-test-v1",
                ("hash-a", "hash-b"),
                notes="known public leak",
            )
        )

        findings = registry.audit(dataset="hidden-test-v1", artifact_hashes=("hash-b", "hash-c"))

        self.assertEqual(
            {finding.kind for finding in findings},
            {"dataset_overlap", "artifact_overlap"},
        )


if __name__ == "__main__":
    unittest.main()
