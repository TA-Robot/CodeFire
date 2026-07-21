import gc
import json
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonSealedBorrowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Owner = cls.namespace["AccountedHTTPUploadSealedValidation"]
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        subprocess.run(
            [str(CODEFIRE), "init"],
            cwd=self.tmp,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.objects = self.tmp / ".codefire/objects"
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label="sealed borrow"):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def prepare(self, ledger):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        return self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=None,
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_signature_and_ancestry_borrow_retained_commit_only(self):
        ledger = self.ledger()
        owner = self.prepare(ledger)
        owner.validate(ledger)
        session = owner._session
        document = session.documents[self.head]
        payload = document.record["payload"]
        original_load = session.load_payload

        def forbidden_load(*_args, **_kwargs):
            raise AssertionError("borrow widened session authority")

        session.load_payload = forbidden_load
        signature = owner.borrow_commits(ledger, "signature")
        try:
            borrowed, upper = signature(self.head)
            self.assertIs(borrowed, payload)
            self.assertEqual(upper, document.retained_bytes)
        finally:
            borrowed = None
            signature.close()
        with self.assertRaisesRegex(self.namespace["CFError"], "single-use"):
            owner.borrow_commits(ledger, "signature")

        ancestry = owner.borrow_commits(ledger, "ancestry")
        try:
            self.assertIs(ancestry(self.head), payload)
        finally:
            ancestry.close(ledger)
            session.load_payload = original_load
        owner.finish(ledger)
        self.assert_closed(ledger)

    def test_borrow_requires_validation_mode_and_same_ledger(self):
        ledger = self.ledger()
        owner = self.prepare(ledger)
        foreign = self.ledger("foreign sealed borrow")
        with self.assertRaisesRegex(self.namespace["CFError"], "has not validated"):
            owner.borrow_commits(ledger, "signature")
        owner.validate(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "mode is invalid"):
            owner.borrow_commits(ledger, "digest")
        with self.assertRaisesRegex(self.namespace["CFError"], "different ledger"):
            owner.borrow_commits(foreign, "signature")
        borrow = owner.borrow_commits(ledger, "signature")
        with self.assertRaisesRegex(self.namespace["CFError"], "different ledger"):
            borrow.close(foreign)
        self.assertIsInstance(borrow(self.head), tuple)
        borrow.close(ledger)
        owner.finish(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

    def test_unknown_commit_never_loads_or_expands_authority(self):
        ledger = self.ledger()
        owner = self.prepare(ledger)
        owner.validate(ledger)
        session = owner._session
        document_ids = set(session.documents)
        original_load = session.load_payload
        calls = []

        def observe_load(*args, **kwargs):
            calls.append((args, kwargs))
            return original_load(*args, **kwargs)

        session.load_payload = observe_load
        borrow = owner.borrow_commits(ledger, "signature")
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "outside validated authority"
            ):
                borrow("CF-COMMIT-absent")
        finally:
            borrow.close()
        self.assertEqual(calls, [])
        self.assertEqual(set(session.documents), document_ids)
        owner.finish(ledger)
        self.assert_closed(ledger)

    def test_active_borrow_blocks_fence_and_is_revoked(self):
        ledger = self.ledger()
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "ancestry")
        with self.assertRaisesRegex(self.namespace["CFError"], "already active"):
            owner.borrow_commits(ledger, "signature")
        with self.assertRaisesRegex(self.namespace["CFError"], "still active"):
            owner.finish(ledger)
        self.assertIsInstance(borrow(self.head), dict)
        borrow.close()
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        owner.finish(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        self.assert_closed(ledger)

    def test_owner_close_revokes_active_borrow(self):
        ledger = self.ledger()
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        owner.close(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        borrow.close()
        self.assert_closed(ledger)

    def test_partial_handle_allocation_releases_preacquired_owner(self):
        prepare = self.namespace["prepare_http_upload_sealed_validation"]
        globals_map = prepare.__globals__
        borrow_type = globals_map["_HTTPUploadSealedCommitBorrow"]
        calls = []

        def fail_second(owner, mode):
            calls.append(mode)
            if mode == "ancestry":
                raise MemoryError("sealed borrow handle allocation")
            return borrow_type(owner, mode)

        globals_map["_HTTPUploadSealedCommitBorrow"] = fail_second
        ledger = self.ledger("sealed borrow allocation fault")
        try:
            with self.assertRaisesRegex(MemoryError, "handle allocation"):
                self.prepare(ledger)
        finally:
            globals_map["_HTTPUploadSealedCommitBorrow"] = borrow_type
        self.assertEqual(calls, ["signature", "ancestry"])
        self.assert_closed(ledger)

    def test_final_fence_rejects_reentrant_borrow(self):
        ledger = self.ledger("sealed borrow reentrant fence")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        observations = []

        class FenceProbe:
            def check(inner_self, phase):
                if observations or phase != "upload sealed validation":
                    return
                try:
                    owner.borrow_commits(ledger, "signature")
                except BaseException as error:
                    observations.append(str(error))

        owner._deadline = FenceProbe()
        owner.finish(ledger)
        self.assertEqual(len(observations), 1)
        self.assertIn("final fence is active", observations[0])
        self.assert_closed(ledger)

    def test_borrowed_payload_dies_before_scalar_release(self):
        class Payload(dict):
            pass

        ledger = self.ledger("sealed borrow payload order")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        document = owner._session.documents[self.head]
        payload = Payload(document.record["payload"])
        document.record["payload"] = payload
        payload_ref = weakref.ref(payload)
        borrow = owner.borrow_commits(ledger, "ancestry")
        borrowed = borrow(self.head)
        self.assertIs(borrowed, payload)
        borrowed = None
        payload = None
        document = None
        borrow.close()
        release = ledger.release
        observations = []

        def observe_release(dimension, amount):
            gc.collect()
            observations.append(payload_ref() is None)
            release(dimension, amount)

        ledger.release = observe_release
        owner.finish(ledger)
        self.assertTrue(observations)
        self.assertTrue(all(observations))
        self.assert_closed(ledger)

    def test_close_fault_revokes_borrow_before_retry(self):
        ledger = self.ledger("sealed borrow close retry")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        release = ledger.release
        failed = [False]

        def fail_retained_once(dimension, amount):
            if dimension == "retained_bytes" and not failed[0]:
                failed[0] = True
                raise RuntimeError("sealed borrow retained release")
            release(dimension, amount)

        ledger.release = fail_retained_once
        with self.assertRaisesRegex(RuntimeError, "retained release"):
            owner.close(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        ledger.release = release
        owner.close(ledger)
        self.assertTrue(failed[0])
        self.assert_closed(ledger)

    def test_final_fence_rejects_aba_after_borrow(self):
        ledger = self.ledger("sealed borrow final fence")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "ancestry")
        borrowed = borrow(self.head)
        self.assertEqual(borrowed.get("type"), "commit")
        borrowed = None
        borrow.close()
        path = self.namespace["object_record_path"](self.objects, self.head)
        self.assertIsNotNone(path)
        original = path.read_bytes()
        path.unlink()
        path.write_bytes(original)
        with self.assertRaises(self.namespace["SealedValidationSubjectChanged"]):
            owner.finish(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        self.assert_closed(ledger)

    def test_final_memory_fence_follows_disk_callbacks(self):
        ledger = self.ledger("sealed borrow post-disk mutation")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        session = owner._session
        borrow = owner.borrow_commits(ledger, "ancestry")
        payload = borrow(self.head)
        borrow.close()
        retained_alias = [payload]
        payload = None
        finalize = session.finalize

        def mutate_after_disk_fence(**kwargs):
            finalize(**kwargs)
            retained_alias[0]["certificate"]["failed_checks"] = 1
            retained_alias.clear()

        session.finalize = mutate_after_disk_fence
        with self.assertRaisesRegex(
            self.namespace["CFError"], "borrowed commit changed"
        ):
            owner.finish(ledger)
        self.assertEqual(retained_alias, [])
        self.assert_closed(ledger)

    def test_final_detach_defers_close_until_payload_drop(self):
        class Payload(dict):
            pass

        ledger = self.ledger("sealed borrow final detach close reentry")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        session = owner._session
        document = session.documents[self.head]
        payload = Payload(document.record["payload"])
        document.record["payload"] = payload
        payload_ref = weakref.ref(payload)
        borrow = owner.borrow_commits(ledger, "ancestry")
        borrowed = borrow(self.head)
        self.assertIs(borrowed, payload)
        borrowed = None
        payload = None
        document = None
        borrow.close()
        close_observations = []

        class FinalDetachProbe:
            def check(inner_self, _phase):
                if (
                    close_observations
                    or not owner._finishing
                    or not session.finalized
                ):
                    return
                owner.close(ledger)
                close_observations.append(owner._session is session)

        owner._deadline = FinalDetachProbe()
        release = ledger.release
        release_observations = []

        def observe_release(dimension, amount):
            gc.collect()
            release_observations.append((dimension, payload_ref() is None))
            release(dimension, amount)

        ledger.release = observe_release
        owner.finish(ledger)
        self.assertEqual(close_observations, [True])
        self.assertTrue(release_observations)
        self.assertTrue(all(dropped for _dimension, dropped in release_observations))
        self.assert_closed(ledger)

    def test_mutation_is_sealed_at_release_and_final_fence(self):
        mutations = (
            lambda payload: payload.__setitem__("borrowed-extra", True),
            lambda payload: payload["certificate"].__setitem__(
                "failed_checks", 1
            ),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                ledger = self.ledger(f"sealed borrow mutation {index}")
                owner = self.prepare(ledger)
                owner.validate(ledger)
                borrow = owner.borrow_commits(ledger, "signature")
                payload, _upper = borrow(self.head)
                mutate(payload)
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "borrowed commit changed"
                ):
                    borrow.close()
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "abort-only"
                ):
                    owner.finish(ledger)
                payload = None
                owner.close(ledger)
                self.assert_closed(ledger)

        ledger = self.ledger("sealed borrow post-release mutation")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "ancestry")
        payload = borrow(self.head)
        borrow.close()
        payload["certificate"]["failed_checks"] = 1
        # The borrow contract forbids retaining payload aliases through the
        # final fence. The mutation persists in the session record after this
        # local is dropped, so the fence still proves post-close tampering
        # without releasing scalar ownership under a live external payload.
        payload = None
        with self.assertRaisesRegex(
            self.namespace["CFError"], "borrowed commit changed"
        ):
            owner.finish(ledger)
        self.assert_closed(ledger)

    def test_borrow_fingerprint_is_accounted_and_deadline_polled(self):
        ledger = self.ledger("sealed borrow fingerprint accounting")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        checks = []

        class DeadlineProbe:
            def check(inner_self, phase):
                checks.append(phase)

        owner._deadline = DeadlineProbe()
        before = ledger.cumulative.get("snapshot_revalidation_bytes", 0)
        canonical_digest = owner._canonical_borrow_digest
        destination_observations = []

        def observe_destination(document):
            destination_observations.append(
                (
                    document.borrow_fingerprint_bytes,
                    owner._claimed_memory["retained_bytes"],
                )
            )
            return canonical_digest(document)

        owner._canonical_borrow_digest = observe_destination
        borrow = owner.borrow_commits(ledger, "signature")
        payload, _upper = borrow(self.head)
        borrow.close()
        owner._canonical_borrow_digest = canonical_digest
        after = ledger.cumulative.get("snapshot_revalidation_bytes", 0)
        self.assertGreater(after, before)
        self.assertGreaterEqual(len(checks), 4)
        self.assertTrue(destination_observations)
        self.assertTrue(
            all(
                fingerprint == owner.BORROW_FINGERPRINT_BYTES
                and retained >= fingerprint
                for fingerprint, retained in destination_observations
            )
        )
        payload = None
        owner.finish(ledger)
        self.assert_closed(ledger)

    def test_borrow_release_rejects_deadline_reentry(self):
        ledger = self.ledger("sealed borrow release reentry")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        payload, _upper = borrow(self.head)
        observations = []

        class ReleaseProbe:
            def check(inner_self, _phase):
                if not owner._borrow_releasing or observations:
                    return
                try:
                    owner.borrow_commits(ledger, "ancestry")
                except BaseException as error:
                    observations.append(str(error))
                try:
                    borrow.close()
                except BaseException as error:
                    observations.append(str(error))
                try:
                    owner.finish(ledger)
                except BaseException as error:
                    observations.append(str(error))
                owner.close(ledger)
                observations.append(owner._session is not None)

        owner._deadline = ReleaseProbe()
        borrow.close()
        self.assertEqual(len(observations), 4)
        self.assertIn("release is active", observations[0])
        self.assertIn("release is already active", observations[1])
        self.assertIn("release is active", observations[2])
        self.assertIs(observations[3], True)
        payload = None
        owner.finish(ledger)
        self.assert_closed(ledger)

    def test_borrow_lookup_rechecks_authority_after_callbacks(self):
        ledger = self.ledger("sealed borrow lookup reentry")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        observations = []

        class LookupProbe:
            def check(inner_self, _phase):
                if not owner._borrow_reading or observations:
                    return
                try:
                    borrow.close()
                except BaseException as error:
                    observations.append(str(error))
                try:
                    owner.borrow_commits(ledger, "ancestry")
                except BaseException as error:
                    observations.append(str(error))
                owner.close(ledger)
                observations.append(owner._session is not None)

        owner._deadline = LookupProbe()
        payload, _upper = borrow(self.head)
        self.assertEqual(len(observations), 3)
        self.assertIn("lookup is active", observations[0])
        self.assertIn("lookup is active", observations[1])
        self.assertIs(observations[2], True)
        self.assertNotEqual(borrow._token, 0)
        self.assertEqual(owner._active_borrow_token, borrow._token)
        payload = None
        borrow.close()
        owner.finish(ledger)
        self.assert_closed(ledger)

        ledger = self.ledger("sealed borrow lookup token recheck")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        revoked = []

        class TokenProbe:
            def check(inner_self, _phase):
                if not owner._borrow_reading or revoked:
                    return
                borrow._token = 0
                owner._active_borrow_token = 0
                revoked.append(True)

        owner._deadline = TokenProbe()
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        self.assertEqual(revoked, [True])
        owner.close(ledger)
        self.assert_closed(ledger)

    def test_revoked_lookup_error_drops_payload_traceback(self):
        class Payload(dict):
            pass

        ledger = self.ledger("sealed borrow revoked traceback")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        document = owner._session.documents[self.head]
        payload = Payload(document.record["payload"])
        document.record["payload"] = payload
        payload_ref = weakref.ref(payload)
        borrow = owner.borrow_commits(ledger, "signature")
        revoked = []

        class RevokeProbe:
            def check(inner_self, _phase):
                if not owner._borrow_reading or revoked:
                    return
                borrow._token = 0
                owner._active_borrow_token = 0
                revoked.append(True)

        owner._deadline = RevokeProbe()
        retained_error = None
        try:
            borrow(self.head)
        except BaseException as error:
            retained_error = error
        self.assertIsNotNone(retained_error)
        self.assertIn("revoked", str(retained_error))
        payload = None
        document = None
        owner.close(ledger)
        gc.collect()
        self.assertIsNone(payload_ref())
        self.assertIsNotNone(retained_error)
        retained_error = None
        self.assert_closed(ledger)

    def test_fixed_borrow_controls_fit_declared_upper(self):
        ledger = self.ledger("sealed borrow control upper")
        owner = self.prepare(ledger)
        controls = sys.getsizeof(owner._borrow_handles)
        for handle in owner._borrow_handles.values():
            controls += sys.getsizeof(handle) + sys.getsizeof(handle.__dict__)
        self.assertLessEqual(controls, owner.BORROW_CONTROL_BYTES)
        owner.close(ledger)
        self.assert_closed(ledger)

    def test_activate_and_revoke_faults_never_strand_authority(self):
        borrow_type = self.namespace["_HTTPUploadSealedCommitBorrow"]
        original_activate = borrow_type._activate
        original_revoke = borrow_type._revoke

        ledger = self.ledger("sealed borrow post-activate fault")
        owner = self.prepare(ledger)
        owner.validate(ledger)

        def activate_then_fail(handle, token):
            original_activate(handle, token)
            raise KeyboardInterrupt("post-activate fault")

        borrow_type._activate = activate_then_fail
        try:
            with self.assertRaisesRegex(KeyboardInterrupt, "post-activate fault"):
                owner.borrow_commits(ledger, "signature")
        finally:
            borrow_type._activate = original_activate
        signature_handle = owner._borrow_handles["signature"]
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            signature_handle(self.head)
        with self.assertRaisesRegex(self.namespace["CFError"], "abort-only"):
            owner.finish(ledger)
        owner.close(ledger)
        self.assert_closed(ledger)

        ledger = self.ledger("sealed borrow pre-activate fault")
        owner = self.prepare(ledger)
        owner.validate(ledger)

        def fail_before_activate(_handle, _token):
            raise RuntimeError("pre-activate fault")

        borrow_type._activate = fail_before_activate
        try:
            with self.assertRaisesRegex(RuntimeError, "pre-activate fault"):
                owner.borrow_commits(ledger, "signature")
        finally:
            borrow_type._activate = original_activate
        with self.assertRaisesRegex(self.namespace["CFError"], "abort-only"):
            owner.finish(ledger)
        owner.close(ledger)
        self.assert_closed(ledger)

        ledger = self.ledger("sealed borrow post-revoke fault")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "signature")
        self.assertIsInstance(borrow(self.head), tuple)

        def revoke_then_fail(handle, selected_owner):
            original_revoke(handle, selected_owner)
            raise RuntimeError("post-revoke fault")

        borrow_type._revoke = revoke_then_fail
        try:
            with self.assertRaisesRegex(RuntimeError, "post-revoke fault"):
                borrow.close()
        finally:
            borrow_type._revoke = original_revoke
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        borrow.close()
        owner.finish(ledger)
        self.assert_closed(ledger)

        ledger = self.ledger("sealed borrow pre-revoke fault")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        borrow = owner.borrow_commits(ledger, "ancestry")

        def fail_before_revoke(_handle, _selected_owner):
            raise RuntimeError("pre-revoke fault")

        borrow_type._revoke = fail_before_revoke
        try:
            with self.assertRaisesRegex(RuntimeError, "pre-revoke fault"):
                borrow.close()
        finally:
            borrow_type._revoke = original_revoke
        with self.assertRaisesRegex(self.namespace["CFError"], "revoked"):
            borrow(self.head)
        borrow.close()
        owner.finish(ledger)
        self.assert_closed(ledger)

    def test_finish_cleanup_failure_precedes_domain_failure_and_retries(self):
        ledger = self.ledger("sealed borrow double failure")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        owner._session.finalize = lambda **_kwargs: (_ for _ in ()).throw(
            RuntimeError("final fence domain failure")
        )
        release = ledger.release
        failed = [False]

        def fail_retained_once(dimension, amount):
            if dimension == "retained_bytes" and not failed[0]:
                failed[0] = True
                raise RuntimeError("final fence cleanup failure")
            release(dimension, amount)

        ledger.release = fail_retained_once
        with self.assertRaisesRegex(RuntimeError, "cleanup failure"):
            owner.finish(ledger)
        ledger.release = release
        owner.close(ledger)
        self.assertTrue(failed[0])
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
