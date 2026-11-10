import unittest

from evoagent.seedsweep import SeedSweepPlanner


# cf-atom: TEST-seed-sweep-planner-dedups-and-truncates
class SeedSweepPlannerTests(unittest.TestCase):
    def test_seed_sweep_planner_rejects_duplicates_and_truncates(self):
        base = {"lr": 0.1}

        runs = SeedSweepPlanner().plan(base, [3, 1, 2, 4], 3)

        self.assertEqual(runs, [{"lr": 0.1, "seed": 3}, {"lr": 0.1, "seed": 1}, {"lr": 0.1, "seed": 2}])
        self.assertEqual(base, {"lr": 0.1})

    def test_seed_sweep_planner_rejects_duplicate_seed(self):
        with self.assertRaises(ValueError) as ctx:
            SeedSweepPlanner().plan({}, [3, 1, 3], 5)
        self.assertIn("3", str(ctx.exception))
        self.assertNotIn("1", str(ctx.exception))

    def test_seed_sweep_planner_rejects_bad_input(self):
        with self.assertRaises(ValueError) as ctx:
            SeedSweepPlanner().plan({}, [1], 0)
        self.assertEqual(str(ctx.exception), "max_runs must be an integer of at least 1")
        with self.assertRaises(ValueError):
            SeedSweepPlanner().plan({}, [1, "2"], 2)

    def test_seed_sweep_planner_rejects_negative_seed(self):
        with self.assertRaises(ValueError):
            SeedSweepPlanner().plan({}, [1, -2], 2)

    def test_seed_sweep_planner_max_seed(self):
        planner = SeedSweepPlanner()
        self.assertEqual(planner.plan({}, [1, 5], 2, max_seed=5), [{"seed": 1}, {"seed": 5}])
        self.assertEqual(planner.plan({}, [1, 99], 2), [{"seed": 1}, {"seed": 99}])
        with self.assertRaises(ValueError):
            planner.plan({}, [1, 6], 2, max_seed=5)

    def test_seed_sweep_planner_builds_range(self):
        self.assertEqual(SeedSweepPlanner.from_range(5, 3), [5, 6, 7])
        for bad in (0, -1, "2", True):
            with self.assertRaises(ValueError):
                SeedSweepPlanner.from_range(0, bad)
        with self.assertRaises(ValueError):
            SeedSweepPlanner.from_range(-1, 2)


if __name__ == "__main__":
    unittest.main()
