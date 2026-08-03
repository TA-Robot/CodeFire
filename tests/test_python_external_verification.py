import dis
import gc
import hashlib
import json
import os
import runpy
import shlex
import sys
import tempfile
import time
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonExternalVerificationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.project = self.tmp / "project"
        self.project.mkdir()
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.Owner = self.namespace["AccountedServerVerification"]
        self.Ledger = self.namespace["DecisionWorkLedger"]
        self.Profile = self.namespace["DecisionWorkProfile"]
        self.prepare = self.namespace["prepare_server_verification"]
        self.commit_id = "CF-COMMIT-" + "a" * 24

    def tearDown(self):
        for path in sorted(self.tmp.rglob("*"), reverse=True):
            if path.is_file() or path.is_symlink():
                path.unlink()
            elif path.is_dir():
                path.rmdir()
        self.tmp.rmdir()

    def ledger(self, overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            "HTTP external verification",
            command_kind="http_request",
        )

    def write_policy(self, checks):
        (self.project / "server_policy.json").write_text(
            json.dumps({"verification": {"required": checks}}),
            encoding="utf-8",
        )

    def command_output(self, stream, count):
        executable = shlex.quote(sys.executable)
        return (
            f'{executable} -c "import sys;'
            f"sys.{stream}.buffer.write(b'x'*{count})"
            '"'
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def authority_snapshot(self):
        relative_paths = [
            "gc_state.json",
            "request_nonce_cache.json",
            "pending_transaction.json",
            "branches/main.json",
            "idempotency/v2/upload/receipt.json",
        ]
        return {
            relative: (
                (self.project / relative).read_bytes()
                if (self.project / relative).is_file()
                else None
            )
            for relative in relative_paths
        }

    def test_bounded_stdout_stderr_limit_and_pre_spawn_budget(self):
        output_limit = 64
        for stream, count, succeeds in [
            ("stdout", output_limit - 1, True),
            ("stderr", output_limit, True),
            ("stdout", output_limit + 1, False),
        ]:
            with self.subTest(stream=stream, count=count):
                self.write_policy(
                    [
                        {
                            "id": f"{stream}-{count}",
                            "command": self.command_output(stream, count),
                        }
                    ]
                )
                ledger = self.ledger()
                owner = self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=output_limit,
                )
                try:
                    if succeeds:
                        owner.run()
                    else:
                        with self.assertRaisesRegex(
                            self.namespace["CFError"], "output exceeds"
                        ):
                            owner.run()
                finally:
                    owner.close()
                self.assert_closed(ledger)

        output_temporary = (
            self.Owner.OUTPUT_MEMORY_BASE_BYTES
            + self.Owner.OUTPUT_MEMORY_MULTIPLIER * (output_limit + 1)
            + self.Owner.OUTPUT_READ_CHUNK_BYTES
        )
        self.write_policy(
            [{"id": "budget-before-spawn", "command": "exit 0"}]
        )
        rejected_ledger = self.ledger(
            {"temporary_bytes": ("concurrent", output_temporary - 1)}
        )
        spawn = self.Owner._spawn_process
        spawn_calls = 0

        def observe_spawn(*args):
            nonlocal spawn_calls
            spawn_calls += 1
            return spawn(*args)

        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    rejected_ledger,
                    output_limit_bytes=output_limit,
                )
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
        self.assertEqual(spawn_calls, 0)
        self.assert_closed(rejected_ledger)

        now = [0.0]
        deadline = self.namespace["HTTPRequestDeadline"](1, lambda: now[0])
        deadline_ledger = self.ledger()
        deadline_owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            deadline_ledger,
            deadline,
            output_limit_bytes=output_limit,
        )
        spawn_calls = 0
        self.Owner._spawn_process = staticmethod(observe_spawn)
        now[0] = 0.001
        try:
            with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
                deadline_owner.run()
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            deadline_owner.close()
        self.assertEqual(spawn_calls, 0)
        self.assert_closed(deadline_ledger)

    def test_failure_exception_uses_bounded_sample_digest(self):
        output_limit = 1024 * 1024
        executable = shlex.quote(sys.executable)
        command = (
            f'{executable} -c "import sys;'
            f"sys.stdout.buffer.write(b'x'*{output_limit});sys.exit(7)"
            '"'
        )
        self.write_policy([{"id": "bounded-failure", "command": command}])
        ledger = self.ledger()
        owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=output_limit,
        )
        failure = None
        try:
            owner.run()
        except self.namespace["CFError"] as exc:
            failure = exc
        self.assertIsNotNone(failure)
        message = failure.args[0]
        expected_digest = hashlib.sha256(b"x" * output_limit).hexdigest()
        self.assertLessEqual(
            len(message.encode("utf-8")),
            self.Owner.FAILURE_CONTROL_LIMIT_BYTES,
        )
        self.assertIn(f"total_bytes={output_limit}", message)
        self.assertIn(f"sha256={expected_digest}", message)
        self.assertLess(len(message), output_limit // 4)
        self.assertEqual(
            ledger.cumulative.get("aggregate_bytes", 0),
            2 * (self.project / "server_policy.json").stat().st_size
            + output_limit
            + len(message.encode("utf-8")),
        )
        owner.close()
        self.assertEqual(message, failure.args[0])
        self.assert_closed(ledger)

    def test_policy_ast_upper_precedes_parse_and_dominates_owner(self):
        policy = {
            "verification": {"required": []},
            "junk": [{} for _ in range(10_000)],
        }
        raw = json.dumps(policy, separators=(",", ":")).encode("utf-8")
        (self.project / "server_policy.json").write_bytes(raw)
        ast_upper = (
            self.Owner.POLICY_AST_BASE_BYTES
            + self.Owner.POLICY_AST_BYTES_PER_INPUT_BYTE * len(raw)
        )
        parse_upper = (
            self.Owner.POLICY_PARSE_BASE_BYTES
            + self.Owner.POLICY_PARSE_BYTES_PER_INPUT_BYTE * len(raw)
        )
        capability_retained, capability_temporary = (
            self.Owner._capability_upper(self.project)
        )
        loads = json.loads
        observations = []

        def observe_loads(document):
            observations.append(
                (
                    ledger.current.get("retained_bytes", 0),
                    ledger.current.get("temporary_bytes", 0),
                )
            )
            return loads(document)

        ledger = self.ledger()
        json.loads = observe_loads
        try:
            owner = self.prepare(
                self.project,
                "main",
                self.commit_id,
                ledger,
                output_limit_bytes=0,
            )
        finally:
            json.loads = loads
        self.assertEqual(
            observations,
            [
                (
                    capability_retained + ast_upper,
                    capability_temporary + parse_upper,
                )
            ],
        )

        def deep_size(value):
            seen = set()
            stack = [value]
            total = 0
            while stack:
                item = stack.pop()
                identity = id(item)
                if identity in seen:
                    continue
                seen.add(identity)
                total += sys.getsizeof(item)
                if isinstance(item, dict):
                    stack.extend(item.keys())
                    stack.extend(item.values())
                elif isinstance(item, (list, tuple, set)):
                    stack.extend(item)
            return total

        self.assertGreaterEqual(
            ledger.current.get("retained_bytes", 0),
            deep_size(owner._policy) + deep_size(owner._plans),
        )
        owner.close()
        self.assert_closed(ledger)

        rejected = self.ledger(
            {
                "retained_bytes": (
                    "concurrent",
                    capability_retained + ast_upper - 1,
                )
            }
        )
        parse_calls = 0

        def count_loads(document):
            nonlocal parse_calls
            parse_calls += 1
            return loads(document)

        json.loads = count_loads
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    rejected,
                    output_limit_bytes=0,
                )
        finally:
            json.loads = loads
        self.assertEqual(parse_calls, 0)
        self.assert_closed(rejected)

    def test_second_check_failure_keeps_authorities_unchanged(self):
        for relative, payload in [
            ("gc_state.json", b'{"current_generation":7}'),
            ("request_nonce_cache.json", b'{"entries":{}}'),
            ("branches/main.json", b'{"head":"old"}'),
        ]:
            path = self.project / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
        marker = self.project / "first-check-ran"
        self.write_policy(
            [
                {"id": "first", "command": f"touch {shlex.quote(str(marker))}"},
                {
                    "id": "second",
                    "command": "printf second-failed >&2; exit 7",
                },
            ]
        )
        before = self.authority_snapshot()
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=128
        )
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "second: second-failed"
            ):
                owner.run()
            self.assertIsNotNone(owner._failure_message)
        finally:
            owner.close()
        self.assertTrue(marker.exists())
        self.assertEqual(self.authority_snapshot(), before)
        self.assert_closed(ledger)

    def test_deadline_kills_and_reaps_process_group(self):
        pid_path = self.project / "verification.pid"
        self.write_policy(
            [
                {
                    "id": "sleep",
                    "timeout_seconds": 30,
                    "command": (
                        f"echo $$ > {shlex.quote(str(pid_path))}; sleep 30"
                    ),
                }
            ]
        )
        ledger = self.ledger()
        deadline = self.namespace["HTTPRequestDeadline"](100, time.monotonic)
        owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            ledger,
            deadline,
            output_limit_bytes=64,
        )
        try:
            with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
                owner.run()
        finally:
            owner.close()
        pid = int(pid_path.read_text(encoding="utf-8"))
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        self.assert_closed(ledger)

    def test_malformed_plans_fail_before_spawn(self):
        cases = [
            (
                "cwd",
                {"id": "bad-cwd", "cwd": "..", "command": "exit 0"},
                "cwd escapes",
            ),
            (
                "env",
                {"id": "bad-env", "env": ["bad"], "command": "exit 0"},
                "env must be an object",
            ),
            (
                "timeout",
                {
                    "id": "bad-timeout",
                    "timeout_seconds": "bad",
                    "command": "exit 0",
                },
                "timeout_seconds must be a number",
            ),
            (
                "nonfinite-timeout",
                {
                    "id": "nan-timeout",
                    "timeout_seconds": float("nan"),
                    "command": "exit 0",
                },
                "timeout_seconds must be positive",
            ),
        ]
        spawn = self.Owner._spawn_process
        spawn_calls = 0

        def observe_spawn(*args):
            nonlocal spawn_calls
            spawn_calls += 1
            return spawn(*args)

        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            for label, check, message in cases:
                with self.subTest(label=label):
                    self.write_policy([check])
                    ledger = self.ledger()
                    with self.assertRaisesRegex(
                        self.namespace["CFError"], message
                    ):
                        self.prepare(
                            self.project, "main", self.commit_id, ledger
                        )
                    self.assert_closed(ledger)
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
        self.assertEqual(spawn_calls, 0)

    def test_cwd_swap_uses_pinned_directory_capability(self):
        work = self.project / "work"
        work.mkdir()
        outside = self.tmp / "outside"
        outside.mkdir()
        self.write_policy(
            [
                {
                    "id": "cwd-swap",
                    "cwd": "work",
                    "command": "touch capability-marker",
                }
            ]
        )
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=64
        )
        pinned = self.project / "work-before-swap"
        work.rename(pinned)
        work.symlink_to(outside, target_is_directory=True)
        try:
            owner.run()
        finally:
            owner.close()
        self.assertTrue((pinned / "capability-marker").is_file())
        self.assertFalse((outside / "capability-marker").exists())
        self.assert_closed(ledger)

    def test_unreaped_process_capability_survives_cleanup_retry(self):
        self.write_policy([{"id": "retry", "command": "sleep 30"}])
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=64
        )
        plan = owner._plans[0]
        process = self.Owner._spawn_process(
            plan[1], plan[2], owner._project_fd, plan[4]
        )
        owner._process = process
        kill_process = self.Owner._kill_process
        wait_process = self.Owner._wait_process
        killpg = os.killpg
        process_kill = process.kill
        process_wait = process.wait

        def fail_cleanup(*_args, **_kwargs):
            raise OSError("injected terminal cleanup fault")

        self.Owner._kill_process = staticmethod(fail_cleanup)
        self.Owner._wait_process = staticmethod(fail_cleanup)
        os.killpg = fail_cleanup
        process.kill = fail_cleanup
        process.wait = fail_cleanup
        try:
            with self.assertRaises(OSError) as caught:
                owner.close()
            self.assertRegex(str(caught.exception), "terminal cleanup fault")
            self.assertIs(owner._process, process)
            self.assertIsNone(process.poll())
            self.assertFalse(owner._closed)
            self.assertGreater(
                ledger.current.get("retained_bytes", 0), 0
            )
        finally:
            self.Owner._kill_process = staticmethod(kill_process)
            self.Owner._wait_process = staticmethod(wait_process)
            os.killpg = killpg
            process.kill = process_kill
            process.wait = process_wait
        owner.close()
        self.assertIsNone(owner._process)
        self.assertIsNotNone(process.poll())
        self.assert_closed(ledger)

    def test_spawn_read_kill_faults_reap_and_cleanup(self):
        self.write_policy([{"id": "fault", "command": "sleep 30"}])
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=64
        )
        spawn = self.Owner._spawn_process

        def fail_spawn(*_args):
            raise OSError("injected verification spawn fault")

        self.Owner._spawn_process = staticmethod(fail_spawn)
        try:
            with self.assertRaisesRegex(OSError, "spawn fault"):
                owner.run()
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            owner.close()
        self.assert_closed(ledger)

        self.write_policy(
            [
                {
                    "id": "read-fault",
                    "command": "printf x; sleep 30",
                }
            ]
        )
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=64
        )
        read_pipe = self.Owner._read_pipe
        spawned = []

        def observe_process(*args):
            process = spawn(*args)
            spawned.append(process)
            return process

        def fail_read(*_args):
            raise OSError("injected verification read fault")

        self.Owner._spawn_process = staticmethod(observe_process)
        self.Owner._read_pipe = staticmethod(fail_read)
        try:
            with self.assertRaisesRegex(OSError, "read fault"):
                owner.run()
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            self.Owner._read_pipe = staticmethod(read_pipe)
            owner.close()
        self.assertEqual(len(spawned), 1)
        self.assertIsNotNone(spawned[0].poll())
        self.assert_closed(ledger)

        self.write_policy(
            [
                {
                    "id": "kill-fault",
                    "timeout_seconds": 0.05,
                    "command": "sleep 30",
                }
            ]
        )
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=64
        )
        kill_process = self.Owner._kill_process
        spawned.clear()

        def fail_kill(_process):
            raise OSError("injected verification kill fault")

        self.Owner._spawn_process = staticmethod(observe_process)
        self.Owner._kill_process = staticmethod(fail_kill)
        try:
            with self.assertRaises(self.namespace["CFError"]) as caught:
                owner.run()
            self.assertRegex(str(caught.exception), "timed out")
            self.assertIsInstance(caught.exception.__cause__, OSError)
            self.assertRegex(str(caught.exception.__cause__), "kill fault")
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            self.Owner._kill_process = staticmethod(kill_process)
            owner.close()
        self.assertEqual(len(spawned), 1)
        self.assertIsNotNone(spawned[0].poll())
        self.assert_closed(ledger)

    def test_capability_budget_precedes_open_and_fd_cleanup_is_retryable(self):
        self.write_policy([])
        capability_retained, capability_temporary = (
            self.Owner._capability_upper(self.project)
        )
        rejected = self.ledger(
            {
                "retained_bytes": (
                    "concurrent",
                    capability_retained - 1,
                )
            }
        )
        globals_ = self.Owner._prepare.__globals__
        open_directory = globals_["_open_directory_nofollow"]
        open_observations = []
        set_factory = set
        set_calls = []

        def observe_set(*args, **kwargs):
            set_calls.append((args, kwargs))
            return set_factory(*args, **kwargs)

        def observe_open(path):
            open_observations.append(
                (
                    path,
                    dict(active_ledger.current),
                    active_ledger.has_open_permits(),
                )
            )
            return open_directory(path)

        active_ledger = rejected
        globals_["_open_directory_nofollow"] = observe_open
        globals_["set"] = observe_set
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    rejected,
                    output_limit_bytes=0,
                )
        finally:
            globals_["_open_directory_nofollow"] = open_directory
            del globals_["set"]
        self.assertEqual(open_observations, [])
        self.assertEqual(set_calls, [])
        helper_opcodes = {
            instruction.opname
            for instruction in dis.get_instructions(
                self.Owner._detach_exception_tracebacks
            )
        }
        self.assertTrue(
            helper_opcodes.isdisjoint(
                {
                    "BUILD_LIST",
                    "BUILD_MAP",
                    "BUILD_SET",
                    "BUILD_TUPLE",
                    "LIST_APPEND",
                    "MAP_ADD",
                    "SET_ADD",
                }
            )
        )
        self.assert_closed(rejected)

        active_ledger = self.ledger()
        globals_["_open_directory_nofollow"] = observe_open
        try:
            owner = self.prepare(
                self.project,
                "main",
                self.commit_id,
                active_ledger,
                output_limit_bytes=0,
            )
        finally:
            globals_["_open_directory_nofollow"] = open_directory
        self.assertTrue(open_observations)
        self.assertTrue(
            all(
                current.get("retained_bytes", 0) >= capability_retained
                and current.get("temporary_bytes", 0)
                >= capability_temporary
                and not open_permits
                for _path, current, open_permits in open_observations
            )
        )

        project_fd = owner._project_fd
        close_capability = self.Owner._close_capability
        failed = False

        def fail_project_close_once(file_descriptor):
            nonlocal failed
            if file_descriptor == project_fd and not failed:
                failed = True
                raise OSError("injected project capability close fault")
            return close_capability(file_descriptor)

        self.Owner._close_capability = staticmethod(fail_project_close_once)
        try:
            with self.assertRaisesRegex(OSError, "capability close fault"):
                owner.close()
        finally:
            self.Owner._close_capability = staticmethod(close_capability)
        self.assertTrue(owner._closed)
        self.assertEqual(owner._project_fd, -1)
        self.assertEqual(
            active_ledger.current.get("retained_bytes", 0), 0
        )
        os.fstat(project_fd)
        owner.close()
        os.close(project_fd)
        self.assert_closed(active_ledger)

    def test_policy_link_rewrite_and_replacement_fail_before_spawn(self):
        marker = self.project / "policy-effect"
        original_policy = json.dumps(
            {
                "verification": {
                    "required": [
                        {
                            "id": "policy-fence",
                            "command": "printf OLD > policy-effect",
                        }
                    ]
                }
            },
            separators=(",", ":"),
        ).encode("utf-8")
        changed_policy = original_policy.replace(b"OLD", b"NEW")
        self.assertEqual(len(original_policy), len(changed_policy))
        policy_path = self.project / "server_policy.json"
        alias = self.tmp / "server-policy-alias.json"
        spawn = self.Owner._spawn_process
        spawn_calls = 0

        def observe_spawn(*args):
            nonlocal spawn_calls
            spawn_calls += 1
            return spawn(*args)

        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            policy_path.write_bytes(original_policy)
            os.link(policy_path, alias)
            ledger = self.ledger()
            with self.assertRaisesRegex(
                self.namespace["CFError"], "bounded JSON file is invalid"
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=64,
                )
            alias.unlink()
            self.assert_closed(ledger)

            for mode in ("hardlink", "rewrite", "replacement"):
                with self.subTest(mode=mode):
                    policy_path.write_bytes(original_policy)
                    ledger = self.ledger()
                    owner = self.prepare(
                        self.project,
                        "main",
                        self.commit_id,
                        ledger,
                        output_limit_bytes=64,
                    )
                    if mode == "hardlink":
                        os.link(policy_path, alias)
                    elif mode == "rewrite":
                        policy_path.write_bytes(changed_policy)
                    else:
                        replacement = self.project / "replacement-policy.json"
                        replacement.write_bytes(changed_policy)
                        os.replace(replacement, policy_path)
                    try:
                        with self.assertRaisesRegex(
                            self.namespace["CFError"],
                            "policy changed before execution",
                        ):
                            owner.run()
                    finally:
                        owner.close()
                        if alias.exists():
                            alias.unlink()
                    self.assertFalse(marker.exists())
                    self.assert_closed(ledger)
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            if alias.exists():
                alias.unlink()
        self.assertEqual(spawn_calls, 0)

    def test_project_replacement_never_mixes_policy_and_cwd(self):
        original_project = self.tmp / "original-project"
        globals_ = self.Owner._prepare.__globals__
        open_directory = globals_["_open_directory_nofollow"]
        swapped = False

        def swap_after_project_open(path):
            nonlocal swapped
            file_descriptor = open_directory(path)
            if Path(path) == self.project and not swapped:
                swapped = True
                self.project.rename(original_project)
                self.project.mkdir()
                self.write_policy(
                    [
                        {
                            "id": "replacement-policy",
                            "command": "touch replacement-policy-effect",
                        }
                    ]
                )
            return file_descriptor

        ledger = self.ledger()
        globals_["_open_directory_nofollow"] = swap_after_project_open
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "project changed before execution",
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=64,
                )
        finally:
            globals_["_open_directory_nofollow"] = open_directory
        self.assertTrue(swapped)
        self.assertFalse(
            (original_project / "replacement-policy-effect").exists()
        )
        self.assertFalse((self.project / "replacement-policy-effect").exists())
        self.assert_closed(ledger)

        late_project = self.tmp / "late-project"
        late_project.mkdir()
        late_original = self.tmp / "late-original-project"
        late_marker = late_original / "late-project-effect"
        (late_project / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "late-project-swap",
                                "command": "touch late-project-effect",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        ledger = self.ledger()
        owner = self.prepare(
            late_project,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=64,
        )
        late_project.rename(late_original)
        late_project.mkdir()
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "project changed before execution",
            ):
                owner.run()
        finally:
            owner.close()
        self.assertFalse(late_marker.exists())
        self.assert_closed(ledger)

    def test_spawn_race_exposes_only_pinned_project_capability(self):
        original_project = self.tmp / "spawn-original-project"
        source = self.project / "authority-value"
        source.write_text("ORIGINAL", encoding="utf-8")
        self.write_policy(
            [
                {
                    "id": "spawn-project-swap",
                    "command": (
                        'cat "$CODEFIRE_REMOTE_PROJECT/authority-value" '
                        "> observed-project"
                    ),
                }
            ]
        )
        ledger = self.ledger()
        owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=64,
        )
        spawn = self.Owner._spawn_process
        swapped = False

        def swap_at_spawn(command, cwd_fd, project_fd, env):
            nonlocal swapped
            self.assertEqual(
                env["CODEFIRE_REMOTE_PROJECT"],
                f"/proc/self/fd/{project_fd}",
            )
            self.project.rename(original_project)
            self.project.mkdir()
            (self.project / "authority-value").write_text(
                "REPLACEMENT", encoding="utf-8"
            )
            self.write_policy([])
            swapped = True
            return spawn(command, cwd_fd, project_fd, env)

        self.Owner._spawn_process = staticmethod(swap_at_spawn)
        try:
            owner.run()
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            owner.close()
        self.assertTrue(swapped)
        self.assertEqual(
            (original_project / "observed-project").read_text(
                encoding="utf-8"
            ),
            "ORIGINAL",
        )
        self.assertFalse((self.project / "observed-project").exists())
        self.assert_closed(ledger)

    def test_missing_project_remains_a_no_policy_noop(self):
        missing = self.tmp / "missing-project"
        rejected = self.ledger()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "project is unavailable"
        ):
            self.prepare(
                missing,
                "main",
                self.commit_id,
                rejected,
                output_limit_bytes=64,
            )
        self.assert_closed(rejected)

        ledger = self.ledger()
        owner = self.prepare(
            missing,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=64,
            allow_missing_project=True,
        )
        self.assertEqual(owner._project_fd, -1)
        self.assertEqual(owner._plans, [])
        try:
            owner.run()
        finally:
            owner.close()
        self.assertFalse(missing.exists())
        self.assert_closed(ledger)

    def test_missing_project_mode_rejects_appeared_authority_without_spawn(self):
        raced = self.tmp / "raced-project"
        marker = raced / "raced-policy-effect"
        ledger = self.ledger()
        globals_ = self.Owner._prepare.__globals__
        open_directory = globals_["_open_directory_nofollow"]
        spawn = self.Owner._spawn_process
        spawn_calls = 0
        raced_once = False

        def appear_at_open(path):
            nonlocal raced_once
            if path == raced and not raced_once:
                raced_once = True
                raced.mkdir()
                (raced / "server_policy.json").write_text(
                    json.dumps(
                        {
                            "verification": {
                                "required": [
                                    {
                                        "id": "appeared-policy",
                                        "command": "touch raced-policy-effect",
                                    }
                                ]
                            }
                        }
                    ),
                    encoding="utf-8",
                )
            return open_directory(path)

        def observe_spawn(*args):
            nonlocal spawn_calls
            spawn_calls += 1
            return spawn(*args)

        globals_["_open_directory_nofollow"] = appear_at_open
        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "project changed before execution",
            ):
                self.prepare(
                    raced,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=64,
                    allow_missing_project=True,
                )
        finally:
            globals_["_open_directory_nofollow"] = open_directory
            self.Owner._spawn_process = staticmethod(spawn)
        self.assertTrue(raced_once)
        self.assertEqual(spawn_calls, 0)
        self.assertFalse(marker.exists())
        self.assert_closed(ledger)

        later = self.tmp / "later-project"
        later_ledger = self.ledger()
        owner = self.prepare(
            later,
            "main",
            self.commit_id,
            later_ledger,
            output_limit_bytes=64,
            allow_missing_project=True,
        )
        later.mkdir()
        (later / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "command": "touch later-policy-effect",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "project changed before execution",
            ):
                owner.run()
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            owner.close()
        self.assertEqual(spawn_calls, 0)
        self.assertFalse((later / "later-policy-effect").exists())
        self.assert_closed(later_ledger)

    def test_constructor_detaches_payload_traceback_before_release(self):
        (self.project / "server_policy.json").write_text(
            json.dumps({"verification": []}),
            encoding="utf-8",
        )
        ledger = self.ledger()
        loads = json.loads
        release = ledger.release
        tracked = {}
        released_without_payload = []

        class TrackedPolicy(dict):
            pass

        def tracked_loads(document):
            policy = TrackedPolicy(loads(document))
            tracked["policy"] = weakref.ref(policy)
            return policy

        def observe_release(dimension, amount):
            if dimension == "retained_bytes":
                gc.collect()
                released_without_payload.append(
                    tracked["policy"]() is None
                )
            return release(dimension, amount)

        json.loads = tracked_loads
        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "verification policy must be an object",
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            json.loads = loads
            ledger.release = release
        self.assertTrue(released_without_payload)
        self.assertTrue(all(released_without_payload))
        self.assertIsNone(tracked["policy"]())
        self.assert_closed(ledger)

    def test_exception_detachment_clears_long_cyclic_chain(self):
        exceptions = []
        for index in range(130):
            try:
                raise self.namespace["CFError"](
                    f"injected exception chain {index}"
                )
            except self.namespace["CFError"] as exc:
                exceptions.append(exc)
        for index in range(len(exceptions) - 1):
            if index % 2:
                exceptions[index].__cause__ = exceptions[index + 1]
            else:
                exceptions[index].__context__ = exceptions[index + 1]
        exceptions[-1].__context__ = exceptions[32]

        self.Owner._detach_exception_tracebacks(exceptions[0])

        self.assertTrue(
            all(
                exc.__traceback__ is None
                and exc.__cause__ is None
                and exc.__context__ is None
                for exc in exceptions
            )
        )

    def test_run_wrapper_detaches_helper_traceback_before_release(self):
        self.write_policy([{"id": "tracked-run", "command": "exit 7"}])
        ledger = self.ledger()
        owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=8,
        )
        plan = owner._plans[0]

        class TrackedCommand(str):
            pass

        tracked_command = TrackedCommand(plan[1])
        command_ref = weakref.ref(tracked_command)
        owner._plans[0] = (
            plan[0],
            tracked_command,
            plan[2],
            plan[3],
            plan[4],
        )
        plan = None
        tracked_command = None
        globals_ = self.namespace["run_server_verification"].__globals__
        prepare = globals_["prepare_server_verification"]
        release = ledger.release
        observations = []

        def prepared_owner(*args, **kwargs):
            return owner

        def observe_release(dimension, amount):
            gc.collect()
            observations.append(command_ref() is None)
            return release(dimension, amount)

        globals_["prepare_server_verification"] = prepared_owner
        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"],
                "server verification failed: tracked-run",
            ):
                self.namespace["run_server_verification"](
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=8,
                )
        finally:
            globals_["prepare_server_verification"] = prepare
            ledger.release = release
        self.assertTrue(observations)
        self.assertTrue(all(observations))
        self.assertIsNone(command_ref())
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)

    def test_constructor_permits_survive_consume_and_cancel_faults(self):
        self.write_policy([])
        cases = [
            ("capability", self.project, False, 1, "_capability_permit"),
            ("policy-read", self.project, False, 2, "_policy_permit"),
            ("config", self.project, False, 4, "_config_permit"),
            (
                "empty-policy",
                self.tmp / "permit-missing-project",
                True,
                2,
                "_policy_permit",
            ),
        ]
        for name, project, allow_missing, target_call, attribute in cases:
            with self.subTest(name=name):
                ledger = self.ledger()
                consume_reserved = ledger.consume_reserved
                cancel_permit = ledger.cancel_permit
                consume_calls = 0
                cancel_failures = 0
                target_permit = None

                def fail_consume(permit, **kwargs):
                    nonlocal consume_calls, target_permit
                    consume_calls += 1
                    if consume_calls == target_call:
                        target_permit = permit
                        raise self.namespace["CFError"](
                            "injected setup consume fault"
                        )
                    return consume_reserved(permit, **kwargs)

                def fail_cancel(permit):
                    nonlocal cancel_failures
                    if permit is target_permit and cancel_failures < 2:
                        cancel_failures += 1
                        raise self.namespace["CFError"](
                            "injected setup cancel fault"
                        )
                    return cancel_permit(permit)

                ledger.consume_reserved = fail_consume
                ledger.cancel_permit = fail_cancel
                owner = object.__new__(self.Owner)
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "setup cancel fault"
                ):
                    self.Owner.__init__(
                        owner,
                        project,
                        "main",
                        self.commit_id,
                        ledger,
                        output_limit_bytes=0,
                        allow_missing_project=allow_missing,
                    )
                self.assertEqual(cancel_failures, 2)
                self.assertIs(getattr(owner, attribute), target_permit)
                self.assertTrue(ledger.permit_is_open(target_permit))
                gc.collect()
                self.assertIs(getattr(owner, attribute), target_permit)
                ledger.consume_reserved = consume_reserved
                ledger.cancel_permit = cancel_permit
                owner.close()
                self.assertTrue(owner._closed)
                self.assert_closed(ledger)

        ledger = self.ledger()
        consume_reserved = ledger.consume_reserved
        cancel_permit = ledger.cancel_permit
        acquire_owner = self.Owner._acquire_capability_owner
        cancel_failures = 0
        owner_refs = []

        def observe_owner(owner, project_root):
            owner_refs.append(weakref.ref(owner))
            return acquire_owner(owner, project_root)

        def fail_first_consume(permit, **kwargs):
            raise self.namespace["CFError"]("injected setup consume fault")

        def fail_cancel_twice(permit):
            nonlocal cancel_failures
            if cancel_failures < 2:
                cancel_failures += 1
                raise self.namespace["CFError"](
                    "injected setup cancel fault"
                )
            return cancel_permit(permit)

        ledger.consume_reserved = fail_first_consume
        ledger.cancel_permit = fail_cancel_twice
        self.Owner._acquire_capability_owner = observe_owner
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "setup cancel fault"
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            self.Owner._acquire_capability_owner = acquire_owner
            ledger.consume_reserved = consume_reserved
            ledger.cancel_permit = cancel_permit
        self.assertEqual(cancel_failures, 2)
        gc.collect()
        self.assertEqual(len(owner_refs), 1)
        self.assertIsNone(owner_refs[0]())
        self.assert_closed(ledger)

    def test_post_consume_base_exception_reconciles_all_acquisitions(self):
        self.write_policy([])
        constructor_cases = [
            ("capability", self.project, False, 1),
            ("policy-read", self.project, False, 2),
            ("policy-read-charge", self.project, False, 3),
            ("config", self.project, False, 4),
            (
                "empty-policy",
                self.tmp / "post-consume-missing-project",
                True,
                2,
            ),
            (
                "empty-config",
                self.tmp / "post-consume-missing-config",
                True,
                3,
            ),
        ]
        for name, project, allow_missing, target_call in constructor_cases:
            with self.subTest(name=name):
                ledger = self.ledger()
                consume_reserved = ledger.consume_reserved
                consume_calls = 0

                def consume_then_interrupt(permit, **kwargs):
                    nonlocal consume_calls
                    consume_calls += 1
                    result = consume_reserved(permit, **kwargs)
                    if consume_calls == target_call:
                        raise KeyboardInterrupt(
                            "injected post-consume constructor fault"
                        )
                    return result

                ledger.consume_reserved = consume_then_interrupt
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "post-consume constructor fault"
                    ):
                        self.prepare(
                            project,
                            "main",
                            self.commit_id,
                            ledger,
                            output_limit_bytes=0,
                            allow_missing_project=allow_missing,
                        )
                finally:
                    ledger.consume_reserved = consume_reserved
                self.assert_closed(ledger)

        self.write_policy(
            [{"id": "post-consume-run", "command": "printf ok"}]
        )
        for attribute in (
            "_policy_fence_storage_permit",
            "_output_storage_permit",
        ):
            with self.subTest(attribute=attribute):
                ledger = self.ledger()
                owner = self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=8,
                )
                target_permit = getattr(owner, attribute)
                consume_reserved = ledger.consume_reserved

                def consume_then_interrupt(permit, **kwargs):
                    result = consume_reserved(permit, **kwargs)
                    if permit is target_permit:
                        raise KeyboardInterrupt(
                            "injected post-consume run fault"
                        )
                    return result

                ledger.consume_reserved = consume_then_interrupt
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "post-consume run fault"
                    ):
                        owner.run()
                finally:
                    ledger.consume_reserved = consume_reserved
                owner.close()
                self.assertIsNone(owner._pending_acquisition)
                self.assert_closed(ledger)

        charge_cases = [
            ("policy-fence-charge", "printf ok", "_policy_fence_permit"),
            ("output-chunk-charge", "printf x", "_output_permit"),
            ("output-failure-charge", "exit 7", "_output_permit"),
        ]
        for name, command, attribute in charge_cases:
            with self.subTest(name=name):
                self.write_policy([{"id": name, "command": command}])
                ledger = self.ledger()
                owner = self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=8,
                )
                target_permit = getattr(owner, attribute)
                consume_reserved = ledger.consume_reserved
                interrupted = False

                def consume_then_interrupt(permit, **kwargs):
                    nonlocal interrupted
                    result = consume_reserved(permit, **kwargs)
                    if permit is target_permit and not interrupted:
                        interrupted = True
                        raise KeyboardInterrupt(
                            "injected post-consume charge fault"
                        )
                    return result

                ledger.consume_reserved = consume_then_interrupt
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "post-consume charge fault"
                    ):
                        owner.run()
                finally:
                    ledger.consume_reserved = consume_reserved
                self.assertTrue(interrupted)
                owner.close()
                self.assert_closed(ledger)

    def test_post_cancel_base_exception_does_not_adopt_acquisition(self):
        ledger = self.ledger()
        consume_reserved = ledger.consume_reserved
        cancel_permit = ledger.cancel_permit

        def fail_consume(permit, **kwargs):
            raise self.namespace["CFError"]("injected pre-consume fault")

        def cancel_then_interrupt(permit):
            cancel_permit(permit)
            raise KeyboardInterrupt("injected post-cancel fault")

        ledger.consume_reserved = fail_consume
        ledger.cancel_permit = cancel_then_interrupt
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "post-cancel fault"
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            ledger.consume_reserved = consume_reserved
            ledger.cancel_permit = cancel_permit
        self.assert_closed(ledger)

    def test_all_reservations_publish_owner_before_register_fault(self):
        self.write_policy(
            [{"id": "reserve-into", "command": "printf ok"}]
        )
        cases = [
            *(('existing', self.project, False, target) for target in range(1, 9)),
            (
                "missing-empty-policy",
                self.tmp / "reserve-into-missing",
                True,
                2,
            ),
        ]
        for name, project, allow_missing, target_call in cases:
            with self.subTest(name=name, target_call=target_call):
                ledger = self.ledger()
                register = ledger._register_reservation
                register_calls = 0

                def register_then_interrupt(permit, reservation):
                    nonlocal register_calls
                    register_calls += 1
                    result = register(permit, reservation)
                    if register_calls == target_call:
                        raise KeyboardInterrupt(
                            "injected post-register reservation fault"
                        )
                    return result

                ledger._register_reservation = register_then_interrupt
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "post-register reservation fault"
                    ):
                        self.prepare(
                            project,
                            "main",
                            self.commit_id,
                            ledger,
                            output_limit_bytes=0,
                            allow_missing_project=allow_missing,
                        )
                finally:
                    ledger._register_reservation = register
                self.assertEqual(register_calls, target_call)
                self.assert_closed(ledger)

    def test_post_release_base_exception_is_exactly_retryable(self):
        self.write_policy([])

        ledger = self.ledger()
        release = ledger.release
        interrupted = False

        def interrupt_policy_release(dimension, amount):
            nonlocal interrupted
            result = release(dimension, amount)
            if dimension == "temporary_bytes" and not interrupted:
                interrupted = True
                raise KeyboardInterrupt("injected post-policy-release fault")
            return result

        ledger.release = interrupt_policy_release
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "post-policy-release fault"
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            ledger.release = release
        self.assertTrue(interrupted)
        self.assert_closed(ledger)

        self.write_policy([{"command": {"invalid": True}}])
        ledger = self.ledger()
        release = ledger.release
        interrupted = False

        def interrupt_config_release(dimension, amount):
            nonlocal interrupted
            result = release(dimension, amount)
            if dimension == "retained_bytes" and not interrupted:
                interrupted = True
                raise KeyboardInterrupt("injected post-config-release fault")
            return result

        ledger.release = interrupt_config_release
        try:
            with self.assertRaises(self.namespace["CFError"]) as caught:
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            ledger.release = release
        self.assertTrue(interrupted)
        self.assertRegex(str(caught.exception), "command must be a string")
        self.assertIsInstance(caught.exception.__cause__, KeyboardInterrupt)
        self.assertRegex(
            str(caught.exception.__cause__), "post-config-release fault"
        )
        self.assert_closed(ledger)

        self.write_policy([])
        for target_dimension in ("temporary_bytes", "retained_bytes"):
            with self.subTest(target_dimension=target_dimension):
                ledger = self.ledger()
                owner = self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
                release = ledger.release
                interrupted = False

                def interrupt_close_release(dimension, amount):
                    nonlocal interrupted
                    result = release(dimension, amount)
                    if dimension == target_dimension and not interrupted:
                        interrupted = True
                        raise KeyboardInterrupt(
                            "injected post-close-release fault"
                        )
                    return result

                ledger.release = interrupt_close_release
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "post-close-release fault"
                    ):
                        owner.close()
                finally:
                    ledger.release = release
                self.assertTrue(interrupted)
                owner.close()
                self.assertIsNone(owner._pending_release)
                self.assert_closed(ledger)

    def test_close_drops_payload_before_retryable_release(self):
        self.write_policy([{"id": "close", "command": "printf ok"}])
        ledger = self.ledger()
        owner = self.prepare(
            self.project, "main", self.commit_id, ledger, output_limit_bytes=8
        )
        owner.run()
        release = ledger.release
        observations = []
        calls = 0

        def fail_once(dimension, amount):
            nonlocal calls
            calls += 1
            observations.append(
                (
                    owner._plans,
                    owner._policy,
                    owner._failure_message,
                    dimension,
                    amount,
                )
            )
            if calls == 1:
                raise self.namespace["CFError"](
                    "injected verification release fault"
                )
            return release(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(
            self.namespace["CFError"], "verification release fault"
        ):
            owner.close()
        self.assertFalse(owner._closed)
        self.assertTrue(
            all(
                plans is None and policy is None and failure is None
                for plans, policy, failure, _dimension, _amount in observations
            )
        )
        owner.close()
        gc.collect()
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)

    def test_capability_close_exception_never_retries_reused_number(self):
        self.write_policy([{"id": "close-fd", "command": "printf ok"}])
        self.assertIs(
            self.Owner._close_capability,
            self.namespace["_CLOSE_FILE_DESCRIPTOR"],
        )
        cases = ("policy", "project", "plan")
        class TrackedPlanText(str):
            pass

        for kind in cases:
            with self.subTest(kind=kind):
                ledger = self.ledger()
                owner = self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
                if kind == "policy":
                    target_fd = owner._policy_fd
                elif kind == "project":
                    target_fd = owner._project_fd
                    reopen_path = self.project
                    reopen_flags = os.O_RDONLY | os.O_DIRECTORY
                else:
                    plan = owner._plans[-1]
                    tracked_text = TrackedPlanText(plan[1])
                    tracked_plan_ref = weakref.ref(tracked_text)
                    owner._plans[-1] = (
                        plan[0],
                        tracked_text,
                        plan[2],
                        plan[3],
                        plan[4],
                    )
                    plan = None
                    tracked_text = None
                    target_fd = owner._plans[-1][2]
                    reopen_path = self.project
                    reopen_flags = os.O_RDONLY | os.O_DIRECTORY
                if kind == "policy":
                    reopen_path = self.project / "server_policy.json"
                    reopen_flags = os.O_RDONLY
                target_metadata = os.fstat(target_fd)
                close_capability = self.Owner._close_capability
                interrupted = False
                reopened_fd = -1

                def close_then_interrupt(file_descriptor):
                    nonlocal interrupted, reopened_fd
                    result = close_capability(file_descriptor)
                    if file_descriptor == target_fd and not interrupted:
                        interrupted = True
                        reopened_fd = os.open(reopen_path, reopen_flags)
                        if reopened_fd != target_fd:
                            os.dup2(reopened_fd, target_fd)
                            os.close(reopened_fd)
                            reopened_fd = target_fd
                        raise KeyboardInterrupt(
                            "injected post-capability-close fault"
                        )
                    return result

                self.Owner._close_capability = staticmethod(close_then_interrupt)
                try:
                    try:
                        owner.close()
                    except KeyboardInterrupt as exc:
                        caught_error = exc
                    else:
                        self.fail("post-capability-close fault was not raised")
                finally:
                    self.Owner._close_capability = staticmethod(close_capability)
                try:
                    self.assertTrue(interrupted)
                    self.assertIn(
                        "post-capability-close fault", str(caught_error)
                    )
                    self.assertEqual(reopened_fd, target_fd)
                    reopened_metadata = os.fstat(reopened_fd)
                    self.assertEqual(
                        (reopened_metadata.st_dev, reopened_metadata.st_ino),
                        (target_metadata.st_dev, target_metadata.st_ino),
                    )
                    if kind == "policy":
                        self.assertEqual(owner._policy_fd, -1)
                    elif kind == "project":
                        self.assertEqual(owner._project_fd, -1)
                    else:
                        self.assertIsNone(owner._plans)
                    retry_closes = []

                    def observe_retry(file_descriptor):
                        retry_closes.append(file_descriptor)
                        return close_capability(file_descriptor)

                    self.Owner._close_capability = staticmethod(observe_retry)
                    try:
                        owner.close()
                    finally:
                        self.Owner._close_capability = staticmethod(
                            close_capability
                        )
                    self.assertNotIn(target_fd, retry_closes)
                    os.fstat(reopened_fd)
                    self.assert_closed(ledger)
                    if kind == "plan":
                        gc.collect()
                        self.assertIsNone(tracked_plan_ref())
                finally:
                    if reopened_fd >= 0:
                        os.close(reopened_fd)

        ledger = self.ledger()
        owner = self.prepare(
            self.project,
            "main",
            self.commit_id,
            ledger,
            output_limit_bytes=0,
        )
        ambient_closed_fd = owner._project_fd
        os.close(ambient_closed_fd)
        with self.assertRaises(OSError) as caught:
            owner.close()
        self.assertEqual(caught.exception.errno, 9)
        self.assertEqual(owner._project_fd, -1)
        self.assertEqual(ledger.current.get("retained_bytes", 0), 0)
        owner.close()
        self.assert_closed(ledger)

    def test_config_failure_drops_traceback_payload_before_release(self):
        (self.project / "server_policy.json").write_text("{}", encoding="utf-8")
        ledger = self.ledger()
        loads = json.loads
        release = ledger.release
        bounded_descriptor = self.Owner.__dict__["_bounded_plan_text"]
        bounded = self.Owner._bounded_plan_text
        refs = []
        release_observations = []

        class TrackedPolicy(dict):
            pass

        class TrackedText(str):
            pass

        def tracked_loads(_document):
            policy = TrackedPolicy(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "tracked-plan",
                                "command": "printf ok",
                                "env": {"TRACKED_KEY": "tracked-value"},
                            },
                            {"id": "invalid-plan", "command": {"bad": True}},
                        ]
                    }
                }
            )
            refs.append(weakref.ref(policy))
            return policy

        def tracked_bounded(cls, value, field, limit=None):
            text = TrackedText(bounded(value, field, limit))
            refs.append(weakref.ref(text))
            return text

        def observe_release(dimension, amount):
            if dimension == "retained_bytes":
                gc.collect()
                release_observations.append(
                    all(ref() is None for ref in refs)
                )
            return release(dimension, amount)

        json.loads = tracked_loads
        self.Owner._bounded_plan_text = classmethod(tracked_bounded)
        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "command must be a string"
            ):
                self.prepare(
                    self.project,
                    "main",
                    self.commit_id,
                    ledger,
                    output_limit_bytes=0,
                )
        finally:
            json.loads = loads
            self.Owner._bounded_plan_text = bounded_descriptor
            ledger.release = release
        self.assertTrue(refs)
        self.assertTrue(release_observations)
        self.assertTrue(all(release_observations))
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
