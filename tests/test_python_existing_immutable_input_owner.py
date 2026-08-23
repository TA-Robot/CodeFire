import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class InputDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None
        self.match = None
        self.calls = []

    def check(self, phase):
        self.calls.append(phase)
        callback = self.callback
        if callback is not None and (
            self.match is None or self.match in phase
        ):
            self.callback = None
            callback(phase)


class HookDict(dict):
    armed = False
    calls = 0

    def items(self):
        if self.armed:
            type(self).calls += 1
            raise AssertionError("dict hook executed")
        return super().items()


class PythonExistingImmutableInputTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Authority = cls.ns["AccountedRemoteSignaturePolicyAuthority"]
        cls.Pair = cls.ns["AccountedHTTPUploadExistingTerminalOwner"]
        cls.Carrier = cls.ns[
            "AccountedHTTPUploadExistingImmutableInputOwner"
        ]
        cls.CFError = cls.ns["CFError"]

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
        self.initial = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text()
        )["head"]
        payload = self.ns["load_object_from_root"](
            self.objects, self.initial
        )
        child = json.loads(json.dumps(payload))
        child["message"] = "immutable input owner"
        child["parents"] = [self.initial]
        self.key = "immutable-input-secret"
        child = self.ns["sign_commit_payload"](child, "alice", self.key)
        self.head = self.ns["store_object"](self.tmp, "commit", child)
        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        (self.project / "server_policy.json").write_bytes(
            json.dumps(
                {
                    "commit_signatures": {
                        "required": True,
                        "keys": {"alice": self.key},
                    },
                    "verification": {"required": []},
                },
                separators=(",", ":"),
            ).encode()
        )

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertIsNone(ledger._upload_terminal_owner)

    def _pair(self, ledger, deadline, branch="main"):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.ns["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
        )
        sealed.validate(ledger)
        policy = self.Authority(self.project, ledger, deadline)
        pair = self.Pair(sealed, policy, ledger)
        pair.verify_signature_and_server(branch, self.head, ledger)
        self.assertTrue(pair.contains_ancestor(self.initial, self.head, ledger))
        pair.finalize_pair(ledger)
        return pair

    def _semantic(self, ledger, deadline, branch="main", head=None):
        records = [
            json.loads(path.read_text())
            for path in sorted(self.objects.glob("*/*.json"))
        ]
        request = {
            "branch": "main",
            "head": self.head if head is None else head,
            "objects": records,
        }
        exact = len(self.ns["canonical_json"](request))
        token = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        token.acquire(ledger)
        token.reconcile_exact(ledger, exact)
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, token
        )
        semantic = self.ns["prepare_http_upload_semantic"](
            source,
            ledger,
            "unproved-project-candidate",
            branch,
            deadline,
        )
        semantic.prepare(ledger)
        return request, semantic

    def _prepared(self, branch="main", semantic_branch=None):
        ledger = self.Ledger(
            self.Profile.http_v1(),
            "immutable input owner",
            command_kind="http_request",
        )
        deadline = InputDeadline()
        pair = self._pair(ledger, deadline, branch)
        request, semantic = self._semantic(
            ledger,
            deadline,
            branch if semantic_branch is None else semantic_branch,
        )
        return ledger, deadline, pair, request, semantic

    def test_ready_is_payload_opaque_and_discard_is_single_use(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        owner.finalize(ledger)
        self.assertEqual(owner._state, owner.READY)
        self.assertIs(ledger._upload_terminal_owner, owner)
        self.assertFalse(hasattr(owner, "payload"))
        self.assertFalse(hasattr(owner, "semantic"))
        self.assertFalse(hasattr(owner, "copy_bytes"))
        self.assertEqual(
            deadline.calls.count("upload immutable input terminal"), 1
        )
        with self.assertRaisesRegex(self.CFError, "single-use"):
            owner.finalize(ledger)
        owner.close(ledger)
        owner.close(ledger)
        self._zero(ledger)

    def test_branch_and_head_mismatch_mint_nothing(self):
        for semantic_branch, mutate_head, message in (
            ("other", False, "branch subject mismatch"),
            ("main", True, "head subject mismatch"),
        ):
            with self.subTest(message=message):
                ledger, _deadline, pair, _request, semantic = self._prepared(
                    semantic_branch=semantic_branch
                )
                if mutate_head:
                    semantic._semantic["head"] = self.initial
                with self.assertRaisesRegex(self.CFError, message):
                    self.Carrier(pair, semantic, ledger)
                semantic.close(ledger)
                pair.close(ledger)
                self._zero(ledger)

    def test_stale_pair_and_semantic_aliases_all_reject(self):
        ledger, _deadline, pair, _request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        calls = (
            lambda: pair.contains_ancestor(self.initial, self.head, ledger),
            lambda: pair.finalize_pair(ledger),
            lambda: pair.close(ledger),
            lambda: semantic.matches_semantic({}),
            lambda: semantic.terminal_detach(ledger),
            lambda: semantic.close(ledger),
        )
        for call in calls:
            with self.assertRaisesRegex(self.CFError, "adopted"):
                call()
        owner.close(ledger)
        self._zero(ledger)

    def test_second_semantic_and_pair_adoption_are_rejected(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        _second_request, second = self._semantic(ledger, deadline)
        owner = self.Carrier(pair, semantic, ledger)
        with self.assertRaisesRegex(self.CFError, "already adopted"):
            self.Carrier(pair, second, ledger)
        second.close(ledger)
        owner.close(ledger)
        self._zero(ledger)

    def test_transfer_faults_leave_no_unreachable_registry_owner(self):
        for post_effect in (False, True):
            with self.subTest(post_effect=post_effect):
                ledger, _deadline, pair, _request, semantic = self._prepared()
                original = self.Carrier._bind_source_tokens

                def fail(selected_pair, selected_semantic, selected_owner):
                    if post_effect:
                        original(selected_pair, selected_semantic, selected_owner)
                    raise RuntimeError("transfer fault")

                self.Carrier._bind_source_tokens = staticmethod(fail)
                try:
                    with self.assertRaisesRegex(RuntimeError, "transfer fault"):
                        self.Carrier(pair, semantic, ledger)
                finally:
                    self.Carrier._bind_source_tokens = staticmethod(original)
                self._zero(ledger)

    def test_equal_length_request_and_record_mutations_abort(self):
        mutations = (
            lambda request, semantic: request.__setitem__("branch", "mAin"),
            lambda request, semantic: request["objects"].reverse(),
            lambda request, semantic: request["objects"].__setitem__(
                0, json.loads(json.dumps(request["objects"][0]))
            ),
            lambda _request, semantic: semantic._semantic.__setitem__(
                "remote_branch", "mAin"
            ),
            lambda _request, semantic: setattr(
                semantic, "_request_digest", "sha256:" + "0" * 64
            ),
            lambda request, semantic: setattr(
                semantic._source, "_request", dict(request)
            ),
            lambda request, semantic: (
                setattr(semantic._source, "_records", list(request["objects"])),
                semantic._source._request.__setitem__(
                    "objects", semantic._source._records
                ),
            ),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                ledger, _deadline, pair, request, semantic = self._prepared()
                owner = self.Carrier(pair, semantic, ledger)
                mutation(request, semantic)
                with self.assertRaises(self.CFError):
                    owner.finalize(ledger)
                self._zero(ledger)

    def test_last_callback_rejects_subclass_without_executing_hook(self):
        ledger, deadline, pair, request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        original = request["objects"][0]
        hooked = HookDict(original)

        def mutate(_phase):
            request["objects"][0] = hooked
            HookDict.armed = True

        deadline.callback = mutate
        deadline.match = "immutable input terminal"
        HookDict.calls = 0
        try:
            with self.assertRaisesRegex(self.CFError, "exact built-in JSON"):
                owner.finalize(ledger)
        finally:
            HookDict.armed = False
        self.assertEqual(HookDict.calls, 0)
        self._zero(ledger)

    def test_last_callback_huge_tree_stops_at_parser_bound(self):
        ledger, deadline, pair, request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        deadline.callback = lambda _phase: request.__setitem__(
            "objects", [None] * 1_000_000
        )
        deadline.match = "immutable input terminal"
        with self.assertRaises(self.CFError):
            owner.finalize(ledger)
        self._zero(ledger)

    def test_reentry_close_is_typed_cancellation(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        observations = []

        def cancel(_phase):
            try:
                owner.finalize(ledger)
            except self.CFError as error:
                observations.append(str(error))
            owner.close(ledger)

        deadline.callback = cancel
        deadline.match = "immutable input terminal"
        with self.assertRaisesRegex(self.CFError, "cancelled"):
            owner.finalize(ledger)
        self.assertEqual(len(observations), 1)
        self.assertIn("in progress", observations[0])
        self._zero(ledger)

    def test_post_revalidation_release_fault_retries_without_probe(self):
        ledger, deadline, pair, _request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        release = ledger.release
        fired = False

        def fail_once(dimension, amount):
            nonlocal fired
            if not fired and dimension == "temporary_bytes":
                fired = True
                raise RuntimeError("terminal release fault")
            return release(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "terminal release fault"):
            owner.finalize(ledger)
        self.assertTrue(semantic._terminal_revalidated)
        ledger.release = release
        owner.finalize(ledger)
        self.assertEqual(
            deadline.calls.count("upload immutable input terminal"), 1
        )
        owner.close(ledger)
        self._zero(ledger)

    def test_cleanup_release_fault_stops_and_retries_in_order(self):
        ledger, _deadline, pair, _request, semantic = self._prepared()
        owner = self.Carrier(pair, semantic, ledger)
        owner.finalize(ledger)
        release = ledger.release
        fired = False

        def fail_once(dimension, amount):
            nonlocal fired
            if not fired and dimension == "retained_bytes":
                fired = True
                raise RuntimeError("cleanup release fault")
            return release(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "cleanup release fault"):
            owner.close(ledger)
        self.assertIs(ledger._upload_terminal_owner, owner)
        self.assertIs(owner._pair_owner, pair)
        ledger.release = release
        owner.close(ledger)
        self._zero(ledger)


if __name__ == "__main__":
    unittest.main()
