import gc
import json
import os
import runpy
import shutil
import sys
import tempfile
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PhaseDeadline:
    def __init__(self, phase):
        self.phase = phase

    def check(self, phase):
        if phase == self.phase:
            raise RuntimeError(f"deadline at {phase}")


class PythonSignatureAuthorityTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.Authority = self.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
        ]
        self.SubjectChanged = self.namespace["SignaturePolicySubjectChanged"]
        self.project = self.tmp / "storage/org/app"
        self.project.mkdir(parents=True)
        self.policy_path = self.project / "server_policy.json"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, overrides=None):
        return self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(overrides),
            "signature authority",
            command_kind="http_request",
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def write_policy(self, *, required=False, history=False, keys=None):
        raw = json.dumps(
            {
                "commit_signatures": {
                    "required": required,
                    "require_history": history,
                    "keys": {} if keys is None else keys,
                }
            },
            separators=(",", ":"),
        ).encode("utf-8")
        self.policy_path.write_bytes(raw)
        return raw

    def empty_loader(self, calls=None):
        def load(object_id):
            if calls is not None:
                calls.append(object_id)
            raise AssertionError("signature payload loader was called")

        return load

    def prepare_noop(self, authority, ledger):
        owner = authority.prepare_policy(self.empty_loader(), ledger)
        owner.verify("CF-COMMIT-unused")
        return owner

    def test_bound_owner_cannot_outlive_authority_payload(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner = self.prepare_noop(authority, ledger)
        self.assertIs(owner._policy_raw, authority._policy_raw)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "authority is single-use"
        ):
            authority.prepare_policy(self.empty_loader(), ledger)
        authority.revalidate(ledger)
        authority.close(ledger)
        self.assertTrue(owner._closed)
        self.assertIsNone(owner._policy_raw)
        self.assertIsNone(authority._policy_raw)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "policy owner is closed"
        ):
            owner.verify("CF-COMMIT-unused")
        self.assert_closed(ledger)

    def test_snapshot_keys_drive_pure_signature_owner(self):
        key = "signature-secret"
        self.write_policy(required=True, keys={"alice": key})
        commit = self.namespace["sign_commit_payload"](
            {"type": "commit", "version": 1, "parents": []},
            "alice",
            key,
        )
        calls = []

        def loader(object_id):
            calls.append(object_id)
            return commit, self.namespace["canonical_json_size"](commit)

        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner = authority.prepare_policy(loader, ledger)
        owner.verify("CF-COMMIT-head")
        authority.revalidate(ledger)
        authority.close(ledger)
        self.assertEqual(calls, ["CF-COMMIT-head"])
        self.assert_closed(ledger)

    def test_capability_exact_minus_one_precedes_open(self):
        retained, temporary = self.Authority.capability_upper(self.project)
        original_open = os.open
        for dimension, upper in (
            ("retained_bytes", retained),
            ("temporary_bytes", temporary),
        ):
            with self.subTest(dimension=dimension):
                ledger = self.ledger({dimension: ("concurrent", upper - 1)})
                opens = 0

                def observe_open(*args, **kwargs):
                    nonlocal opens
                    opens += 1
                    return original_open(*args, **kwargs)

                os.open = observe_open
                try:
                    with self.assertRaises(
                        self.namespace["ResourceBudgetExceeded"]
                    ):
                        self.Authority(self.project, ledger)
                finally:
                    os.open = original_open
                self.assertEqual(opens, 0)
                self.assert_closed(ledger)

    def test_path_materialization_follows_preflight_reservation(self):
        ledger = self.ledger()
        original_abspath = os.path.abspath
        observations = []

        def observe_abspath(path):
            observations.append(
                ledger.current.get("temporary_bytes", 0)
            )
            return original_abspath(path)

        os.path.abspath = observe_abspath
        try:
            authority = self.Authority(self.project, ledger)
        finally:
            os.path.abspath = original_abspath
        self.assertTrue(observations)
        self.assertTrue(
            all(
                current
                >= self.Authority.PATH_PREFLIGHT_TEMPORARY_BYTES
                for current in observations
            )
        )
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_policy_exact_minus_one_precedes_read(self):
        raw = self.write_policy()
        capability_retained, capability_total_temporary = (
            self.Authority.capability_upper(self.project)
        )
        capability_temporary = (
            capability_total_temporary
            - self.Authority.PATH_PREFLIGHT_TEMPORARY_BYTES
        )
        policy_retained, policy_temporary = self.Authority.policy_upper(len(raw))
        cases = (
            (
                "retained_bytes",
                capability_retained + policy_retained - 1,
            ),
            (
                "temporary_bytes",
                capability_temporary + policy_temporary - 1,
            ),
        )
        original_read = self.Authority._read_exact_policy
        for dimension, limit in cases:
            with self.subTest(dimension=dimension):
                ledger = self.ledger({dimension: ("concurrent", limit)})
                reads = 0

                def observe_read(*args, **kwargs):
                    nonlocal reads
                    reads += 1
                    return original_read(*args, **kwargs)

                self.Authority._read_exact_policy = observe_read
                try:
                    with self.assertRaises(
                        self.namespace["ResourceBudgetExceeded"]
                    ):
                        self.Authority(self.project, ledger)
                finally:
                    self.Authority._read_exact_policy = original_read
                self.assertEqual(reads, 0)
                self.assert_closed(ledger)

    def test_final_fence_keeps_capability_overlap_headroom(self):
        for policy_present in (False, True):
            with self.subTest(policy_present=policy_present):
                if self.policy_path.exists():
                    self.policy_path.unlink()
                raw = self.write_policy() if policy_present else b""
                _, capability_total = self.Authority.capability_upper(
                    self.project
                )
                capability_temporary = (
                    capability_total
                    - self.Authority.PATH_PREFLIGHT_TEMPORARY_BYTES
                )
                policy_temporary = (
                    self.Authority.policy_upper(len(raw))[1]
                    if policy_present
                    else 0
                )
                policy_parse_temporary = self.namespace[
                    "AccountedCommitSignaturePolicy"
                ].policy_temporary_upper(
                    len(raw) if policy_present else len(b"{}")
                )
                ledger = self.ledger(
                    {
                        "temporary_bytes": (
                            "concurrent",
                            max(
                                capability_total,
                                capability_temporary
                                + policy_temporary
                                + policy_parse_temporary,
                            ),
                        )
                    }
                )
                authority = self.Authority(self.project, ledger)
                self.prepare_noop(authority, ledger)
                original_open = os.open
                observations = []

                def observe_open(*args, **kwargs):
                    observations.append(
                        ledger.current.get("temporary_bytes", 0)
                    )
                    return original_open(*args, **kwargs)

                os.open = observe_open
                try:
                    authority.revalidate(ledger)
                finally:
                    os.open = original_open
                self.assertTrue(observations)
                self.assertTrue(
                    all(
                        current >= capability_temporary
                        for current in observations
                    )
                )
                authority.close(ledger)
                self.assert_closed(ledger)

    def test_initial_symlink_and_hardlink_policy_are_rejected(self):
        outside = self.tmp / "outside-policy.json"
        outside.write_text("{}", encoding="utf-8")
        self.policy_path.symlink_to(outside)
        ledger = self.ledger()
        with self.assertRaisesRegex(self.namespace["CFError"], "is unsafe"):
            self.Authority(self.project, ledger)
        self.assert_closed(ledger)
        self.policy_path.unlink()

        os.link(outside, self.policy_path)
        ledger = self.ledger()
        with self.assertRaisesRegex(self.namespace["CFError"], "is unsafe"):
            self.Authority(self.project, ledger)
        self.assert_closed(ledger)

    def test_final_fence_rejects_changes_and_keeps_swap_back_snapshot(self):
        mutations = {}

        def same_size():
            raw = self.policy_path.read_bytes()
            replacement = bytearray(raw)
            replacement[-2] = ord(" ") if replacement[-2] != ord(" ") else ord("\n")
            self.policy_path.write_bytes(replacement)

        def append():
            with self.policy_path.open("ab") as stream:
                stream.write(b" ")

        def replace():
            replacement = self.project / "replacement.json"
            replacement.write_bytes(self.policy_path.read_bytes())
            os.replace(replacement, self.policy_path)

        def hardlink():
            os.link(self.policy_path, self.tmp / "policy-hardlink")

        def swap_back():
            original = self.project / "original-policy.json"
            replacement = self.project / "replacement-policy.json"
            os.replace(self.policy_path, original)
            replacement.write_bytes(original.read_bytes())
            os.replace(replacement, self.policy_path)
            self.policy_path.unlink()
            os.replace(original, self.policy_path)

        mutations.update(
            same_size=same_size,
            append=append,
            replace=replace,
            hardlink=hardlink,
        )
        for name, mutation in mutations.items():
            with self.subTest(mutation=name):
                if self.policy_path.exists():
                    self.policy_path.unlink()
                hardlink_path = self.tmp / "policy-hardlink"
                if hardlink_path.exists():
                    hardlink_path.unlink()
                self.write_policy()
                ledger = self.ledger()
                authority = self.Authority(self.project, ledger)
                self.prepare_noop(authority, ledger)
                mutation()
                with self.assertRaises(self.SubjectChanged):
                    authority.revalidate(ledger)
                authority.close(ledger)
                self.assert_closed(ledger)

        self.policy_path.unlink()
        original_raw = self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner = self.prepare_noop(authority, ledger)
        swap_back()
        matches_snapshot = all(
            (
                authority._fingerprint(os.stat(path)) == expected
                for path, expected in (
                    (self.policy_path, authority._policy_fingerprint),
                    (self.project, authority._project_fingerprint),
                    (authority._prefix_path, authority._prefix_fingerprint),
                )
            )
        )
        if matches_snapshot:
            authority.revalidate(ledger)
        else:
            with self.assertRaises(self.SubjectChanged):
                authority.revalidate(ledger)
        self.assertEqual(owner._policy_raw, original_raw)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_missing_subjects_are_fenced(self):
        missing_project = self.tmp / "missing-root/org/app"
        missing_project.parent.mkdir(parents=True)
        ledger = self.ledger()
        authority = self.Authority(missing_project, ledger)
        calls = []
        owner = authority.prepare_policy(self.empty_loader(calls), ledger)
        owner.verify("CF-COMMIT-unused")
        missing_project.mkdir()
        with self.assertRaises(self.SubjectChanged):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assertEqual(calls, [])
        self.assert_closed(ledger)

        project = self.tmp / "missing-policy"
        project.mkdir()
        ledger = self.ledger()
        authority = self.Authority(project, ledger)
        self.prepare_noop(authority, ledger)
        (project / "server_policy.json").write_text("{}", encoding="utf-8")
        with self.assertRaises(self.SubjectChanged):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_project_replacement_and_swap_back_are_fenced(self):
        for swap_back in (False, True):
            with self.subTest(swap_back=swap_back):
                if self.project.exists():
                    shutil.rmtree(self.project)
                self.project.mkdir(parents=True)
                original_raw = self.write_policy()
                ledger = self.ledger()
                authority = self.Authority(self.project, ledger)
                owner = self.prepare_noop(authority, ledger)
                original = self.tmp / "original-project"
                os.replace(self.project, original)
                self.project.mkdir()
                (self.project / "server_policy.json").write_text(
                    "{}", encoding="utf-8"
                )
                if swap_back:
                    shutil.rmtree(self.project)
                    os.replace(original, self.project)
                if swap_back:
                    matches_snapshot = all(
                        (
                            authority._fingerprint(os.stat(path)) == expected
                            for path, expected in (
                                (
                                    self.project,
                                    authority._project_fingerprint,
                                ),
                                (
                                    authority._prefix_path,
                                    authority._prefix_fingerprint,
                                ),
                            )
                        )
                    )
                    if matches_snapshot:
                        authority.revalidate(ledger)
                    else:
                        with self.assertRaises(self.SubjectChanged):
                            authority.revalidate(ledger)
                    self.assertEqual(owner._policy_raw, original_raw)
                else:
                    with self.assertRaises(self.SubjectChanged):
                        authority.revalidate(ledger)
                authority.close(ledger)
                self.assert_closed(ledger)
                if not swap_back:
                    shutil.rmtree(original)

    def test_project_replacement_during_policy_fence_is_rejected(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        revalidate_project = self.Authority._revalidate_project
        calls = 0

        def replace_after_first_fence(owner):
            nonlocal calls
            revalidate_project(owner)
            calls += 1
            if calls == 1:
                original = self.tmp / "project-during-policy-fence"
                os.replace(self.project, original)
                self.project.mkdir()
                self.write_policy()

        self.Authority._revalidate_project = replace_after_first_fence
        try:
            with self.assertRaises(self.SubjectChanged):
                authority.revalidate(ledger)
        finally:
            self.Authority._revalidate_project = revalidate_project
        self.assertEqual(calls, 1)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_final_deadline_and_constructor_fault_cleanup(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(
            self.project,
            ledger,
            PhaseDeadline("commit signature policy final fence"),
        )
        self.prepare_noop(authority, ledger)
        with self.assertRaisesRegex(RuntimeError, "deadline at"):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)

        read = self.Authority._read_exact_policy

        def fail_read(*_args, **_kwargs):
            raise RuntimeError("injected signature authority read")

        ledger = self.ledger()
        self.Authority._read_exact_policy = fail_read
        try:
            with self.assertRaisesRegex(RuntimeError, "authority read"):
                self.Authority(self.project, ledger)
        finally:
            self.Authority._read_exact_policy = read
        self.assert_closed(ledger)

    def test_intermediate_close_fault_does_not_retry_reused_fd(self):
        self.write_policy()
        ledger = self.ledger()
        close_capability = self.Authority.CLOSE_CAPABILITY
        failed = False
        ambient_fd = -1
        close_attempts = []

        def close_then_reuse(descriptor):
            nonlocal failed, ambient_fd
            close_attempts.append(descriptor)
            close_capability(descriptor)
            if not failed:
                failed = True
                opened = os.open(
                    self.tmp,
                    os.O_RDONLY | getattr(os, "O_DIRECTORY", 0),
                )
                if opened != descriptor:
                    os.dup2(opened, descriptor)
                    close_capability(opened)
                ambient_fd = descriptor
                raise RuntimeError("injected intermediate close ABA")

        self.Authority.CLOSE_CAPABILITY = staticmethod(close_then_reuse)
        try:
            with self.assertRaisesRegex(RuntimeError, "intermediate close ABA"):
                self.Authority(self.project, ledger)
        finally:
            self.Authority.CLOSE_CAPABILITY = staticmethod(close_capability)
        self.assertTrue(failed)
        self.assertGreaterEqual(len(close_attempts), 2)
        self.assertNotIn(ambient_fd, close_attempts[1:])
        os.fstat(ambient_fd)
        gc.collect()
        os.fstat(ambient_fd)
        close_capability(ambient_fd)
        self.assert_closed(ledger)

    def test_final_overlap_close_fault_cleans_remaining_descriptor(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        close_capability = self.Authority.CLOSE_CAPABILITY
        failed = False
        ambient_fd = -1
        close_attempts = []

        def close_then_reuse(descriptor):
            nonlocal failed, ambient_fd
            close_attempts.append(descriptor)
            close_capability(descriptor)
            if not failed:
                failed = True
                opened = os.open(self.policy_path, os.O_RDONLY)
                if opened != descriptor:
                    os.dup2(opened, descriptor)
                    close_capability(opened)
                ambient_fd = descriptor
                raise RuntimeError("injected final overlap close ABA")

        self.Authority.CLOSE_CAPABILITY = staticmethod(close_then_reuse)
        try:
            with self.assertRaisesRegex(RuntimeError, "overlap close ABA"):
                authority.revalidate(ledger)
        finally:
            self.Authority.CLOSE_CAPABILITY = staticmethod(close_capability)
        self.assertTrue(failed)
        self.assertGreaterEqual(len(close_attempts), 2)
        self.assertNotIn(ambient_fd, close_attempts[1:])
        os.fstat(ambient_fd)
        authority.close(ledger)
        os.fstat(ambient_fd)
        close_capability(ambient_fd)
        self.assert_closed(ledger)

    def test_fd_close_aba_never_retries_reused_number(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        close_capability = self.Authority.CLOSE_CAPABILITY
        failed = False
        ambient_fd = -1

        def close_then_reuse(descriptor):
            nonlocal failed, ambient_fd
            close_capability(descriptor)
            if not failed:
                failed = True
                opened = os.open(self.policy_path, os.O_RDONLY)
                if opened != descriptor:
                    os.dup2(opened, descriptor)
                    close_capability(opened)
                ambient_fd = descriptor
                raise RuntimeError("injected post-close ABA")

        self.Authority.CLOSE_CAPABILITY = staticmethod(close_then_reuse)
        try:
            with self.assertRaisesRegex(RuntimeError, "post-close ABA"):
                authority.close(ledger)
        finally:
            self.Authority.CLOSE_CAPABILITY = staticmethod(close_capability)
        self.assertTrue(failed)
        self.assertGreaterEqual(ambient_fd, 0)
        os.fstat(ambient_fd)
        authority.close(ledger)
        os.fstat(ambient_fd)
        close_capability(ambient_fd)
        self.assert_closed(ledger)

    def test_release_fault_retries_after_child_payload_drop(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner = self.prepare_noop(authority, ledger)
        owner.close(ledger)
        release = ledger.release
        failed = False
        observations = []

        def fail_retained_once(dimension, amount):
            nonlocal failed
            observations.append(
                (
                    owner._policy_raw,
                    authority._policy_raw,
                    authority._policy_fd,
                    dimension,
                )
            )
            if dimension == "retained_bytes" and not failed:
                failed = True
                raise RuntimeError("injected signature authority release")
            return release(dimension, amount)

        ledger.release = fail_retained_once
        try:
            with self.assertRaisesRegex(RuntimeError, "authority release"):
                authority.close(ledger)
        finally:
            ledger.release = release
        self.assertTrue(failed)
        self.assertTrue(
            all(
                owner_raw is None and authority_raw is None and policy_fd < 0
                for owner_raw, authority_raw, policy_fd, _ in observations
            )
        )
        authority.close(ledger)
        gc.collect()
        self.assert_closed(ledger)

    def test_final_release_error_traceback_drops_raw_policy(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        pinned_raw = authority._policy_raw
        pinned_digest = authority._policy_digest
        release = ledger.release
        failed = False
        release_raw_values = []

        def fail_temporary_once(dimension, amount):
            nonlocal failed
            if dimension == "temporary_bytes" and not failed:
                frame = sys._getframe()
                while frame is not None:
                    if frame.f_code.co_name == "_revalidate_policy":
                        release_raw_values.append(frame.f_locals.get("raw"))
                        break
                    frame = frame.f_back
                failed = True
                raise RuntimeError("injected final policy release")
            return release(dimension, amount)

        ledger.release = fail_temporary_once
        caught = None
        try:
            authority.revalidate(ledger)
        except RuntimeError as error:
            caught = error
        finally:
            ledger.release = release
        self.assertIsNotNone(caught)
        self.assertTrue(failed)
        self.assertEqual(release_raw_values, [None])
        authority.close(ledger)
        traceback = caught.__traceback__
        policy_frames = []
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "_revalidate_policy":
                policy_frames.append(traceback.tb_frame)
            traceback = traceback.tb_next
        self.assertTrue(policy_frames)
        self.assertTrue(
            all(
                all(
                    value is not pinned_raw and value is not pinned_digest
                    for value in frame.f_locals.values()
                )
                for frame in policy_frames
            )
        )
        self.assert_closed(ledger)

    def test_final_success_drops_second_read_before_temporary_release(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        release = ledger.release
        release_raw_values = []

        def observe_release(dimension, amount):
            if dimension == "temporary_bytes":
                frame = sys._getframe()
                while frame is not None:
                    if frame.f_code.co_name == "_revalidate_policy":
                        release_raw_values.append(frame.f_locals.get("raw"))
                        break
                    frame = frame.f_back
            return release(dimension, amount)

        ledger.release = observe_release
        try:
            authority.revalidate(ledger)
        finally:
            ledger.release = release
        self.assertEqual(release_raw_values, [None])
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_child_close_fault_retains_authority_until_retry(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner = self.prepare_noop(authority, ledger)
        release = ledger.release
        failed = False
        observations = []

        def fail_child_once(dimension, amount):
            nonlocal failed
            if owner._policy_raw is None:
                observations.append(
                    (authority._policy_raw is not None, authority._policy_fd)
                )
                if dimension == "retained_bytes" and not failed:
                    failed = True
                    raise RuntimeError("injected child close release")
            return release(dimension, amount)

        ledger.release = fail_child_once
        try:
            with self.assertRaisesRegex(RuntimeError, "child close release"):
                authority.close(ledger)
        finally:
            ledger.release = release
        self.assertTrue(failed)
        self.assertTrue(owner._policy_raw is None)
        self.assertIsNotNone(authority._policy_raw)
        self.assertGreaterEqual(authority._policy_fd, 0)
        self.assertTrue(
            all(
                raw_retained and policy_fd >= 0
                for raw_retained, policy_fd in observations
            )
        )
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_close_later_fault_cannot_mutate_first_shared_tail(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        self.prepare_noop(authority, ledger)
        close_capability = authority._close_capability
        first = RuntimeError("exact authority first cleanup")
        shared = LookupError("authority shared cleanup")
        tail = OSError("authority protected cleanup tail")
        later = ValueError("authority later cleanup")
        first.__cause__ = shared
        shared.__cause__ = tail
        later.__cause__ = shared
        calls = []

        def fail_two(attribute):
            calls.append(attribute)
            if len(calls) == 1:
                raise first
            if len(calls) == 2:
                raise later
            return close_capability(attribute)

        authority._close_capability = fail_two
        try:
            with self.assertRaises(RuntimeError) as caught:
                authority.close(ledger)
        finally:
            authority._close_capability = close_capability
        self.assertIs(caught.exception, first)
        self.assertIs(first.__cause__, shared)
        self.assertIs(shared.__cause__, tail)
        self.assertIsNone(later.__cause__)
        self.assertGreaterEqual(len(calls), 3)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_policy_first_errors_precede_loader(self):
        cases = (
            (b"[", "Expecting value"),
            (
                b'{"commit_signatures":{"required":true,"keys":[]}}',
                "commit_signatures.keys must be an object",
            ),
        )
        for raw, expected in cases:
            with self.subTest(expected=expected):
                self.policy_path.write_bytes(raw)
                ledger = self.ledger()
                authority = self.Authority(self.project, ledger)
                calls = []
                with self.assertRaisesRegex(Exception, expected):
                    authority.prepare_policy(self.empty_loader(calls), ledger)
                authority.close(ledger)
                self.assertEqual(calls, [])
                self.assert_closed(ledger)

    def test_malformed_policy_error_does_not_retain_raw_subject(self):
        self.policy_path.write_bytes(b"[distinctive-secret-policy")
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        pinned_raw = authority._policy_raw
        caught = None
        try:
            authority.prepare_policy(self.empty_loader(), ledger)
        except self.namespace["CFError"] as error:
            caught = error
        self.assertIsNotNone(caught)
        self.assertIn("Expecting value", str(caught))
        self.assertFalse(hasattr(caught, "doc"))
        self.assertIsNone(caught.__cause__)
        self.assertIsNone(caught.__context__)
        authority.close(ledger)
        traceback = caught.__traceback__
        authority_frames = []
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "prepare_policy":
                authority_frames.append(traceback.tb_frame)
            traceback = traceback.tb_next
        self.assertTrue(authority_frames)
        self.assertTrue(
            all(
                all(
                    value is not pinned_raw
                    for value in frame.f_locals.values()
                )
                for frame in authority_frames
            )
        )
        self.assert_closed(ledger)

    def test_invalid_utf8_error_does_not_retain_raw_subject(self):
        self.policy_path.write_bytes(b"\xffdistinctive-secret-policy")
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        pinned_raw = authority._policy_raw
        caught = None
        try:
            authority.prepare_policy(self.empty_loader(), ledger)
        except self.namespace["CFError"] as error:
            caught = error
        self.assertIsNotNone(caught)
        self.assertIn("UTF-8 is invalid", str(caught))
        self.assertFalse(hasattr(caught, "object"))
        self.assertIsNone(caught.__cause__)
        self.assertIsNone(caught.__context__)
        authority.close(ledger)
        traceback = caught.__traceback__
        authority_frames = []
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "prepare_policy":
                authority_frames.append(traceback.tb_frame)
            traceback = traceback.tb_next
        self.assertTrue(authority_frames)
        self.assertTrue(
            all(
                all(
                    value is not pinned_raw
                    for value in frame.f_locals.values()
                )
                for frame in authority_frames
            )
        )
        self.assert_closed(ledger)

    def test_child_constructor_failure_detaches_payload_and_instance(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        pinned_raw = authority._policy_raw
        owner_class = self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ]
        instances = []
        raw_seen = False

        class FailingOwner:
            def __init__(self, policy_raw, *_args):
                nonlocal raw_seen
                raw_seen = policy_raw is pinned_raw
                instances.append(weakref.ref(self))
                raise RuntimeError("injected child constructor")

        self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ] = FailingOwner
        caught = None
        try:
            authority.prepare_policy(self.empty_loader(), ledger)
        except RuntimeError as error:
            caught = error
        finally:
            self.Authority.prepare_policy.__globals__[
                "AccountedCommitSignaturePolicy"
            ] = owner_class
        self.assertTrue(raw_seen)
        self.assertIsNotNone(caught)
        self.assertIsNone(caught.__cause__)
        self.assertIsNone(caught.__context__)
        gc.collect()
        self.assertIsNone(instances[0]())
        release = ledger.release
        observations = []

        def observe_release(dimension, amount):
            observations.append(
                (authority._policy_raw, authority._policy_digest)
            )
            return release(dimension, amount)

        ledger.release = observe_release
        try:
            authority.close(ledger)
        finally:
            ledger.release = release
        traceback = caught.__traceback__
        authority_frames = []
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "prepare_policy":
                authority_frames.append(traceback.tb_frame)
            traceback = traceback.tb_next
        self.assertTrue(authority_frames)
        self.assertTrue(
            all(
                all(
                    value is not pinned_raw
                    for value in frame.f_locals.values()
                )
                for frame in authority_frames
            )
        )
        self.assertTrue(observations)
        self.assertTrue(
            all(raw is None and digest is None for raw, digest in observations)
        )
        self.assert_closed(ledger)

    def test_prepare_policy_preserves_generic_child_cleanup_chain(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner_class = self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ]
        primary = RuntimeError("exact child primary")
        cleanup = OSError("exact child cleanup")
        cleanup_prior = LookupError("exact child cleanup prior")
        primary.__cause__ = cleanup
        cleanup.__cause__ = cleanup_prior

        class FailingOwner:
            def __init__(self, *_args):
                raise primary

        self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ] = FailingOwner
        try:
            with self.assertRaises(RuntimeError) as caught:
                authority.prepare_policy(self.empty_loader(), ledger)
        finally:
            self.Authority.prepare_policy.__globals__[
                "AccountedCommitSignaturePolicy"
            ] = owner_class
        self.assertIs(caught.exception, primary)
        self.assertIs(caught.exception.__cause__, cleanup)
        self.assertIs(cleanup.__cause__, cleanup_prior)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_decoder_payload_is_replaced_but_cleanup_tail_survives(self):
        self.write_policy()
        ledger = self.ledger()
        authority = self.Authority(self.project, ledger)
        owner_class = self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ]
        decoder = json.JSONDecodeError(
            "injected decoder", "distinctive decoder payload", 3
        )
        cleanup = RuntimeError("exact decoder cleanup")
        cleanup_prior = LookupError("exact decoder cleanup prior")
        decoder.__cause__ = cleanup
        cleanup.__cause__ = cleanup_prior

        class FailingOwner:
            def __init__(self, *_args):
                raise decoder

        self.Authority.prepare_policy.__globals__[
            "AccountedCommitSignaturePolicy"
        ] = FailingOwner
        try:
            with self.assertRaises(self.namespace["CFError"]) as caught:
                authority.prepare_policy(self.empty_loader(), ledger)
        finally:
            self.Authority.prepare_policy.__globals__[
                "AccountedCommitSignaturePolicy"
            ] = owner_class
        self.assertIsNot(caught.exception, decoder)
        self.assertFalse(hasattr(caught.exception, "doc"))
        self.assertIs(caught.exception.__cause__, cleanup)
        self.assertIs(cleanup.__cause__, cleanup_prior)
        self.assertIsNone(decoder.__cause__)
        authority.close(ledger)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
