import unittest

from evoagent.traceability import CodeFireTraceAudit, TraceLink


# cf-atom: TEST-codefire-trace-audit-connects-requirements-code-tests
class CodeFireTraceAuditTests(unittest.TestCase):
    def test_codefire_trace_audit_connects_requirements_code_tests(self):
        links = (
            TraceLink("REQ-X-001", "DES-X-001", "refined_by"),
            TraceLink("DES-X-001", "CODE-X", "implemented_by"),
            TraceLink("REQ-X-001", "TEST-X", "verified_by"),
        )

        report = CodeFireTraceAudit().audit(links)

        self.assertTrue(report.passed)
        self.assertEqual(report.requirements, ("REQ-X-001",))
        self.assertEqual(report.implementations, ("CODE-X",))
        self.assertEqual(report.tests, ("TEST-X",))

    def test_codefire_trace_audit_flags_missing_code_and_tests(self):
        links = (TraceLink("REQ-X-001", "DES-X-001", "refined_by"),)

        report = CodeFireTraceAudit().audit(links)

        self.assertFalse(report.passed)
        self.assertEqual(
            {finding.kind for finding in report.findings},
            {"missing_implementation", "missing_test"},
        )


if __name__ == "__main__":
    unittest.main()
