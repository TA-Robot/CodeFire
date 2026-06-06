import unittest

from evoagent.memory import ResearchMemoryIndex


# cf-atom: TEST-research-memory-index-queries-by-tag-kind-and-text
class ResearchMemoryIndexTests(unittest.TestCase):
    def test_research_memory_index_queries_by_tag_kind_and_text(self):
        index = ResearchMemoryIndex()
        failure = index.add(
            kind="failure",
            title="unstable seed on toy-tabular",
            summary="candidate regressed due to high variance feature selection",
            tags=("toy-tabular", "variance", "failure"),
        )
        decision = index.add(
            kind="decision",
            title="prefer cheap probes",
            summary="run cheap uncertainty reducers before expensive replication",
            tags=("scheduling", "cost"),
        )

        self.assertEqual(index.search(tag="TOY-TABULAR"), (failure,))
        self.assertEqual(index.search(kind="decision"), (decision,))
        self.assertEqual(index.search(text="variance"), (failure,))
        self.assertEqual(index.search(tag="cost", kind="decision", text="cheap"), (decision,))

    def test_research_memory_index_requires_kind_and_title(self):
        with self.assertRaises(ValueError):
            ResearchMemoryIndex().add(kind="", title="missing kind", summary="", tags=())


if __name__ == "__main__":
    unittest.main()
