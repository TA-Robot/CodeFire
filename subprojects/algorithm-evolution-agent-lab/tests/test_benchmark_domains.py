import unittest

from evoagent.benchmark_domains import BenchmarkDomainCatalog, BenchmarkDomainDefinition


# cf-atom: TEST-benchmark-domain-catalog-supports-multiple-domains
class BenchmarkDomainCatalogTests(unittest.TestCase):
    def test_benchmark_domain_catalog_supports_multiple_domains(self):
        catalog = BenchmarkDomainCatalog()
        tabular = catalog.add(BenchmarkDomainDefinition("toy-tabular", "tabular", "synthetic-v1", "accuracy"))
        sequence = catalog.add(BenchmarkDomainDefinition("toy-sequence", "sequence", "synthetic-seq-v1", "loss"))

        self.assertEqual(catalog.domains(), ("sequence", "tabular"))
        self.assertEqual(catalog.by_domain("tabular"), (tabular,))
        self.assertEqual(catalog.all(), (tabular, sequence))

    def test_benchmark_domain_catalog_rejects_duplicate_id(self):
        catalog = BenchmarkDomainCatalog()
        catalog.add(BenchmarkDomainDefinition("toy-tabular", "tabular", "synthetic-v1", "accuracy"))

        with self.assertRaises(ValueError):
            catalog.add(BenchmarkDomainDefinition("toy-tabular", "tabular", "synthetic-v2", "accuracy"))


if __name__ == "__main__":
    unittest.main()
