import unittest

from evoagent.safety import ClaimEvidenceSummary, ClaimReadiness, can_claim_sota, claim_readiness


# cf-atom: TEST-sota-claim-requires-evidence
class SotaClaimGateTests(unittest.TestCase):
    def test_sota_claim_requires_evidence(self):
        self.assertFalse(
            can_claim_sota(
                benchmark_results=0,
                baseline_comparisons=1,
                reproducible=True,
            )
        )
        self.assertFalse(
            can_claim_sota(
                benchmark_results=1,
                baseline_comparisons=0,
                reproducible=True,
            )
        )
        self.assertFalse(
            can_claim_sota(
                benchmark_results=1,
                baseline_comparisons=1,
                reproducible=False,
            )
        )
        self.assertTrue(
            can_claim_sota(
                benchmark_results=1,
                baseline_comparisons=1,
                reproducible=True,
            )
        )

    # cf-atom: TEST-claim-readiness-promotes-with-evidence
    def test_claim_readiness_promotes_with_evidence(self):
        self.assertEqual(claim_readiness(ClaimEvidenceSummary()), ClaimReadiness.IDEA)
        self.assertEqual(
            claim_readiness(ClaimEvidenceSummary(valid_runs=1, baseline_comparisons=1)),
            ClaimReadiness.LOCAL_IMPROVEMENT,
        )
        self.assertEqual(
            claim_readiness(
                ClaimEvidenceSummary(
                    valid_runs=3,
                    baseline_comparisons=2,
                    repeated_runs=3,
                    benchmark_protocol_satisfied=True,
                    ablations_included=True,
                    limitations_included=True,
                    reproduction_bundle_ready=True,
                    external_baseline_comparison=True,
                )
            ),
            ClaimReadiness.EXTERNAL_SOTA_CLAIM,
        )

    def test_claim_readiness_open_objection_blocks_external_claim(self):
        readiness = claim_readiness(
            ClaimEvidenceSummary(
                valid_runs=3,
                baseline_comparisons=2,
                repeated_runs=3,
                benchmark_protocol_satisfied=True,
                ablations_included=True,
                limitations_included=True,
                reproduction_bundle_ready=True,
                external_baseline_comparison=True,
                reviewer_objections_open=1,
            )
        )

        self.assertEqual(readiness, ClaimReadiness.BENCHMARK_CANDIDATE)


if __name__ == "__main__":
    unittest.main()
