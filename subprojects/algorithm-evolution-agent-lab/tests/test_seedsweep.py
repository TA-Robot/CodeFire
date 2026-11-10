import unittest

from evoagent.seedsweep import SeedSweepPlanner


# cf-atom: TEST-seed-sweep-planner-dedups-and-truncates
class SeedSweepPlannerTests(unittest.TestCase):
    def test_seed_sweep_planner_dedups_and_truncates(self):
        base = {"lr": 0.1}

        runs = SeedSweepPlanner().plan(base, [3, 1, 3, 2, 1, 4], 3)

        self.assertEqual(runs, [{"lr": 0.1, "seed": 3}, {"lr": 0.1, "seed": 1}, {"lr": 0.1, "seed": 2}])
        self.assertEqual(base, {"lr": 0.1})

    def test_seed_sweep_planner_rejects_bad_input(self):
        with self.assertRaises(ValueError):
            SeedSweepPlanner().plan({}, [1], 0)
        with self.assertRaises(ValueError):
            SeedSweepPlanner().plan({}, [1, "2"], 2)


if __name__ == "__main__":
    unittest.main()
