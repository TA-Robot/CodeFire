import gc
import json
import os
import runpy
import shutil
import stat
import subprocess
import tempfile
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonUploadSealedOwnerTests(unittest.TestCase):
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
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.objects_root = self.tmp / ".codefire/objects"
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8")
        )["head"]

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def bounds(self, root=None):
        paths = list((root or self.objects_root).glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        return len(paths), sum(sizes), max(sizes, default=0)

    def ledger(self, overrides=None, label="sealed owner"):
        return self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(overrides),
            label,
            command_kind="http_request",
        )

    def prepare(self, ledger, head=None, root=None, deadline=None, bounds=None):
        count, aggregate, maximum = bounds or self.bounds(root)
        return self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            root or self.objects_root,
            head or self.head,
            record_count_upper=count,
            aggregate_bytes_upper=aggregate,
            maximum_record_bytes_upper=maximum,
            deadline=deadline,
        )

    def write_commit(self, payload):
        object_id = self.namespace["object_id"]("commit", payload)
        record = {
            "object_id": object_id,
            "type": "commit",
            "hash": self.namespace["object_digest"]("commit", payload),
            "payload": payload,
        }
        path = self.objects_root / "commits" / f"{object_id}.json"
        path.write_text(
            json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        return object_id

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_exact_minus_one_precedes_session_allocator(self):
        Owner = self.namespace["AccountedHTTPUploadSealedValidation"]
        count, aggregate, maximum = self.bounds()
        retained, _temporary = Owner.allocation_upper(count, aggregate, maximum)
        globals_ = self.namespace["prepare_http_upload_sealed_validation"].__globals__
        Session = globals_["SealedCommitValidationSession"]
        allocations = []

        class ObservedSession(Session):
            def __init__(self, *args, **kwargs):
                allocations.append(True)
                super().__init__(*args, **kwargs)

        globals_["SealedCommitValidationSession"] = ObservedSession
        try:
            rejected = self.ledger(
                {"retained_bytes": ("concurrent", retained - 1)}, "sealed -1"
            )
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.prepare(rejected)
            self.assertEqual(allocations, [])
            self.assert_closed(rejected)

            exact = self.ledger(
                {"retained_bytes": ("concurrent", retained)}, "sealed exact"
            )
            owner = self.prepare(exact)
            self.assertEqual(len(allocations), 1)
            json_module = globals_["json"]
            parsed_input_types = []

            class ObservedJSON:
                def __getattr__(self, name):
                    return getattr(json_module, name)

                def loads(self, value, *args, **kwargs):
                    parsed_input_types.append(type(value))
                    return json_module.loads(value, *args, **kwargs)

            globals_["json"] = ObservedJSON()
            try:
                owner.validate(exact)
            finally:
                globals_["json"] = json_module
            self.assertTrue(parsed_input_types)
            self.assertTrue(all(value is bytes for value in parsed_input_types))
            owner.finish(exact)
            self.assert_closed(exact)

            head_size = (
                self.objects_root / "commits" / f"{self.head}.json"
            ).stat().st_size
            false_ledger = self.ledger(label="sealed false source upper")
            false_owner = self.prepare(
                false_ledger,
                bounds=(count, head_size - 1, head_size - 1),
            )
            frame_allocations = []
            charge_frame = false_owner._session._charge_frame

            def observe_frame(*args, **kwargs):
                frame_allocations.append(True)
                return charge_frame(*args, **kwargs)

            false_owner._session._charge_frame = observe_frame
            with self.assertRaisesRegex(
                self.namespace["CFError"], "record-size upper"
            ):
                false_owner.validate(false_ledger)
            self.assertEqual(frame_allocations, [])
            self.assert_closed(false_ledger)
        finally:
            globals_["SealedCommitValidationSession"] = Session

    def test_deep_shared_dag_and_deadline_cleanup(self):
        template = json.loads(
            (self.objects_root / "commits" / f"{self.head}.json").read_text(
                encoding="utf-8"
            )
        )["payload"]
        parent = self.head
        for index in range(180):
            payload = json.loads(json.dumps(template))
            payload["parents"] = [parent]
            payload["message"] = f"accounted-depth-{index}"
            parent = self.write_commit(payload)
        left = json.loads(json.dumps(template))
        left["parents"] = [parent]
        left["message"] = "shared-left"
        left_id = self.write_commit(left)
        right = json.loads(json.dumps(template))
        right["parents"] = [parent]
        right["message"] = "shared-right"
        right_id = self.write_commit(right)
        top = json.loads(json.dumps(template))
        top["parents"] = [left_id, right_id]
        top["message"] = "shared-top"
        top_id = self.write_commit(top)

        ledger = self.ledger(
            {
                "retained_bytes": ("concurrent", 512 * 1024 * 1024),
                "temporary_bytes": ("concurrent", 128 * 1024 * 1024),
            },
            "deep shared sealed",
        )
        owner = self.prepare(ledger, top_id)
        owner.validate(ledger)
        self.assertGreater(ledger.counters.get("revisits", 0), 0)
        owner.finish(ledger)
        self.assert_closed(ledger)

        class ExpireMidTraversal:
            def __init__(self):
                self.polls = 0

            def check(self, phase):
                self.assert_phase = phase
                self.polls += 1
                if self.polls == 80:
                    raise self_error(phase)

        def self_error(phase):
            return self.namespace["RequestDeadlineExceeded"](phase, 30_000)

        deadline = ExpireMidTraversal()
        interrupted = self.ledger(
            {
                "retained_bytes": ("concurrent", 512 * 1024 * 1024),
                "temporary_bytes": ("concurrent", 128 * 1024 * 1024),
            },
            "deadline sealed",
        )
        owner = self.prepare(interrupted, top_id, deadline=deadline)
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            owner.validate(interrupted)
        self.assertEqual(deadline.assert_phase, "upload sealed validation")
        self.assert_closed(interrupted)

    def test_cycle_and_parent_error_precedence_remain_dfs(self):
        root = self.tmp / "overlay"
        root.mkdir()
        first = "CF-COMMIT-aaaaaaaaaaaa"
        second = "CF-COMMIT-bbbbbbbbbbbb"
        loader_globals = self.namespace["validate_sealed_commit"].__globals__
        original_loader = loader_globals["load_object_from_root"]

        graphs = [
            (
                {first: {"type": "commit", "parents": [second]}, second: {"type": "commit", "parents": [first]}},
                first,
                "parent cycle detected",
            ),
            (
                {
                    first: {"type": "commit", "parents": [second, "CF-COMMIT-ffffffffffff"]},
                    second: {"type": "not-commit", "parents": []},
                },
                first,
                f"not a commit object: {second}",
            ),
        ]
        try:
            for graph, head, expected in graphs:
                def load_overlay(_root, object_id, graph=graph):
                    if object_id not in graph:
                        raise self.namespace["CFError"](f"object not found: {object_id}")
                    return graph[object_id]

                loader_globals["load_object_from_root"] = load_overlay
                sizes = [
                    self.namespace["canonical_json_size"]({"payload": payload})
                    for payload in graph.values()
                ]
                ledger = self.ledger(label="overlay sealed")
                owner = self.prepare(
                    ledger,
                    head,
                    root,
                    bounds=(len(graph), sum(sizes), max(sizes)),
                )
                with self.assertRaisesRegex(self.namespace["CFError"], expected):
                    owner.validate(ledger)
                self.assert_closed(ledger)
        finally:
            loader_globals["load_object_from_root"] = original_loader

    def test_foreign_ledger_and_source_mutation_are_fail_closed(self):
        ledger = self.ledger(label="source mutation")
        foreign = self.ledger(label="foreign")
        owner = self.prepare(ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "different ledger"
        ):
            owner.validate(foreign)
        self.assertGreater(ledger.current.get("retained_bytes", 0), 0)
        owner.validate(ledger)
        head_path = self.objects_root / "commits" / f"{self.head}.json"
        before = head_path.stat()
        raw = head_path.read_bytes()
        marker = raw.index(b"  ")
        head_path.write_bytes(raw[:marker] + b" \t" + raw[marker + 2 :])
        os.utime(head_path, ns=(before.st_atime_ns, before.st_mtime_ns))
        with self.assertRaises(self.namespace["SealedValidationSubjectChanged"]):
            owner.finish(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

        invalid_cases = []
        symlink_id = "CF-COMMIT-cccccccccccc"
        symlink_path = self.objects_root / "commits" / f"{symlink_id}.json"
        symlink_path.symlink_to(f"{self.head}.json")
        invalid_cases.append((symlink_id, "object not found"))
        oversized_id = "CF-COMMIT-dddddddddddd"
        oversized_path = self.objects_root / "commits" / f"{oversized_id}.json"
        with oversized_path.open("wb") as handle:
            handle.truncate(self.namespace["MAX_OBJECT_RECORD_BYTES"] + 1)
        invalid_cases.append((oversized_id, "exceeds the"))
        for invalid_id, expected in invalid_cases:
            invalid_ledger = self.ledger(label="invalid source precedence")
            invalid_owner = self.prepare(
                invalid_ledger,
                invalid_id,
                bounds=(1, 0, 0),
            )
            with self.assertRaisesRegex(self.namespace["CFError"], expected) as raised:
                invalid_owner.validate(invalid_ledger)
            self.assertNotIn("source upper", str(raised.exception))
            self.assert_closed(invalid_ledger)

    def test_allocator_cancel_and_payload_release_faults_leave_retry_owner(self):
        globals_ = self.namespace["prepare_http_upload_sealed_validation"].__globals__
        Session = globals_["SealedCommitValidationSession"]
        ledger = self.ledger(label="allocator fault")
        partial_session_refs = []
        partial_payload_refs = []
        allocation_release_observations = []

        class PartialPayload:
            pass

        class FailingSession:
            def __new__(cls, *_args, **_kwargs):
                value = super().__new__(cls)
                partial_session_refs.append(weakref.ref(value))
                return value

            def __init__(self, *_args, **_kwargs):
                self.payload = PartialPayload()
                partial_payload_refs.append(weakref.ref(self.payload))
                raise MemoryError("injected sealed session allocation")

        allocation_release = ledger.release

        def observe_allocation_release(dimension, amount):
            gc.collect()
            allocation_release_observations.append(
                (
                    dimension,
                    all(reference() is None for reference in partial_session_refs),
                    all(reference() is None for reference in partial_payload_refs),
                )
            )
            return allocation_release(dimension, amount)

        ledger.release = observe_allocation_release
        globals_["SealedCommitValidationSession"] = FailingSession
        try:
            with self.assertRaisesRegex(MemoryError, "injected sealed"):
                self.prepare(ledger)
        finally:
            globals_["SealedCommitValidationSession"] = Session
            ledger.release = allocation_release
        self.assertTrue(allocation_release_observations)
        self.assertTrue(
            all(
                session_dead and payload_dead
                for _dimension, session_dead, payload_dead in allocation_release_observations
            )
        )
        self.assert_closed(ledger)

        cancel_ledger = self.ledger(label="cancel fault")
        consume = cancel_ledger.consume_reserved
        cancel = cancel_ledger.cancel_permit
        cancel_calls = 0

        def fail_consume(*_args, **_kwargs):
            raise self.namespace["CFError"]("injected sealed consume")

        def fail_cancel_once(permit):
            nonlocal cancel_calls
            cancel_calls += 1
            if cancel_calls == 1:
                raise self.namespace["CFError"]("injected sealed cancel")
            return cancel(permit)

        cancel_ledger.consume_reserved = fail_consume
        cancel_ledger.cancel_permit = fail_cancel_once
        with self.assertRaisesRegex(self.namespace["CFError"], "injected sealed cancel"):
            self.prepare(cancel_ledger)
        cancel_ledger.consume_reserved = consume
        cancel_ledger.cancel_permit = cancel
        self.assertEqual(cancel_calls, 2)
        self.assert_closed(cancel_ledger)

        class PostCancelFault(BaseException):
            pass

        post_cancel_ledger = self.ledger(label="post-cancel fault")
        post_cancel_consume = post_cancel_ledger.consume_reserved
        post_cancel = post_cancel_ledger.cancel_permit
        post_cancel_calls = 0

        def fail_post_cancel_consume(*_args, **_kwargs):
            raise self.namespace["CFError"]("injected pre-consume failure")

        def cancel_then_raise(permit):
            nonlocal post_cancel_calls
            post_cancel_calls += 1
            post_cancel(permit)
            raise PostCancelFault("injected post-cancel failure")

        post_cancel_ledger.consume_reserved = fail_post_cancel_consume
        post_cancel_ledger.cancel_permit = cancel_then_raise
        try:
            with self.assertRaises(PostCancelFault):
                self.prepare(post_cancel_ledger)
        finally:
            post_cancel_ledger.consume_reserved = post_cancel_consume
            post_cancel_ledger.cancel_permit = post_cancel
        self.assertEqual(post_cancel_calls, 1)
        self.assert_closed(post_cancel_ledger)

        release_ledger = self.ledger(label="release retry")
        owner = self.prepare(release_ledger)
        owner.validate(release_ledger)
        document_refs = tuple(
            weakref.ref(document) for document in owner._session.documents.values()
        )
        self.assertTrue(document_refs)
        release = release_ledger.release
        observations = []

        def fail_release_once(dimension, amount):
            gc.collect()
            observations.append(
                (dimension, all(reference() is None for reference in document_refs))
            )
            if len(observations) == 1:
                raise self.namespace["CFError"]("injected sealed release")
            return release(dimension, amount)

        release_ledger.release = fail_release_once
        with self.assertRaisesRegex(
            self.namespace["CFError"], "injected sealed release"
        ):
            owner.finish(release_ledger)
        self.assertFalse(owner._closed)
        owner.close(release_ledger)
        self.assertTrue(owner._closed)
        self.assertTrue(all(payload_dead for _dimension, payload_dead in observations))
        self.assertTrue(all(reference() is None for reference in document_refs))
        self.assert_closed(release_ledger)

        class PostCommitReleaseFault(BaseException):
            pass

        for target_dimension in ("temporary_bytes", "retained_bytes"):
            post_commit_ledger = self.ledger(
                label=f"post-commit {target_dimension} release"
            )
            owner = self.prepare(post_commit_ledger)
            owner.validate(post_commit_ledger)
            original_release = post_commit_ledger.release
            injected = False

            def release_then_raise(dimension, amount):
                nonlocal injected
                original_release(dimension, amount)
                if dimension == target_dimension and not injected:
                    injected = True
                    raise PostCommitReleaseFault(target_dimension)

            post_commit_ledger.release = release_then_raise
            with self.assertRaises(PostCommitReleaseFault):
                owner.finish(post_commit_ledger)
            self.assertTrue(injected)
            owner.close(post_commit_ledger)
            post_commit_ledger.release = original_release
            self.assertTrue(owner._closed)
            self.assert_closed(post_commit_ledger)

    def test_post_consume_chained_failure_drops_all_traceback_payloads(self):
        ledger = self.ledger(label="post-consume chained failure")
        consume = ledger.consume_reserved
        release = ledger.release
        payload_refs = []
        release_observations = []

        class TracebackPayload:
            pass

        def raise_chain(depth):
            payload = TracebackPayload()
            payload_refs.append(weakref.ref(payload))
            if depth == 0:
                raise self.namespace["CFError"]("sealed chain leaf")
            try:
                raise_chain(depth - 1)
            except BaseException as cause:
                raise self.namespace["CFError"](
                    f"sealed chain level {depth}"
                ) from cause

        def consume_then_raise(*args, **kwargs):
            consume(*args, **kwargs)
            raise_chain(72)

        def observe_release(dimension, amount):
            gc.collect()
            release_observations.append(
                (dimension, all(reference() is None for reference in payload_refs))
            )
            return release(dimension, amount)

        ledger.consume_reserved = consume_then_raise
        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "sealed chain level 72"
            ):
                self.prepare(ledger)
        finally:
            ledger.consume_reserved = consume
            ledger.release = release
        self.assertEqual(len(payload_refs), 73)
        self.assertTrue(release_observations)
        self.assertTrue(
            all(payloads_dead for _dimension, payloads_dead in release_observations)
        )
        self.assertTrue(all(reference() is None for reference in payload_refs))
        self.assert_closed(ledger)

        first = self.namespace["CFError"]("cycle first")
        second = self.namespace["CFError"]("cycle second")
        first.__cause__ = second
        second.__cause__ = first
        detached = self.namespace[
            "AccountedHTTPUploadSealedValidation"
        ]._detach_failure(first)
        self.assertIs(detached, first)
        self.assertIsNone(first.__cause__)
        self.assertIsNone(second.__cause__)

    def test_root_id_precedence_and_typed_preflight_size_drift(self):
        validate_payload = self.namespace["_validate_sealed_commit_payload"]
        roots = {
            "content_manifest": "CF-CONTENT-aaaaaaaaaaaa",
            "atom_index": "invalid-atom-index",
            "trace_graph": "CF-TRACE-aaaaaaaaaaaa",
            "fire_delta": "CF-FIRE-aaaaaaaaaaaa",
            "verification": "CF-VERIFICATION-aaaaaaaaaaaa",
            "policy": "CF-POLICY-aaaaaaaaaaaa",
        }
        preflighted = []

        def reject_later_root(object_id):
            preflighted.append(object_id)
            raise self.namespace["CFError"]("injected record upper")

        with self.assertRaisesRegex(
            self.namespace["CFError"],
            "root atom_index must be an object id",
        ):
            validate_payload(
                {"roots": roots},
                lambda _object_id: self.fail("root payload must not load"),
                preflight_source=reject_later_root,
            )
        self.assertEqual(preflighted, [])

        ledger = self.ledger(label="typed preflight size drift")
        owner = self.prepare(ledger)
        owner._preflight_source(owner._session, self.head)
        expected_size = owner._source_sizes[self.head]
        head_path = self.objects_root / "commits" / f"{self.head}.json"
        with head_path.open("ab") as handle:
            handle.write(b" ")
        with self.assertRaises(
            self.namespace["SealedValidationSubjectChanged"]
        ) as raised:
            owner.validate(ledger)
        self.assertEqual(raised.exception.object_id, self.head)
        self.assertEqual(raised.exception.change_kind, "size_changed")
        self.assertEqual(raised.exception.expected_size, expected_size)
        self.assertEqual(raised.exception.actual_size, expected_size + 1)
        self.assertEqual(ledger.terminal_reason, "subject_changed")
        self.assert_closed(ledger)

    def test_session_release_debt_reconciliation_and_close_single_flight(self):
        class ReleaseFault(BaseException):
            pass

        for commit_before_raise in (False, True):
            ledger = self.ledger(
                label=f"document release {'post' if commit_before_raise else 'pre'}"
            )
            owner = self.prepare(ledger)
            owner.validate(ledger)
            session = owner._session
            self.assertGreater(session._pending_document_retained_bytes, 0)
            release_claim = owner._release_memory_claim
            injected = False

            def fail_document_release(dimension, amount):
                nonlocal injected
                if dimension == "retained_bytes" and not injected:
                    injected = True
                    if commit_before_raise:
                        release_claim(dimension, amount)
                    session.close(None)
                    raise ReleaseFault("injected document release")
                return release_claim(dimension, amount)

            owner._release_memory_claim = fail_document_release
            with self.assertRaises(ReleaseFault):
                owner.close(ledger)
            self.assertTrue(injected)
            if commit_before_raise:
                self.assertEqual(session._pending_document_retained_bytes, 0)
            else:
                self.assertGreater(session._pending_document_retained_bytes, 0)
            self.assertFalse(owner._closed)
            self.assertFalse(owner._closing)
            self.assertFalse(session._closing)
            owner._release_memory_claim = release_claim
            owner.close(ledger)
            self.assertTrue(owner._closed)
            self.assert_closed(ledger)

        for commit_before_raise in (False, True):
            ledger = self.ledger(
                label=f"frame release {'post' if commit_before_raise else 'pre'}"
            )
            owner = self.prepare(ledger)
            session = owner._session
            frame_id = "CF-COMMIT-eeeeeeeeeeee"
            session._charge_frame(frame_id, 0, 1, edge=False)
            frame_bytes = session._frame_bytes
            release_claim = owner._release_memory_claim

            def fail_frame_release(dimension, amount):
                if commit_before_raise:
                    release_claim(dimension, amount)
                raise ReleaseFault("injected frame release")

            owner._release_memory_claim = fail_frame_release
            with self.assertRaises(ReleaseFault):
                session._release_frame(frame_id)
            self.assertEqual(
                session._frame_bytes,
                0 if commit_before_raise else frame_bytes,
            )
            owner._release_memory_claim = release_claim
            owner.close(ledger)
            self.assert_closed(ledger)

        for commit_before_raise in (False, True):
            ledger = self.ledger(
                label=f"validate-finally {'post' if commit_before_raise else 'pre'}"
            )
            owner = self.prepare(ledger)
            session = owner._session
            release_claim = owner._release_memory_claim
            injected = False

            def fail_validate_finally_release(dimension, amount):
                nonlocal injected
                if dimension == "retained_bytes" and not injected:
                    injected = True
                    if commit_before_raise:
                        release_claim(dimension, amount)
                    raise ReleaseFault("injected validate-finally release")
                return release_claim(dimension, amount)

            owner._release_memory_claim = fail_validate_finally_release
            with self.assertRaises(ReleaseFault):
                session.validate_commit("CF-COMMIT-ffffffffffff")
            self.assertTrue(injected)
            if commit_before_raise:
                self.assertEqual(session._frame_bytes, 0)
            else:
                self.assertGreater(session._frame_bytes, 0)
            owner._release_memory_claim = release_claim
            owner.close(ledger)
            self.assert_closed(ledger)

        ledger = self.ledger(label="owner close single flight")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        release = ledger.release
        reentries = []

        def release_with_reentry(dimension, amount):
            release(dimension, amount)
            reentries.append(dimension)
            owner.close(ledger)

        ledger.release = release_with_reentry
        try:
            owner.finish(ledger)
        finally:
            ledger.release = release
        self.assertEqual(reentries, ["temporary_bytes", "retained_bytes"])
        self.assertTrue(owner._closed)
        self.assert_closed(ledger)

    def test_reserve_into_registration_faults_keep_owner_cleanup(self):
        for commit_before_raise in (False, True):
            ledger = self.ledger(
                label=f"reserve-into {'post' if commit_before_raise else 'pre'}"
            )
            register = ledger._register_reservation

            def fail_registration(permit, reservation):
                if commit_before_raise:
                    register(permit, reservation)
                raise self.namespace["CFError"]("injected sealed registration")

            ledger._register_reservation = fail_registration
            try:
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "injected sealed registration"
                ):
                    self.prepare(ledger)
            finally:
                ledger._register_reservation = register
            self.assert_closed(ledger)

    def test_descriptor_close_faults_never_retry_numeric_fd(self):
        globals_ = self.namespace["prepare_http_upload_sealed_validation"].__globals__
        Descriptor = globals_["_SealedValidationFileDescriptor"]

        class DescriptorFault(BaseException):
            pass

        ledger = self.ledger(label="root fstat construction cleanup")
        capture_identity = Descriptor.capture_identity
        close_fd = Descriptor._close_fd
        captured_fds = []
        close_failed = False

        def capture_root_then_raise(descriptor_owner):
            metadata = capture_identity(descriptor_owner)
            captured_fds.append(descriptor_owner.fileno())
            raise self.namespace["CFError"]("injected root fstat boundary")

        def fail_constructor_close_once(file_descriptor):
            nonlocal close_failed
            if not close_failed:
                close_failed = True
                raise DescriptorFault("injected constructor close")
            return close_fd(file_descriptor)

        Descriptor.capture_identity = capture_root_then_raise
        Descriptor._close_fd = staticmethod(fail_constructor_close_once)
        try:
            with self.assertRaises(DescriptorFault):
                self.prepare(ledger)
        finally:
            Descriptor.capture_identity = capture_identity
            Descriptor._close_fd = staticmethod(close_fd)
        self.assertTrue(close_failed)
        self.assertEqual(len(captured_fds), 1)
        # A synthetic exception before the captured builtin close is invoked
        # cannot be retried safely: numeric ownership is already consumed.
        os.fstat(captured_fds[0])
        os.close(captured_fds[0])
        self.assert_closed(ledger)

        for target_kind, commit_before_raise in (
            ("root", False),
            ("subdir", True),
        ):
            ledger = self.ledger(label=f"{target_kind} descriptor close fault")
            owner = self.prepare(ledger)
            owner.validate(ledger)
            session = owner._session
            target_owner = (
                session._root_fd_owner
                if target_kind == "root"
                else next(iter(session._subdir_fds.values()))
            )
            target_fd = target_owner.fileno()
            close_fd = Descriptor._close_fd
            injected = False

            def fail_target_close(file_descriptor):
                nonlocal injected
                if file_descriptor == target_fd and not injected:
                    injected = True
                    if commit_before_raise:
                        close_fd(file_descriptor)
                    raise DescriptorFault(f"injected {target_kind} close")
                return close_fd(file_descriptor)

            Descriptor._close_fd = staticmethod(fail_target_close)
            try:
                with self.assertRaises(DescriptorFault):
                    owner.close(ledger)
            finally:
                Descriptor._close_fd = staticmethod(close_fd)
            self.assertTrue(injected)
            self.assertEqual(target_owner._fd, -1)
            owner.close(ledger)
            self.assertEqual(session._fd_owners, [])
            self.assertEqual(session._subdir_fds, {})
            self.assert_closed(ledger)
            if commit_before_raise:
                with self.assertRaises(OSError) as closed_fd:
                    os.fstat(target_fd)
                self.assertEqual(closed_fd.exception.errno, 9)
            else:
                os.fstat(target_fd)
                os.close(target_fd)

        ledger = self.ledger(label="transient read descriptor close fault")
        owner = self.prepare(ledger)
        session = owner._session
        close_fd = Descriptor._close_fd
        injected = False
        leaked_read_fd = None

        def fail_first_read_close(file_descriptor):
            nonlocal injected, leaked_read_fd
            if not injected:
                injected = True
                leaked_read_fd = file_descriptor
                raise DescriptorFault("injected transient read close")
            return close_fd(file_descriptor)

        Descriptor._close_fd = staticmethod(fail_first_read_close)
        try:
            with self.assertRaises(DescriptorFault):
                owner.validate(ledger)
        finally:
            Descriptor._close_fd = staticmethod(close_fd)
        self.assertTrue(injected)
        self.assertTrue(owner._closed)
        self.assertEqual(session._fd_owners, [])
        self.assert_closed(ledger)
        os.fstat(leaked_read_fd)
        os.close(leaked_read_fd)

        ledger = self.ledger(label="subdir fstat transient owner")
        owner = self.prepare(ledger)
        session = owner._session
        capture_identity = Descriptor.capture_identity
        injected = False

        def capture_then_raise(descriptor_owner):
            nonlocal injected
            metadata = capture_identity(descriptor_owner)
            if descriptor_owner is not session._root_fd_owner and not injected:
                injected = True
                raise DescriptorFault("injected subdir fstat boundary")
            return metadata

        Descriptor.capture_identity = capture_then_raise
        try:
            with self.assertRaises(DescriptorFault):
                owner.validate(ledger)
        finally:
            Descriptor.capture_identity = capture_identity
        self.assertTrue(injected)
        self.assertEqual(session._subdir_fds, {})
        self.assertEqual(session._fd_owners, [])
        self.assert_closed(ledger)

        ledger = self.ledger(label="ambient same-inode descriptor reuse")
        owner = self.prepare(ledger)
        owner.validate(ledger)
        session = owner._session
        close_fd = Descriptor._close_fd
        ambient_fd = None
        ambient_identity = None

        def close_reuse_then_raise(file_descriptor):
            nonlocal ambient_fd, ambient_identity
            before = os.fstat(file_descriptor)
            close_fd(file_descriptor)
            if ambient_fd is None:
                flags = (
                    os.O_RDONLY
                    | getattr(os, "O_DIRECTORY", 0)
                    | getattr(os, "O_NOFOLLOW", 0)
                )
                ambient_fd = os.open(self.objects_root / "commits", flags)
                self.assertEqual(ambient_fd, file_descriptor)
                replacement = os.fstat(ambient_fd)
                ambient_identity = (replacement.st_dev, replacement.st_ino)
                self.assertEqual(
                    ambient_identity,
                    (before.st_dev, before.st_ino),
                )
                raise DescriptorFault("injected same-inode fd reuse")

        Descriptor._close_fd = staticmethod(close_reuse_then_raise)
        try:
            with self.assertRaises(DescriptorFault):
                owner.finish(ledger)
        finally:
            Descriptor._close_fd = staticmethod(close_fd)
        self.assertIsNotNone(ambient_fd)
        try:
            ambient_stat = os.fstat(ambient_fd)
            self.assertTrue(stat.S_ISDIR(ambient_stat.st_mode))
            self.assertEqual(
                (ambient_stat.st_dev, ambient_stat.st_ino),
                ambient_identity,
            )
            self.assertEqual(session._fd_owners, [])
            self.assertEqual(session._subdir_fds, {})
            self.assert_closed(ledger)
        finally:
            os.close(ambient_fd)


if __name__ == "__main__":
    unittest.main()
