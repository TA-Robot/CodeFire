import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonAncestryBorrowHandoffTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.CFError = cls.namespace["CFError"]

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
        self.initial_head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label="sealed ancestry handoff"):
        return self.Ledger(
            self.Profile.http_v1(
                {
                    "retained_bytes": ("concurrent", 512 * 1024 * 1024),
                    "temporary_bytes": ("concurrent", 128 * 1024 * 1024),
                }
            ),
            label,
            command_kind="http_request",
        )

    def write_commit(self, payload):
        object_id = self.namespace["object_id"]("commit", payload)
        record = {
            "object_id": object_id,
            "type": "commit",
            "hash": self.namespace["object_digest"]("commit", payload),
            "payload": payload,
        }
        path = self.objects / "commits" / f"{object_id}.json"
        path.write_text(
            json.dumps(record, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        return object_id

    def graph(self):
        template = json.loads(
            (self.objects / "commits" / f"{self.initial_head}.json").read_text(
                encoding="utf-8"
            )
        )["payload"]

        def commit(message, parents):
            payload = json.loads(json.dumps(template))
            payload["message"] = message
            payload["parents"] = parents
            return self.write_commit(payload)

        shared = commit("ancestry-shared", [self.initial_head])
        left = commit("ancestry-left", [shared])
        right = commit("ancestry-right", [shared])
        top = commit("ancestry-top", [left, right])
        return top, left, right, shared

    def bounds(self):
        sizes = [path.stat().st_size for path in self.objects.glob("*/*.json")]
        return len(sizes), sum(sizes), max(sizes)

    def prepare_sealed(self, ledger, head, deadline=None):
        count, aggregate, maximum = self.bounds()
        owner = self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            head,
            record_count_upper=count,
            aggregate_bytes_upper=aggregate,
            maximum_record_bytes_upper=maximum,
            deadline=deadline,
        )
        owner.validate(ledger)
        return owner

    def prepare_handoff(self, owner, ledger, deadline=None):
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            owner, ledger, deadline
        )
        handoff.prepare(ledger)
        return handoff

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_full_bfs_uses_retained_borrow_without_loader_or_reopen(self):
        top, left, right, shared = self.graph()
        ledger = self.ledger()
        sealed = self.prepare_sealed(ledger, top)
        session = sealed._session

        def forbidden_load(*_args, **_kwargs):
            raise AssertionError("ancestry handoff widened sealed authority")

        session.load_payload = forbidden_load
        revisits_before = ledger.counters.get("revisits", 0)
        Borrow = self.namespace["_HTTPUploadSealedCommitBorrow"]
        original_call = Borrow.__call__
        calls = []

        def observe_call(handle, object_id):
            calls.append(object_id)
            return original_call(handle, object_id)

        Borrow.__call__ = observe_call
        try:
            handoff = self.prepare_handoff(sealed, ledger)
            self.assertTrue(handoff.contains(top, top))
        finally:
            Borrow.__call__ = original_call
        self.assertEqual(
            calls, [top, left, right, shared, self.initial_head]
        )
        self.assertEqual(
            ledger.counters.get("revisits", 0) - revisits_before, 1
        )
        self.assertTrue(handoff._closed)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_same_ledger_and_exact_deadline_precede_activation(self):
        top, _left, _right, _shared = self.graph()

        class Deadline:
            def check(self, _phase):
                return None

        deadline = Deadline()
        ledger = self.ledger("handoff identity")
        foreign = self.ledger("handoff foreign")
        sealed = self.prepare_sealed(ledger, top, deadline)
        handle = sealed._borrow_handles["ancestry"]
        with self.assertRaisesRegex(self.CFError, "different ledger"):
            self.prepare_handoff(sealed, foreign, deadline)
        self.assertFalse(handle._used)
        with self.assertRaisesRegex(self.CFError, "share the sealed deadline"):
            self.prepare_handoff(sealed, ledger, Deadline())
        self.assertFalse(handle._used)
        handoff = self.prepare_handoff(sealed, ledger, deadline)
        handoff.close(ledger)
        sealed.finish(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

    def test_ancestry_constructor_failure_releases_activated_borrow(self):
        top, _left, _right, _shared = self.graph()
        ledger = self.ledger("handoff constructor fault")
        sealed = self.prepare_sealed(ledger, top)
        factory = self.namespace["prepare_http_upload_sealed_ancestry"]
        globals_map = factory.__globals__
        Ancestry = globals_map["AccountedHTTPUploadAncestry"]

        def fail_constructor(*_args, **_kwargs):
            raise MemoryError("ancestry constructor fault")

        globals_map["AccountedHTTPUploadAncestry"] = fail_constructor
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            sealed, ledger
        )
        try:
            with self.assertRaisesRegex(MemoryError, "constructor fault"):
                handoff.prepare(ledger)
        finally:
            globals_map["AccountedHTTPUploadAncestry"] = Ancestry
        handoff.close(ledger)
        self.assertEqual(sealed._active_borrow_token, 0)
        self.assertEqual(sealed._borrow_handles["ancestry"]._token, 0)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_partial_ancestry_constructor_cleanup_retries_published_owner(self):
        top, _left, _right, _shared = self.graph()
        ledger = self.ledger("handoff partial constructor cleanup")
        sealed = self.prepare_sealed(ledger, top)
        Ancestry = self.namespace["AccountedHTTPUploadAncestry"]
        allocate_seen = Ancestry._allocate_seen
        close_ancestry = Ancestry.close
        close_calls = 0

        def fail_seen():
            raise MemoryError("ancestry seen allocation")

        def fail_twice_cleanup(owner, *args, **kwargs):
            nonlocal close_calls
            close_calls += 1
            if close_calls <= 2:
                raise RuntimeError("ancestry constructor cleanup fault")
            return close_ancestry(owner, *args, **kwargs)

        Ancestry._allocate_seen = staticmethod(fail_seen)
        Ancestry.close = fail_twice_cleanup
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            sealed, ledger
        )
        try:
            with self.assertRaisesRegex(RuntimeError, "constructor cleanup fault"):
                handoff.prepare(ledger)
            published_ancestry = handoff._ancestry
            self.assertIsNotNone(published_ancestry)
            self.assertFalse(published_ancestry._closed)
            with self.assertRaisesRegex(
                RuntimeError, "constructor cleanup fault"
            ):
                handoff.close(ledger)
            self.assertIs(handoff._ancestry, published_ancestry)
            self.assertIsNotNone(handoff._borrow)
        finally:
            Ancestry._allocate_seen = staticmethod(allocate_seen)
            Ancestry.close = close_ancestry
        handoff.close(ledger)
        # The failed constructor's exact published owner remains reachable;
        # the caller retries cleanup through the already-owned composite.
        self.assertEqual(close_calls, 2)
        self.assertTrue(published_ancestry._closed)
        self.assertEqual(sealed._active_borrow_token, 0)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_activation_failure_is_abort_only_and_cleanup_owned(self):
        top, _left, _right, _shared = self.graph()
        ledger = self.ledger("handoff activation fault")
        sealed = self.prepare_sealed(ledger, top)
        handle = sealed._borrow_handles["ancestry"]
        activate = handle._activate

        def fail_after_activate(token):
            activate(token)
            raise KeyboardInterrupt("ancestry activation fault")

        handle._activate = fail_after_activate
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            sealed, ledger
        )
        with self.assertRaisesRegex(KeyboardInterrupt, "activation fault"):
            handoff.prepare(ledger)
        self.assertTrue(sealed._borrow_failed)
        self.assertEqual(sealed._active_borrow_token, 0)
        handoff.close(ledger)
        sealed.close(ledger)
        self.assert_closed(ledger)

    def test_prepare_callback_rejects_every_partial_public_surface(self):
        top, _left, _right, _shared = self.graph()

        class Deadline:
            def __init__(self):
                self.handoff = None
                self.armed = False
                self.observations = []

            def check(self, _phase):
                if not self.armed or self.handoff is None:
                    return
                self.armed = False
                for operation in [
                    lambda: self.handoff.contains(top, top),
                    lambda: self.handoff.prepare(ledger),
                ]:
                    try:
                        operation()
                    except BaseException as error:
                        self.observations.append(str(error))
                self.handoff.close()

        deadline = Deadline()
        ledger = self.ledger("handoff prepare reentry")
        sealed = self.prepare_sealed(ledger, top, deadline)
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            sealed, ledger, deadline
        )
        deadline.handoff = handoff
        deadline.armed = True
        with self.assertRaisesRegex(self.CFError, "cancelled by close"):
            handoff.prepare(ledger)
        self.assertEqual(len(deadline.observations), 2)
        self.assertIn("not prepared", deadline.observations[0])
        self.assertIn("prepare is already active", deadline.observations[1])
        self.assertTrue(handoff._closed)
        with self.assertRaisesRegex(self.CFError, "closed"):
            handoff.contains(top, top)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_callback_reentry_defers_close_and_rejects_second_decision(self):
        top, _left, _right, _shared = self.graph()

        class Deadline:
            def __init__(self):
                self.handoff = None
                self.fired = False
                self.errors = []

            def check(self, _phase):
                if self.handoff is None or self.fired:
                    return
                self.fired = True
                self.handoff.close()
                try:
                    self.handoff.contains(top, top)
                except BaseException as error:
                    self.errors.append(str(error))

        deadline = Deadline()
        ledger = self.ledger("handoff callback reentry")
        sealed = self.prepare_sealed(ledger, top, deadline)
        handoff = self.prepare_handoff(sealed, ledger, deadline)
        deadline.handoff = handoff
        self.assertTrue(handoff.contains(top, top))
        self.assertTrue(deadline.fired)
        self.assertEqual(len(deadline.errors), 1)
        self.assertIn("already active", deadline.errors[0])
        self.assertTrue(handoff._closed)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_ancestry_cleanup_fault_keeps_borrow_until_retry(self):
        top, _left, _right, _shared = self.graph()
        ledger = self.ledger("handoff ancestry cleanup retry")
        sealed = self.prepare_sealed(ledger, top)
        handoff = self.prepare_handoff(sealed, ledger)
        ancestry = handoff._ancestry
        borrow = handoff._borrow
        close_ancestry = ancestry.close
        calls = 0

        def fail_once(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 1:
                raise RuntimeError("ancestry close fault")
            return close_ancestry(*args, **kwargs)

        ancestry.close = fail_once
        with self.assertRaisesRegex(RuntimeError, "ancestry close fault"):
            handoff.close()
        self.assertIs(handoff._ancestry, ancestry)
        self.assertIs(handoff._borrow, borrow)
        self.assertNotEqual(borrow._token, 0)
        handoff.close()
        self.assertTrue(handoff._closed)
        sealed.finish(ledger)
        self.assert_closed(ledger)

    def test_post_effect_borrow_close_fault_is_retryable(self):
        top, _left, _right, _shared = self.graph()
        ledger = self.ledger("handoff borrow cleanup retry")
        sealed = self.prepare_sealed(ledger, top)
        handoff = self.prepare_handoff(sealed, ledger)
        borrow = handoff._borrow
        close_borrow = borrow.close
        calls = 0

        def fail_after_close(*args, **kwargs):
            nonlocal calls
            calls += 1
            close_borrow(*args, **kwargs)
            if calls == 1:
                raise RuntimeError("borrow post-close fault")

        borrow.close = fail_after_close
        with self.assertRaisesRegex(RuntimeError, "post-close fault"):
            handoff.close()
        self.assertIsNone(handoff._ancestry)
        self.assertIs(handoff._borrow, borrow)
        self.assertEqual(borrow._token, 0)
        handoff.close()
        self.assertTrue(handoff._closed)
        sealed.finish(ledger)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
