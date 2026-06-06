import unittest

from evoagent.roles import RoleComponent, RoleRegistry


# cf-atom: TEST-role-registry-registers-extensible-components
class RoleRegistryTests(unittest.TestCase):
    def test_role_registry_registers_extensible_components(self):
        registry = RoleRegistry()
        planner = registry.register(RoleComponent("planner", "static-planner", ("hypothesis_generation",)))
        runner = registry.register(RoleComponent("runner", "local-runner", ("local_command", "artifact_capture")))

        self.assertEqual(registry.get("planner"), planner)
        self.assertTrue(registry.supports("runner", "artifact_capture"))
        self.assertEqual(registry.all(), (planner, runner))

    def test_role_registry_rejects_duplicate_role(self):
        registry = RoleRegistry()
        registry.register(RoleComponent("planner", "static-planner", ("hypothesis_generation",)))

        with self.assertRaises(ValueError):
            registry.register(RoleComponent("planner", "codex-planner", ("hypothesis_generation",)))


if __name__ == "__main__":
    unittest.main()
