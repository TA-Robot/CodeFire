"""CFR-168: the live HTTP upload route owns its ancestry/signature traversals.

Every test drives the real ``handle_http_upload`` production entry point.  The
request-owned ``DecisionWorkLedger`` and ``HTTPRequestDeadline`` must reach both
fast-forward ancestry and commit-signature history, the fast-forward decision
must be goal-directed instead of building a reachable-history distance map, and
every budget/deadline failure must precede branch/object publication while
leaving no retained state, permit, or terminal debt behind.
"""

import gc
import hashlib
import json
import runpy
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import tracemalloc
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class InjectedClock:
    """One monotonic, non-decreasing clock advanced only by the test."""

    def __init__(self) -> None:
        self.now = 0.0

    def __call__(self) -> float:
        return self.now

    def expire(self) -> None:
        self.now += 1.0


class PythonHTTPUploadAccountedTraversalTests(unittest.TestCase):
    DEADLINE_LIMIT_MS = 100

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
        self.globals = self.namespace["handle_http_upload"].__globals__
        self.local_objects = self.tmp / ".codefire/objects"
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        self.records = self.reachable_records(self.local_objects, self.head)
        self.template = self.object_record(self.local_objects, self.head)[
            "payload"
        ]
        self.storage_root = self.tmp / "http-storage"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    # ---------------------------------------------------------------- helpers

    def ledger(self, overrides=None, label="upload accounted traversal"):
        return self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(overrides),
            label,
            command_kind="http_request",
        )

    def deadline(self, clock):
        return self.namespace["HTTPRequestDeadline"](
            self.DEADLINE_LIMIT_MS, clock
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def project_root(self, app):
        return self.namespace["http_project_root"](
            self.storage_root, "org", app
        )

    def upload(self, app, head, records, *, ledger=None, deadline=None, key=None):
        request = {"branch": "main", "head": head, "objects": records}
        if key is not None:
            request["idempotency_key"] = key
        return self.namespace["handle_http_upload"](
            self.storage_root,
            "org",
            app,
            "main",
            request,
            decision_work=ledger,
            deadline=deadline,
        )

    def object_record(self, objects_root, object_id):
        path = self.namespace["object_record_path"](objects_root, object_id)
        self.assertIsNotNone(path, object_id)
        return self.namespace["object_record_core"](
            json.loads(path.read_text(encoding="utf-8"))
        )

    def reachable_records(self, objects_root, head):
        records = []
        seen = set()
        stack = [head]
        while stack:
            object_id = stack.pop()
            if object_id in seen:
                continue
            seen.add(object_id)
            record = self.object_record(objects_root, object_id)
            records.append(record)
            stack.extend(self.namespace["object_record_references"](record))
        return sorted(records, key=lambda record: record["object_id"])

    def commit_record(self, payload):
        object_id = self.namespace["object_id"]("commit", payload)
        return object_id, {
            "object_id": object_id,
            "type": "commit",
            "hash": self.namespace["object_digest"]("commit", payload),
            "payload": payload,
        }

    def commit(self, parents, message, *, signer=None, key=None):
        payload = json.loads(json.dumps(self.template))
        payload["parents"] = list(parents)
        payload["message"] = message
        if signer is not None:
            payload = self.namespace["sign_commit_payload"](
                payload, signer, key, signer
            )
        return self.commit_record(payload)

    def chain(self, parent, count, prefix):
        """Return ``(tip, records)`` for one linear chain above ``parent``."""
        records = []
        for index in range(count):
            parents = [] if parent is None else [parent]
            parent, record = self.commit(parents, f"{prefix}-{index}")
            records.append(record)
        return parent, records

    def signed_chain(self, parent, count, prefix, key):
        records = []
        for index in range(count):
            parent, record = self.commit(
                [parent], f"{prefix}-{index}", signer="alice", key=key
            )
            records.append(record)
        return parent, records

    def bootstrap(self, app):
        ledger = self.ledger(label=f"{app} bootstrap")
        status, _ = self.upload(app, self.head, self.records, ledger=ledger)
        self.assertEqual(status, 200)
        self.assert_closed(ledger)
        return self.project_root(app)

    def write_signature_policy(self, project_root, policy):
        (project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": policy}), encoding="utf-8"
        )

    def authority_snapshot(self, project_root):
        paths = (
            "remote_project.json",
            "generation.json",
            "branches/main.json",
            "request_nonce_cache.json",
            "pending_transaction.json",
            "server_policy.json",
        )
        return {
            path: (
                (project_root / path).read_bytes()
                if (project_root / path).is_file()
                else None
            )
            for path in paths
        }

    def durable_snapshot(self, project_root):
        """Digest every durable byte under one server project root.

        Covers the authority files, branch heads, the object store, the nonce
        cache, pending/applied transactions and idempotency receipts in one
        comparison, so a publication of any kind on a failing path cannot hide
        behind a narrower path list.

        ``locks/`` is the one exception, and only for its bytes.  Lock owner
        metadata is diagnostic and never authoritative -- it carries the
        holder's pid and an ISO timestamp, so it legitimately differs whenever a
        request takes the project gate.  Each lock path is still a key, so a
        leaked or newly created lock is still caught; only its contents are
        excluded.
        """
        snapshot = {}
        for path in sorted(project_root.rglob("*")):
            if not path.is_file():
                continue
            key = str(path.relative_to(project_root))
            snapshot[key] = (
                "<diagnostic lock metadata>"
                if key.startswith("locks/")
                else hashlib.sha256(path.read_bytes()).hexdigest()
            )
        return snapshot

    def branch_head(self, project_root):
        return json.loads(
            (project_root / "branches/main.json").read_text(encoding="utf-8")
        )["head"]

    def observe_traversals(self, samples):
        """Wrap both live traversal entry points and record their arguments."""
        signature = self.globals[
            "enforce_remote_commit_signature_policy_accounted"
        ]
        ancestry = self.globals["is_ancestor_from_objects_accounted"]

        def phase(name, ledger, deadline):
            return {
                "phase": name,
                "ledger": ledger,
                "deadline": deadline,
                "graph_nodes_before": ledger.cumulative.get("graph_nodes", 0),
                "record_reads_before": ledger.counters.get("record_reads", 0),
                "revisits_before": ledger.counters.get("revisits", 0),
                "retained_before": ledger.current.get("retained_bytes", 0),
                # The concurrent high-water reached by every phase that ran
                # before this one.  A later budget can be pinned just above it
                # so this phase owns the first refused acquisition.
                "retained_peak_before": ledger.peak.get("retained_bytes", 0),
                "polls_before": (
                    0 if deadline is None else deadline.poll_count
                ),
            }

        def finish(sample, ledger, deadline, decision):
            sample["graph_nodes"] = (
                ledger.cumulative.get("graph_nodes", 0)
                - sample["graph_nodes_before"]
            )
            sample["record_reads"] = (
                ledger.counters.get("record_reads", 0)
                - sample["record_reads_before"]
            )
            sample["polls"] = (
                0
                if deadline is None
                else deadline.poll_count - sample["polls_before"]
            )
            sample["graph_nodes_after"] = ledger.cumulative.get(
                "graph_nodes", 0
            )
            # Phase-scoped like every other measurement here: the bundle
            # object-collection walk that runs before ancestry charges its own
            # revisits on the same shared DAG, so only the delta attributes
            # work to this traversal's owner contract.
            sample["revisits"] = (
                ledger.counters.get("revisits", 0) - sample["revisits_before"]
            )
            sample["decision"] = decision

        def observed_signature(project_root, objects_root, commit_id, *rest):
            ledger, deadline = rest
            sample = phase("signature", ledger, deadline)
            samples.append(sample)
            try:
                signature(project_root, objects_root, commit_id, *rest)
            finally:
                finish(sample, ledger, deadline, None)

        def observed_ancestry(objects_root, ancestor, commit_id, *rest):
            ledger, deadline = rest
            sample = phase("ancestry", ledger, deadline)
            samples.append(sample)
            decision = None
            try:
                decision = ancestry(objects_root, ancestor, commit_id, *rest)
                return decision
            finally:
                finish(sample, ledger, deadline, decision)

        self.globals[
            "enforce_remote_commit_signature_policy_accounted"
        ] = observed_signature
        self.globals["is_ancestor_from_objects_accounted"] = observed_ancestry
        return lambda: self.restore_traversals(signature, ancestry)

    def restore_traversals(self, signature, ancestry):
        self.globals[
            "enforce_remote_commit_signature_policy_accounted"
        ] = signature
        self.globals["is_ancestor_from_objects_accounted"] = ancestry

    def observe_sealed_validation(self, samples):
        """Record every live sealed-validation preparation and its bounds."""
        prepare = self.globals["prepare_http_upload_sealed_validation"]

        def observed(ledger, objects_root, head, **kwargs):
            sample = {
                "record_count": kwargs["record_count_upper"],
                "aggregate_bytes": kwargs["aggregate_bytes_upper"],
                "maximum_record_bytes": kwargs["maximum_record_bytes_upper"],
                # The concurrent retained ownership already live when the
                # validation reservation is attempted.
                "retained_before": ledger.current.get("retained_bytes", 0),
                "owner": None,
            }
            samples.append(sample)
            sample["owner"] = prepare(ledger, objects_root, head, **kwargs)
            return sample["owner"]

        self.globals["prepare_http_upload_sealed_validation"] = observed
        return lambda: self.globals.__setitem__(
            "prepare_http_upload_sealed_validation", prepare
        )

    def observe_session_allocations(self, allocations):
        """Record every ``SealedCommitValidationSession`` construction."""
        session_type = self.globals["SealedCommitValidationSession"]

        class ObservedSession(session_type):
            def __init__(inner_self, *args, **kwargs):
                allocations.append(True)
                super().__init__(*args, **kwargs)

        self.globals["SealedCommitValidationSession"] = ObservedSession
        return lambda: self.globals.__setitem__(
            "SealedCommitValidationSession", session_type
        )

    # The reproduced CFB-138 per-record heap factor, pinned as a literal so no
    # product constant can quietly restore it.
    REPRODUCED_RECORD_HEAP_MULTIPLIER = 128

    def owner_lifetime_retained_bound(self, count, aggregate, maximum):
        """Recompute ``concurrent_retained_upper`` from the owner's constants.

        Every term is a lifetime that really overlaps: the owner's own fields,
        the two borrow handles, the validation session's reserved control state,
        one document control block plus one traversal frame plus one borrow
        fingerprint per record, the retained documents' own canonical measure
        once, the live frames' object ids once, and the pair-probe plan.
        ``maximum`` deliberately does not appear: no retained claim in the
        covered pool multiplies a record's bytes.
        """
        owner = self.namespace["AccountedHTTPUploadSealedValidation"]
        del maximum
        return (
            owner.RETAINED_BASE_BYTES
            + owner.BORROW_CONTROL_BYTES
            + owner._session_control_upper()
            + owner.PUBLICATION_PROOF_BYTES
            + owner.PUBLICATION_PROOF_ROOT_BYTES
            * owner.PUBLICATION_PROOF_ROOT_LIMIT
            + (owner.RETAINED_RECORD_BYTES + owner.BORROW_FINGERPRINT_BYTES)
            * count
            + 2 * aggregate
            + owner._pair_plan_upper(count)
        )

    def owner_lifetime_temporary_bound(self, count, maximum):
        """Recompute ``concurrent_temporary_upper`` from the owner's constants.

        The three per-record scratch sets never overlap, so the concurrent
        upper takes the largest instead of the sum, and the streamed canonical
        encoder contributes a constant plus its key projections rather than a
        whole-record duplicate.
        """
        owner = self.namespace["AccountedHTTPUploadSealedValidation"]
        read_scratch = owner.TEMPORARY_READ_SCRATCH_MULTIPLIER * maximum
        encoder_scratch = owner._stream_scratch_upper(maximum)
        probe_scratch = (
            owner.PAIR_PROBE_BYTES_MULTIPLIER * maximum + encoder_scratch
        )
        return (
            owner.TEMPORARY_BASE_BYTES
            + owner.PAIR_PROBE_TEMPORARY_BASE_BYTES
            + owner._pair_builder_temporary_upper(count)
            + owner._ordered_projection_upper(count)
            + owner.TEMPORARY_RECORD_BYTES * count
            + max(read_scratch, encoder_scratch, probe_scratch)
        )

    def reproduced_all_record_retained_bound(self, count, aggregate, maximum):
        """The CFB-138 reservation: every record's decode upper, live at once."""
        owner = self.namespace["AccountedHTTPUploadSealedValidation"]
        del maximum
        return (
            owner.RETAINED_BASE_BYTES
            + owner.BORROW_CONTROL_BYTES
            + (owner.RETAINED_RECORD_BYTES + owner.BORROW_FINGERPRINT_BYTES)
            * count
            + self.REPRODUCED_RECORD_HEAP_MULTIPLIER * aggregate
            + owner._pair_plan_upper(count)
        )

    def reproduced_max_record_retained_bound(self, count, aggregate, maximum):
        """The reviewed reservation: a per-record heap factor on one record.

        This is the term HIGH-2 reproduced.  It refused every bundle holding a
        single record above ~127 KiB in ``semantic_validation``, before either
        traversal, purely because of the multiplier.
        """
        return (
            self.owner_lifetime_retained_bound(count, aggregate, maximum)
            + self.REPRODUCED_RECORD_HEAP_MULTIPLIER * maximum
        )

    def cumulative_retained_work(self, ledger):
        """Sum every retained acquisition the request charged, over all phases."""
        return sum(
            values.get("retained_bytes", 0)
            for values in ledger.snapshot()["phases"].values()
        )

    def divergent_fixture(self, app, depth, prefix=None):
        """Bootstrap ``app`` and return ``(project_root, tip, records)``.

        ``prefix`` names the commit messages.  Fixtures that must stage
        byte-identical bundles across several projects pass the same prefix, so
        their aggregate byte bounds -- and therefore the validation
        reservation -- are identical.
        """
        prefix = app if prefix is None else prefix
        project_root = self.bootstrap(app)
        root_id, root = self.commit([], f"{prefix} divergent root")
        tip, chain = self.chain(root_id, depth, prefix)
        return project_root, tip, [root, *chain, *self.records]

    # ------------------------------------------------- zero-residual helpers

    # One repeated unit per record shape.  ``escape`` and ``control`` expand
    # under the canonical encoder (2 and 6 bytes per character), which is exactly
    # why the size curves are measured in canonical bytes rather than characters.
    RECORD_SHAPES = (
        ("ascii", "m"),
        ("cjk", "漢"),
        ("emoji", "\U0001f600"),
        ("escape", '"\\\n\t'),
        ("control", "\x01\x02"),
    )

    def canonical_size(self, value):
        return self.namespace["canonical_json_size"](value)

    def sized_commit(self, parents, label, target_bytes, unit="m"):
        """One commit whose message fills about ``target_bytes`` canonical bytes."""
        per_unit = self.canonical_size(unit) - 2
        repeat = max(1, target_bytes // per_unit)
        return self.commit(parents, f"{label}-" + unit * repeat)

    def fan_in(self, parent, width, prefix):
        """Return ``(tip, records)`` for one ``width``-parent merge commit."""
        records = []
        parents = []
        for index in range(width):
            child_id, child = self.commit([parent], f"{prefix}-leaf-{index}")
            parents.append(child_id)
            records.append(child)
        tip_id, tip = self.commit(parents, f"{prefix}-merge")
        records.append(tip)
        return tip_id, records

    def diamonds(self, parent, count, prefix):
        """Return ``(tip, records)`` for ``count`` stacked diamonds."""
        records = []
        for index in range(count):
            left_id, left = self.commit([parent], f"{prefix}-left-{index}")
            right_id, right = self.commit([parent], f"{prefix}-right-{index}")
            parent, merge = self.commit(
                [left_id, right_id], f"{prefix}-merge-{index}"
            )
            records.extend((left, right, merge))
        return parent, records

    def request_ledger_observer(self):
        """Count every ledger the product constructs while a request runs."""
        real = self.globals["DecisionWorkLedger"]
        minted = []

        class ObservedLedger(real):
            def __init__(inner, profile, command, command_kind="local_graph"):
                minted.append(command)
                super().__init__(profile, command, command_kind)

        self.globals["DecisionWorkLedger"] = ObservedLedger
        return minted, lambda: self.globals.__setitem__(
            "DecisionWorkLedger", real
        )

    def observe_session_owners(self, observations):
        """Record every session's ledger, owners and control-reservation proof.

        Each entry records the outstanding control reservation seen *before* the
        allocation the site is about to perform, which is what the
        charge-before-allocate rule requires.
        """
        session_type = self.globals["SealedCommitValidationSession"]
        real_new_fd = session_type._new_fd_owner
        real_subdir = session_type._subdir_fd
        # The probe must run against a build that has none of this repair's
        # structures, so every one of them is read defensively.  On such a build
        # each site reports ``outstanding = 0`` and the assertions fail on the
        # reservation, not on a missing attribute.
        accepts_covered = "covered" in real_new_fd.__code__.co_varnames

        class ObservedSession(session_type):
            def __init__(inner, objects_root, ledger, **kwargs):
                entry = {
                    "ledger": ledger,
                    "ledger_command": ledger.command,
                    "control_owner": kwargs.get("control_owner"),
                    "memory_owner": kwargs.get("memory_owner"),
                    "sites": [],
                }
                observations.append(entry)
                inner._observation = entry
                super().__init__(objects_root, ledger, **kwargs)

            def _record(inner, site, needed):
                outstanding = getattr(
                    inner, "control_reservation_outstanding", None
                )
                inner._observation["sites"].append(
                    {
                        "site": site,
                        "needed": needed,
                        "outstanding": (
                            0
                            if outstanding is None
                            else outstanding("retained_bytes")
                        ),
                        "fd_owners": len(inner._fd_owners),
                    }
                )

            def _new_fd_owner(inner, **kwargs):
                covered = kwargs.get("covered", True)
                pooled = bool(getattr(inner, "_fd_owner_pool", ()))
                if covered and not pooled:
                    inner._record(
                        "_fd_owners.append",
                        getattr(inner, "DESCRIPTOR_OWNER_BYTES", 1),
                    )
                if accepts_covered:
                    return real_new_fd(inner, **kwargs)
                return real_new_fd(inner)

            def _subdir_fd(inner, subdir):
                if subdir not in inner._subdir_fds:
                    inner._record(
                        "_subdir_fds insert",
                        getattr(inner, "SUBDIR_ENTRY_BYTES", 1)
                        + len(subdir.encode("utf-8")),
                    )
                return real_subdir(inner, subdir)

        self.globals["SealedCommitValidationSession"] = ObservedSession
        return lambda: self.globals.__setitem__(
            "SealedCommitValidationSession", session_type
        )

    def observe_root_traversals(self, traversals):
        """Record every root each session actually traverses."""
        session_type = self.globals["SealedCommitValidationSession"]
        real = session_type.validate_commit

        def observed(inner, commit_id, poll=None):
            traversals.append(commit_id)
            return real(inner, commit_id, poll)

        session_type.validate_commit = observed
        return lambda: setattr(session_type, "validate_commit", real)

    # ------------------------------------------------ review-closure helpers

    def ordered_projection_upper(self, documents):
        """The exact ``finalize`` ordered object-id projection upper.

        One list of ``documents`` pointers; ``_sort_strings_polling`` is an
        in-place heapsort and allocates nothing further.  Measured 56 bytes
        empty, 64 for one record, 272 for 27.
        """
        return (
            sys.getsizeof([])
            + self.namespace["_CANONICAL_POINTER_BYTES"] * documents
        )

    def write_local_commit(self, parents, message):
        """Write one commit straight into the local store and return its id."""
        object_id, record = self.commit(parents, message)
        (self.local_objects / "commits" / f"{object_id}.json").write_text(
            json.dumps(record, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        return object_id

    def observe_source_memory(self, owners):
        """Record every ``_AccountedUploadSourceMemory`` a route constructs."""
        real = self.globals["_AccountedUploadSourceMemory"]

        class Observed(real):
            def __init__(inner, ledger):
                owners.append(inner)
                super().__init__(ledger)

        self.globals["_AccountedUploadSourceMemory"] = Observed
        return lambda: self.globals.__setitem__(
            "_AccountedUploadSourceMemory", real
        )

    def completed_quarantine(self, app, key="quarantine-key"):
        """Return ``(project_root, tip, records, quarantine_path)``.

        The quarantine is built the way a real interrupted upload leaves one: a
        genuine upload publishes its authorities and then crashes in
        ``remove_remote_transaction_stage``, and the surviving pending marker is
        moved to its removing-quarantine name.
        """
        project_root = self.bootstrap(app)
        tip, child = self.commit([self.head], f"{app} quarantine child")
        real_remove = self.globals["remove_remote_transaction_stage"]

        def crash_before_cleanup(*_args):
            raise RuntimeError("simulated completed pending release gap")

        self.globals["remove_remote_transaction_stage"] = crash_before_cleanup
        crash_ledger = self.ledger(label=f"{app} crash")
        try:
            with self.assertRaises(BaseException):
                self.upload(
                    app, tip, [child], ledger=crash_ledger, key=key
                )
        finally:
            self.globals[
                "remove_remote_transaction_stage"
            ] = real_remove
        pending_path = project_root / "pending_transaction.json"
        self.assertTrue(pending_path.is_file())
        transaction_id = json.loads(
            pending_path.read_text(encoding="utf-8")
        )["transaction_id"]
        shutil.rmtree(
            project_root / "transactions" / transaction_id,
            ignore_errors=True,
        )
        quarantine = project_root / (
            f".pending_transaction.removing-{transaction_id}.json"
        )
        pending_path.rename(quarantine)
        return project_root, tip, [child], quarantine

    def recovery_snapshot(self, project_root, quarantine):
        """Digest every authority a completed-quarantine recovery can touch."""

        def digest(path):
            if not path.exists():
                return "absent"
            if path.is_file():
                return hashlib.sha256(path.read_bytes()).hexdigest()
            running = hashlib.sha256()
            for entry in sorted(path.rglob("*")):
                running.update(
                    str(entry.relative_to(path)).encode("utf-8")
                )
                if entry.is_file():
                    running.update(entry.read_bytes())
            return running.hexdigest()

        return {
            "objects": digest(project_root / "objects"),
            "branches": digest(project_root / "branches"),
            "generation": digest(project_root / "generation.json"),
            "nonce": digest(project_root / "request_nonce_cache.json"),
            "pending": digest(project_root / "pending_transaction.json"),
            "receipts": digest(project_root / "receipts"),
            "audit": digest(project_root / "audit.log"),
            "quarantine": digest(quarantine),
        }

    def legacy_object_loads(self, call):
        """Count unaccounted ``load_object_from_root`` reads during ``call``."""
        loads = []
        real = self.globals["load_object_from_root"]

        def counting(objects_root, object_id):
            loads.append(object_id)
            return real(objects_root, object_id)

        self.globals["load_object_from_root"] = counting
        try:
            return call(), loads
        finally:
            self.globals["load_object_from_root"] = real

    # ------------------------------------------------------------------ tests

    def test_exact_request_ledger_and_deadline_reach_both_traversals(self):
        project_root = self.bootstrap("identity")
        self.write_signature_policy(
            project_root,
            {"required": True, "require_history": True, "keys": {"alice": "s"}},
        )
        tip, records = self.signed_chain(self.head, 3, "identity", "s")
        clock = InjectedClock()
        deadline = self.deadline(clock)
        ledger = self.ledger(label="identity request")
        samples = []
        restore = self.observe_traversals(samples)
        try:
            status, result = self.upload(
                "identity", tip, records, ledger=ledger, deadline=deadline
            )
        finally:
            restore()
        self.assertEqual(status, 200)
        self.assertEqual(result, {"branch": "main", "head": tip})
        self.assertEqual([sample["phase"] for sample in samples], ["signature", "ancestry"])
        for sample in samples:
            with self.subTest(phase=sample["phase"]):
                # The exact request-owned objects, not equal copies.
                self.assertIs(sample["ledger"], ledger)
                self.assertIs(sample["deadline"], deadline)
                self.assertGreater(sample["polls"], 0)
                self.assertGreater(sample["graph_nodes"], 0)
                self.assertGreater(sample["record_reads"], 0)
        self.assertTrue(samples[1]["decision"])
        self.assertEqual(self.branch_head(project_root), tip)
        self.assert_closed(ledger)

    def test_immediate_parent_fast_forward_stops_goal_directed(self):
        project_root = self.bootstrap("fast-forward")
        tip, chain = self.chain(self.head, 300, "deep")
        chain_ledger = self.ledger(label="fast-forward chain")
        status, _ = self.upload(
            "fast-forward", tip, chain + self.records, ledger=chain_ledger
        )
        self.assertEqual(status, 200)
        self.assert_closed(chain_ledger)
        self.assertEqual(self.branch_head(project_root), tip)

        child_id, child = self.commit([tip], "immediate parent")
        legacy = {}
        accounted = self.globals["is_ancestor_from_objects_accounted"]
        samples = []

        def observed(objects_root, ancestor, commit_id, ledger, deadline):
            before = ledger.cumulative.get("graph_nodes", 0)
            reads_before = ledger.counters.get("record_reads", 0)
            decision = accounted(
                objects_root, ancestor, commit_id, ledger, deadline
            )
            samples.append(
                (
                    decision,
                    ledger.cumulative.get("graph_nodes", 0) - before,
                    ledger.counters.get("record_reads", 0) - reads_before,
                    ledger.cumulative.get("graph_edges", 0),
                )
            )
            # Compare against the reproduced defect on the same staged store.
            legacy["decision"], loads = self.legacy_object_loads(
                lambda: self.namespace["is_ancestor_from_objects"](
                    objects_root, ancestor, commit_id
                )
            )
            legacy["loads"] = len(loads)
            legacy["distances"] = len(
                self.namespace["ancestor_distances_from_objects"](
                    objects_root, commit_id
                )
            )
            return decision

        self.globals["is_ancestor_from_objects_accounted"] = observed
        ledger = self.ledger(label="fast-forward decision")
        try:
            status, _ = self.upload(
                "fast-forward", child_id, [child], ledger=ledger
            )
        finally:
            self.globals["is_ancestor_from_objects_accounted"] = accounted
        self.assertEqual(status, 200)
        self.assertEqual(len(samples), 1)
        decision, nodes, reads, _edges = samples[0]

        # Same decision as the distance-map predicate it replaces.
        self.assertTrue(decision)
        self.assertTrue(legacy["decision"])
        # The reproduced defect: the whole reachable history, uncharged.
        self.assertEqual(legacy["loads"], 302)
        self.assertEqual(legacy["distances"], 302)
        # Goal-directed: the head is read, its immediate parent is the sought
        # ancestor, and nothing beyond it is visited or charged.
        self.assertEqual(nodes, 2)
        self.assertEqual(reads, 1)
        self.assertLess(nodes, legacy["loads"])
        self.assertEqual(self.branch_head(project_root), child_id)
        self.assert_closed(ledger)

    def test_deep_divergent_history_traversal_is_iterative_and_charged(self):
        depth = 1_200
        project_root, tip, records = self.divergent_fixture("deep", depth)
        samples = []
        sealed = []
        restore = self.observe_traversals(samples)
        restore_sealed = self.observe_sealed_validation(sealed)
        # No override: a 1201-commit bundle reaches ancestry on the unmodified
        # http_v1 profile.  Sealed validation reserves the concurrent retained
        # bytes its owners really hold at one instant, so the reservation no
        # longer scales with every record's decode upper at once.
        ledger = self.ledger(label="deep divergent")
        try:
            status, payload = self.upload(
                "deep", tip, records, ledger=ledger
            )
        finally:
            restore_sealed()
            restore()
        self.assertEqual(status, 409)
        retained_limit = ledger.profile.limit("retained_bytes")
        self.assertEqual(retained_limit, 32 * 1024 * 1024)
        self.assertLess(ledger.peak.get("retained_bytes"), retained_limit)
        # The default profile is reached because of the owner-lifetime bound,
        # not because the fixture or the ceiling was made smaller.
        self.assertEqual(len(sealed), 1)
        bounds = (
            sealed[0]["record_count"],
            sealed[0]["aggregate_bytes"],
            sealed[0]["maximum_record_bytes"],
        )
        self.assertGreater(bounds[0], depth)
        self.assertEqual(
            sealed[0]["owner"]._coverage_retained_upper,
            self.owner_lifetime_retained_bound(*bounds),
        )
        self.assertGreater(
            self.reproduced_all_record_retained_bound(*bounds), retained_limit
        )
        self.assertEqual(
            payload,
            {
                "error": "upload rejected: remote branch is not an ancestor "
                "of local branch head"
            },
        )
        ancestry = [
            sample for sample in samples if sample["phase"] == "ancestry"
        ]
        self.assertEqual(len(ancestry), 1)
        self.assertFalse(ancestry[0]["decision"])
        # Iterative: every node of a 1201-commit divergent history is visited
        # and charged without exhausting the interpreter stack.
        self.assertEqual(ancestry[0]["graph_nodes"], depth + 1)
        self.assertEqual(ancestry[0]["record_reads"], depth + 1)
        self.assertEqual(ledger.maximum.get("logical_depth"), depth)
        # Publication is refused, so the branch authority is untouched.
        self.assertEqual(self.branch_head(project_root), self.head)
        self.assertIsNone(
            self.namespace["object_record_path"](
                project_root / "objects", tip
            )
        )
        self.assert_closed(ledger)

    def test_scale_curves_stay_within_the_owner_lifetime_bound(self):
        """1x/3x/10x work and memory curves under the unmodified profile.

        Measured on this fixture (depth 120/360/1200, http_v1 retained limit
        33,554,432)::

            factor  records  nodes  reads  depth  cumulative  concurrent  CFB-138
                                                  retained    peak        reserved
            1x          127    121    121    120   1,600,708   1,025,102   13,416,088
            3x          367    361    361    360   4,292,260   2,646,062   39,501,208
            10x       1,207  1,201  1,201  1,200  13,743,133   8,322,480  130,978,456

        Graph work is exactly the traversed history at every scale.  The
        concurrent retained peak is the sealed-validation reservation, whose
        per-record slope is a fixed control allowance plus two canonical bytes
        -- not the whole record's decode upper -- so 10x the history costs ~8x
        the bytes and stays at a quarter of the default ceiling.  The last
        column is the reproduced all-records-live reservation, which passes the
        ceiling between 1x and 3x; the assertions below pin both curves.
        """
        base_depth = 120
        retained_limit = None
        curves = []
        for factor in (1, 3, 10):
            with self.subTest(factor=factor):
                depth = base_depth * factor
                app = f"scale-{factor}x"
                project_root, tip, records = self.divergent_fixture(app, depth)
                samples = []
                sealed = []
                restore = self.observe_traversals(samples)
                restore_sealed = self.observe_sealed_validation(sealed)
                ledger = self.ledger(label=f"scale {factor}x")
                try:
                    status, payload = self.upload(
                        app, tip, records, ledger=ledger
                    )
                finally:
                    restore_sealed()
                    restore()
                self.assertEqual(status, 409)
                self.assertEqual(
                    payload,
                    {
                        "error": "upload rejected: remote branch is not an "
                        "ancestor of local branch head"
                    },
                )
                retained_limit = ledger.profile.limit("retained_bytes")
                ancestry = [
                    sample
                    for sample in samples
                    if sample["phase"] == "ancestry"
                ]
                self.assertEqual(len(ancestry), 1)
                self.assertFalse(ancestry[0]["decision"])
                self.assertEqual(len(sealed), 1)
                owner = sealed[0]["owner"]
                bounds = (
                    sealed[0]["record_count"],
                    sealed[0]["aggregate_bytes"],
                    sealed[0]["maximum_record_bytes"],
                )
                bound = owner._coverage_retained_upper
                measured_peak = owner._claimed_memory_peak["retained_bytes"]
                concurrent_peak = ledger.peak.get("retained_bytes")

                # The reservation is exactly the documented owner-lifetime
                # bound, and it is the request's concurrent high-water: the
                # boundary tests below therefore land on this owner.
                self.assertEqual(
                    bound, self.owner_lifetime_retained_bound(*bounds)
                )
                self.assertEqual(concurrent_peak, bound)
                self.assertLess(bound, retained_limit)
                # The bound really covers what the streamed owners hold.
                self.assertGreater(measured_peak, 0)
                self.assertLessEqual(measured_peak, bound)
                curves.append(
                    {
                        "factor": factor,
                        "depth": depth,
                        "records": bounds[0],
                        "bounds": bounds,
                        "graph_nodes": ancestry[0]["graph_nodes"],
                        "record_reads": ancestry[0]["record_reads"],
                        "logical_depth": ledger.maximum.get("logical_depth"),
                        "cumulative_retained": self.cumulative_retained_work(
                            ledger
                        ),
                        "reservation": bound,
                        "concurrent_peak": concurrent_peak,
                        "measured_peak": measured_peak,
                        "reproduced": self.reproduced_all_record_retained_bound(
                            *bounds
                        ),
                    }
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assertIsNone(
                    self.namespace["object_record_path"](
                        project_root / "objects", tip
                    )
                )
                self.assert_closed(ledger)

        self.assertEqual([curve["factor"] for curve in curves], [1, 3, 10])
        # Every fixture stages the same non-chain remainder (the divergent root
        # plus the bootstrap closure), so the staged record count is the depth
        # plus that fixed remainder at every scale.
        remainder = curves[0]["records"] - curves[0]["depth"]
        self.assertLessEqual(remainder, 1 + len(self.records))
        for curve in curves:
            with self.subTest(factor=curve["factor"], curve="graph"):
                # Graph work is exactly the traversed history, at every scale.
                self.assertEqual(curve["graph_nodes"], curve["depth"] + 1)
                self.assertEqual(curve["record_reads"], curve["depth"] + 1)
                self.assertEqual(curve["logical_depth"], curve["depth"])
                self.assertEqual(curve["records"], curve["depth"] + remainder)

        # Memory curves: the derived bound's slack over what the owners really
        # hold does not grow with scale, and neither does the cumulative
        # retained work charged per record.
        for previous, current in zip(curves, curves[1:]):
            with self.subTest(factor=current["factor"], curve="memory"):
                self.assertGreater(
                    current["concurrent_peak"], previous["concurrent_peak"]
                )
                self.assertGreater(
                    current["reservation"], previous["reservation"]
                )
                self.assertLessEqual(
                    current["concurrent_peak"] * previous["measured_peak"],
                    previous["concurrent_peak"] * current["measured_peak"],
                )
                self.assertLessEqual(
                    current["cumulative_retained"] * previous["records"],
                    previous["cumulative_retained"] * current["records"],
                )
        for curve in curves:
            with self.subTest(factor=curve["factor"], curve="slack"):
                self.assertLessEqual(
                    curve["concurrent_peak"], 10 * curve["measured_peak"]
                )
                # No other owner's live claim ever exceeds the reservation the
                # sealed owner holds and releases at its final fence.
                self.assertLessEqual(
                    curve["concurrent_peak"], curve["reservation"]
                )

        # The bound is linear in the staged aggregate with slope two -- the
        # documents once, the live frames' object ids once -- because the memo
        # holds each staged record exactly once.  The reproduced reservation
        # multiplied that same aggregate by the whole per-record decode upper,
        # which is what broke the default profile at 3x and beyond.
        multiplier = self.REPRODUCED_RECORD_HEAP_MULTIPLIER
        count, aggregate, maximum = curves[-1]["bounds"]
        self.assertEqual(
            self.owner_lifetime_retained_bound(count, aggregate + 4096, maximum)
            - self.owner_lifetime_retained_bound(count, aggregate, maximum),
            2 * 4096,
        )
        self.assertGreater(multiplier, 2)
        self.assertEqual(
            self.reproduced_all_record_retained_bound(
                count, aggregate + 4096, maximum
            )
            - self.reproduced_all_record_retained_bound(
                count, aggregate, maximum
            ),
            multiplier * 4096,
        )
        self.assertLess(curves[0]["reproduced"], retained_limit)
        for curve in curves[1:]:
            with self.subTest(factor=curve["factor"], curve="reversal"):
                self.assertGreater(curve["reproduced"], retained_limit)

    def test_retained_boundaries_reach_the_streaming_validation_owner(self):
        depth = 120
        _root, tip, records = self.divergent_fixture(
            "retained-observe", depth, prefix="retained"
        )
        samples = []
        sealed = []
        restore = self.observe_traversals(samples)
        restore_sealed = self.observe_sealed_validation(sealed)
        observation = self.ledger(label="retained observation")
        try:
            status, _payload = self.upload(
                "retained-observe", tip, records, ledger=observation
            )
        finally:
            restore_sealed()
            restore()
        self.assertEqual(status, 409)
        self.assert_closed(observation)
        exact = observation.peak.get("retained_bytes")
        self.assertEqual(len(sealed), 1)
        # The sealed-validation reservation owns the request's retained
        # high-water, so the boundaries below are decided by the corrected
        # owner and nothing earlier.
        reservation = sealed[0]["owner"]._coverage_retained_upper
        self.assertEqual(reservation, exact)
        self.assertEqual(sealed[0]["retained_before"], 0)
        self.assertLess(exact, observation.profile.limit("retained_bytes"))

        for label, limit in (("exact", exact), ("plus-one", exact + 1)):
            with self.subTest(boundary=label):
                app = f"retained-{label}"
                project_root, tip, records = self.divergent_fixture(
                    app, depth, prefix="retained"
                )
                samples = []
                restore = self.observe_traversals(samples)
                ledger = self.ledger(
                    {"retained_bytes": ("concurrent", limit)},
                    f"retained {label}",
                )
                try:
                    status, payload = self.upload(
                        app, tip, records, ledger=ledger
                    )
                finally:
                    restore()
                # No slack at all, and one spare byte, both reach ancestry.
                self.assertEqual(status, 409)
                self.assertEqual(
                    payload,
                    {
                        "error": "upload rejected: remote branch is not an "
                        "ancestor of local branch head"
                    },
                )
                self.assertEqual(ledger.peak.get("retained_bytes"), exact)
                ancestry = [
                    sample
                    for sample in samples
                    if sample["phase"] == "ancestry"
                ]
                self.assertEqual(len(ancestry), 1)
                self.assertFalse(ancestry[0]["decision"])
                self.assertEqual(ancestry[0]["graph_nodes"], depth + 1)
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assert_closed(ledger)

        app = "retained-minus-one"
        project_root, tip, records = self.divergent_fixture(
            app, depth, prefix="retained"
        )
        before = self.authority_snapshot(project_root)
        samples = []
        sealed = []
        allocations = []
        restore = self.observe_traversals(samples)
        restore_sealed = self.observe_sealed_validation(sealed)
        restore_session = self.observe_session_allocations(allocations)
        ledger = self.ledger(
            {"retained_bytes": ("concurrent", reservation - 1)},
            "retained minus one",
        )
        try:
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ) as caught:
                self.upload(app, tip, records, ledger=ledger)
        finally:
            restore_session()
            restore_sealed()
            restore()
        diagnostic = caught.exception.diagnostic()
        self.assertEqual(diagnostic["kind"], "resource_budget_exceeded")
        self.assertEqual(diagnostic["dimension"], "retained_bytes")
        self.assertEqual(diagnostic["phase"], "semantic_validation")
        self.assertEqual(diagnostic["limit"], reservation - 1)
        self.assertEqual(diagnostic["observed"], reservation)
        self.assertEqual(diagnostic["command"], "retained minus one")
        self.assertEqual(
            diagnostic["exit_code"],
            self.namespace["RESOURCE_BUDGET_EXIT_CODE"],
        )
        self.assertEqual(ledger.terminal_reason, "budget_exceeded")
        # The refused acquisition is the corrected owner's own reservation: it
        # was entered, and it precedes both its session allocator and every
        # later traversal.
        self.assertEqual(len(sealed), 1)
        self.assertIsNone(sealed[0]["owner"])
        self.assertEqual(sealed[0]["retained_before"], 0)
        self.assertEqual(allocations, [])
        self.assertEqual(samples, [])
        # Pre-effect: nothing published, nothing staged, no retained debt.
        self.assertEqual(self.authority_snapshot(project_root), before)
        self.assertEqual(self.branch_head(project_root), self.head)
        self.assertIsNone(
            self.namespace["object_record_path"](
                project_root / "objects", tip
            )
        )
        self.assert_closed(ledger)

    def test_reproduced_all_record_reservation_fails_the_default_profile(self):
        """The pre-fix target cannot reach ancestry on the default profile."""
        depth = 1_200
        app = "reproduced"
        project_root, tip, records = self.divergent_fixture(app, depth)
        before = self.authority_snapshot(project_root)
        sealed_type = self.namespace["AccountedHTTPUploadSealedValidation"]
        fixed = sealed_type.__dict__["allocation_upper"]
        corrected = []

        def reproduced_allocation_upper(count, aggregate, maximum):
            retained, temporary = fixed.__func__(
                sealed_type, count, aggregate, maximum
            )
            corrected.append(retained)
            return (
                self.reproduced_all_record_retained_bound(
                    count, aggregate, maximum
                ),
                temporary,
            )

        samples = []
        sealed = []
        allocations = []
        restore = self.observe_traversals(samples)
        restore_sealed = self.observe_sealed_validation(sealed)
        restore_session = self.observe_session_allocations(allocations)
        sealed_type.allocation_upper = staticmethod(reproduced_allocation_upper)
        ledger = self.ledger(label="reproduced reservation")
        try:
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ) as caught:
                self.upload(app, tip, records, ledger=ledger)
        finally:
            sealed_type.allocation_upper = fixed
            restore_session()
            restore_sealed()
            restore()
        self.assertEqual(len(sealed), 1)
        self.assertIsNone(sealed[0]["owner"])
        bounds = (
            sealed[0]["record_count"],
            sealed[0]["aggregate_bytes"],
            sealed[0]["maximum_record_bytes"],
        )
        reproduced = self.reproduced_all_record_retained_bound(*bounds)
        retained_limit = ledger.profile.limit("retained_bytes")
        diagnostic = caught.exception.diagnostic()
        self.assertEqual(diagnostic["kind"], "resource_budget_exceeded")
        self.assertEqual(diagnostic["dimension"], "retained_bytes")
        self.assertEqual(diagnostic["phase"], "semantic_validation")
        self.assertEqual(diagnostic["limit"], retained_limit)
        self.assertEqual(diagnostic["observed"], reproduced)
        self.assertGreater(reproduced, retained_limit)
        # Exactly the CFB-138 shape: the refused reservation is far above the
        # concurrent retained bytes the request had actually reached.
        self.assertLess(ledger.peak.get("retained_bytes"), retained_limit)
        self.assertGreater(ledger.peak.get("retained_bytes"), 0)
        self.assertLess(
            ledger.peak.get("retained_bytes"), reproduced // 10
        )
        # The corrected bound for the very same staged fixture fits, so the
        # reversal is the reservation model and nothing about the fixture.
        self.assertEqual(corrected, [self.owner_lifetime_retained_bound(*bounds)])
        self.assertLess(corrected[0], retained_limit)
        # Refused before the session allocator, both traversals, and any
        # publication: no partial authority survives.
        self.assertEqual(allocations, [])
        self.assertEqual(samples, [])
        self.assertEqual(ledger.terminal_reason, "budget_exceeded")
        self.assertEqual(self.authority_snapshot(project_root), before)
        self.assertEqual(self.branch_head(project_root), self.head)
        self.assertIsNone(
            self.namespace["object_record_path"](
                project_root / "objects", tip
            )
        )
        self.assert_closed(ledger)

    def test_shared_dag_traversal_is_deterministic_and_charged_once(self):
        root_id, root = self.commit([], "shared divergent root")
        first_id, first = self.commit([root_id], "shared first")
        left_id, left = self.commit([first_id], "shared left")
        right_id, right = self.commit([first_id], "shared right")
        merge_id, merge = self.commit([left_id, right_id], "shared merge")
        records = [root, first, left, right, merge, *self.records]
        expected_order = [merge_id, left_id, right_id, first_id, root_id]

        runs = []
        for app in ("shared-first", "shared-second"):
            project_root = self.bootstrap(app)
            source = self.globals["_AccountedUploadCommitSource"]
            payload = source.payload
            loads = []

            def observed_payload(inner_self, object_id, _payload=payload):
                loads.append(object_id)
                return _payload(inner_self, object_id)

            samples = []
            restore = self.observe_traversals(samples)
            source.payload = observed_payload
            ledger = self.ledger(label=f"{app} shared dag")
            try:
                status, _ = self.upload(
                    app, merge_id, records, ledger=ledger
                )
            finally:
                source.payload = payload
                restore()
            self.assertEqual(status, 409)
            ancestry = [
                sample for sample in samples if sample["phase"] == "ancestry"
            ]
            self.assertEqual(len(ancestry), 1)
            runs.append(
                (
                    loads,
                    ancestry[0]["graph_nodes"],
                    ancestry[0]["record_reads"],
                    ancestry[0]["revisits"],
                    ledger.cumulative.get("graph_edges"),
                    ancestry[0]["decision"],
                )
            )
            self.assertEqual(self.branch_head(project_root), self.head)
            self.assert_closed(ledger)

        # Deterministic breadth-first order, one owner contract, one read per
        # unique node, and the shared node's second frame charged as a revisit.
        self.assertEqual(runs[0], runs[1])
        loads, nodes, reads, revisits, _edges, decision = runs[0]
        self.assertEqual(loads, expected_order)
        self.assertEqual(nodes, 5)
        self.assertEqual(reads, 5)
        self.assertEqual(revisits, 1)
        self.assertFalse(decision)

    def test_exact_and_exact_minus_one_budgets_precede_mutation(self):
        policy = {
            "required": True,
            "require_history": True,
            "keys": {"alice": "s"},
        }

        def fixture(app):
            project_root = self.bootstrap(app)
            self.write_signature_policy(project_root, policy)
            tip, records = self.signed_chain(self.head, 3, "budget", "s")
            return project_root, tip, records

        project_root, tip, records = fixture("budget-observe")
        samples = []
        restore = self.observe_traversals(samples)
        observation = self.ledger(label="budget observation")
        try:
            status, _ = self.upload(
                "budget-observe", tip, records, ledger=observation
            )
        finally:
            restore()
        self.assertEqual(status, 200)
        self.assert_closed(observation)
        signature, ancestry = samples
        signature_exit = signature["graph_nodes_after"]
        total = ancestry["graph_nodes_after"]
        self.assertEqual(total, observation.cumulative.get("graph_nodes"))
        self.assertGreater(signature["graph_nodes"], 0)
        self.assertGreater(ancestry["graph_nodes"], 0)

        # Exact: the request completes on a budget with no slack at all.
        project_root, tip, records = fixture("budget-exact")
        exact = self.ledger(
            {"graph_nodes": ("cumulative", total)}, "budget exact"
        )
        status, _ = self.upload("budget-exact", tip, records, ledger=exact)
        self.assertEqual(status, 200)
        self.assertEqual(exact.cumulative.get("graph_nodes"), total)
        self.assertEqual(self.branch_head(project_root), tip)
        self.assert_closed(exact)

        for label, limit, failing_phase, entered in (
            ("ancestry", total - 1, "ancestry", ["signature", "ancestry"]),
            ("signature", signature_exit - 1, "signature", ["signature"]),
        ):
            with self.subTest(phase=label):
                app = f"budget-minus-one-{label}"
                project_root, tip, records = fixture(app)
                before = self.authority_snapshot(project_root)
                samples = []
                restore = self.observe_traversals(samples)
                ledger = self.ledger(
                    {"graph_nodes": ("cumulative", limit)},
                    f"budget minus one {label}",
                )
                try:
                    with self.assertRaises(
                        self.namespace["ResourceBudgetExceeded"]
                    ) as caught:
                        self.upload(app, tip, records, ledger=ledger)
                finally:
                    restore()
                diagnostic = caught.exception.diagnostic()
                self.assertEqual(
                    set(diagnostic),
                    {
                        "kind",
                        "exit_code",
                        "profile",
                        "dimension",
                        "observed",
                        "limit",
                        "phase",
                        "command",
                        "continuation_supported",
                    },
                )
                self.assertEqual(diagnostic["kind"], "resource_budget_exceeded")
                self.assertEqual(diagnostic["dimension"], "graph_nodes")
                self.assertEqual(diagnostic["limit"], limit)
                self.assertEqual(diagnostic["observed"], limit + 1)
                self.assertEqual(diagnostic["phase"], "graph_discovery")
                self.assertEqual(
                    diagnostic["command"], f"budget minus one {label}"
                )
                self.assertFalse(diagnostic["continuation_supported"])
                self.assertEqual(
                    diagnostic["exit_code"],
                    self.namespace["RESOURCE_BUDGET_EXIT_CODE"],
                )
                # The typed outcome came from the intended traversal and no
                # later phase ran.
                self.assertEqual(
                    [sample["phase"] for sample in samples], entered
                )
                self.assertEqual(samples[-1]["phase"], failing_phase)
                self.assertEqual(ledger.terminal_reason, "budget_exceeded")
                # No mutation, no staged object, no retained/permit debt.
                self.assertEqual(
                    self.authority_snapshot(project_root), before
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assertIsNone(
                    self.namespace["object_record_path"](
                        project_root / "objects", tip
                    )
                )
                self.assert_closed(ledger)

    def test_ancestry_collection_budget_precedes_its_allocators(self):
        project_root = self.bootstrap("collection")
        child_id, child = self.commit([self.head], "collection child")
        samples = []
        sealed = []
        restore = self.observe_traversals(samples)
        restore_sealed = self.observe_sealed_validation(sealed)
        observation = self.ledger(label="collection observation")
        try:
            status, _ = self.upload(
                "collection", child_id, [child], ledger=observation
            )
        finally:
            restore_sealed()
            restore()
        self.assertEqual(status, 200)
        self.assert_closed(observation)
        # Ancestry is entered with the request's retained ownership fully
        # released: the sealed-validation owner's final revalidation fence ran
        # before it, so the collection reservation is the only live claim at that
        # instant.  Earlier phases -- object staging and the sealed validation
        # record reads -- do peak above it and release, so a budget pinned at the
        # entry level alone would be refused before ancestry is ever reached.
        # Pin the ceiling just above that earlier high-water instead, and size the
        # collection upper to overrun it by one byte.
        entry_retained = samples[-1]["retained_before"]
        entry_peak = samples[-1]["retained_peak_before"]
        self.assertEqual(len(sealed), 1)
        self.assertEqual(entry_retained, 0)
        self.assertGreater(entry_peak, entry_retained)
        limit = entry_peak + 4096

        owner = self.namespace["AccountedHTTPUploadAncestry"]
        base_bytes = owner.COLLECTION_BASE_BYTES
        # The real frontier/seen upper is ~1 KiB, far below the ceiling every
        # earlier phase needs.  Inflating only the declared upper makes this
        # one reservation the first refused acquisition; the ordering property
        # under test -- charge, then allocate -- is independent of its size.
        inflated_base = limit - entry_retained + 1 - owner.TRANSITION_BYTES
        self.assertGreater(inflated_base, base_bytes)
        allocate_seen = owner._allocate_seen
        allocate_frontier = owner._allocate_frontier
        allocations = []

        def observed_seen():
            allocations.append("seen")
            return allocate_seen()

        def observed_frontier():
            allocations.append("frontier")
            return allocate_frontier()

        app = "collection-minus-one"
        project_root = self.bootstrap(app)
        child_id, child = self.commit([self.head], "collection child")
        before = self.authority_snapshot(project_root)
        samples = []
        restore = self.observe_traversals(samples)
        owner._allocate_seen = staticmethod(observed_seen)
        owner._allocate_frontier = staticmethod(observed_frontier)
        owner.COLLECTION_BASE_BYTES = inflated_base
        ledger = self.ledger(
            {"retained_bytes": ("concurrent", limit)},
            "collection minus one",
        )
        try:
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ) as caught:
                self.upload(app, child_id, [child], ledger=ledger)
        finally:
            owner.COLLECTION_BASE_BYTES = base_bytes
            owner._allocate_seen = staticmethod(allocate_seen)
            owner._allocate_frontier = staticmethod(allocate_frontier)
            restore()
        # The refusal came from the ancestry traversal's own collection
        # reservation, one byte over, not from any earlier phase.
        self.assertEqual([sample["phase"] for sample in samples], ["signature", "ancestry"])
        self.assertEqual(samples[-1]["phase"], "ancestry")
        self.assertEqual(samples[-1]["retained_before"], entry_retained)
        diagnostic = caught.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "retained_bytes")
        self.assertEqual(diagnostic["phase"], "graph_discovery")
        self.assertEqual(diagnostic["limit"], limit)
        self.assertEqual(diagnostic["observed"], limit + 1)
        self.assertEqual(diagnostic["command"], "collection minus one")
        self.assertEqual(ledger.terminal_reason, "budget_exceeded")
        # The frontier/seen retained upper is charged before either collection
        # exists, so no traversal state was ever allocated.
        self.assertEqual(allocations, [])
        self.assertEqual(self.authority_snapshot(project_root), before)
        self.assertEqual(self.branch_head(project_root), self.head)
        self.assertIsNone(
            self.namespace["object_record_path"](
                project_root / "objects", child_id
            )
        )
        self.assert_closed(ledger)

    def test_deadline_expiry_in_both_phases_precedes_mutation(self):
        policy = {
            "required": True,
            "require_history": True,
            "keys": {"alice": "s"},
        }
        signature_policy = self.globals["AccountedCommitSignaturePolicy"]
        verify = signature_policy.verify

        for label, phase in (
            ("signature", "commit signature history"),
            ("ancestry", "upload ancestry"),
        ):
            with self.subTest(phase=label):
                app = f"deadline-{label}"
                project_root = self.bootstrap(app)
                self.write_signature_policy(project_root, policy)
                tip, records = self.signed_chain(self.head, 3, "deadline", "s")
                before = self.authority_snapshot(project_root)
                clock = InjectedClock()
                deadline = self.deadline(clock)
                ledger = self.ledger(label=f"deadline {label}")
                samples = []
                restore = self.observe_traversals(samples)

                def expire_then_verify(inner_self, commit_id, is_head=True):
                    clock.expire()
                    return verify(inner_self, commit_id, is_head)

                accounted = self.globals["is_ancestor_from_objects_accounted"]

                def expire_then_decide(
                    objects_root, ancestor, commit_id, active, active_deadline
                ):
                    clock.expire()
                    return accounted(
                        objects_root,
                        ancestor,
                        commit_id,
                        active,
                        active_deadline,
                    )

                if label == "signature":
                    signature_policy.verify = expire_then_verify
                else:
                    self.globals[
                        "is_ancestor_from_objects_accounted"
                    ] = expire_then_decide
                try:
                    with self.assertRaises(
                        self.namespace["RequestDeadlineExceeded"]
                    ) as caught:
                        self.upload(
                            app,
                            tip,
                            records,
                            ledger=ledger,
                            deadline=deadline,
                        )
                finally:
                    signature_policy.verify = verify
                    self.globals[
                        "is_ancestor_from_objects_accounted"
                    ] = accounted
                    restore()
                # Deterministic phase attribution on the exact request deadline.
                self.assertEqual(caught.exception.phase, phase)
                self.assertEqual(
                    caught.exception.limit_ms, self.DEADLINE_LIMIT_MS
                )
                self.assertEqual(deadline.expired_phase, phase)
                self.assertEqual(samples[-1]["phase"], label)
                self.assertGreater(samples[-1]["polls"], 0)
                # Nothing published, and all ownership released.
                self.assertEqual(
                    self.authority_snapshot(project_root), before
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assertIsNone(
                    self.namespace["object_record_path"](
                        project_root / "objects", tip
                    )
                )
                self.assert_closed(ledger)

    def test_signature_policy_decisions_and_precedence_are_preserved(self):
        key = "signing-secret"
        signed_tip, signed_records = self.signed_chain(
            self.head, 2, "policy", key
        )
        unsigned_tip, unsigned_records = self.chain(self.head, 1, "unsigned")

        cases = (
            ("absent", None, signed_tip, signed_records, None),
            (
                "not-required",
                {"required": False},
                unsigned_tip,
                unsigned_records,
                None,
            ),
            (
                "required-signed",
                {"required": True, "keys": {"alice": key}},
                signed_tip,
                signed_records,
                None,
            ),
            (
                "required-history",
                {
                    "required": True,
                    "require_history": True,
                    "keys": {"alice": key},
                },
                signed_tip,
                signed_records,
                None,
            ),
            (
                "required-unsigned",
                {"required": True, "keys": {"alice": key}},
                unsigned_tip,
                unsigned_records,
                "signature is required",
            ),
            (
                "unknown-signer",
                {"required": True, "keys": {"bob": key}},
                signed_tip,
                signed_records,
                "no key for signer 'alice'",
            ),
            (
                "keys-not-an-object",
                {"required": True, "keys": []},
                signed_tip,
                signed_records,
                "commit_signatures.keys must be an object",
            ),
        )
        for label, policy, tip, records, message in cases:
            with self.subTest(policy=label):
                app = f"policy-{label}"
                project_root = self.bootstrap(app)
                if policy is not None:
                    self.write_signature_policy(project_root, policy)
                before = self.authority_snapshot(project_root)
                ledger = self.ledger(label=f"policy {label}")
                if message is None:
                    status, _ = self.upload(
                        app, tip, records, ledger=ledger
                    )
                    self.assertEqual(status, 200)
                    self.assertEqual(self.branch_head(project_root), tip)
                else:
                    with self.assertRaisesRegex(
                        self.namespace["CFError"], message
                    ):
                        self.upload(app, tip, records, ledger=ledger)
                    self.assertEqual(
                        self.authority_snapshot(project_root), before
                    )
                    self.assertEqual(
                        self.branch_head(project_root), self.head
                    )
                self.assert_closed(ledger)

    def test_accounted_signature_policy_matches_the_legacy_first_error(self):
        """Diagnostic precedence is identical on a shared staged fixture."""
        project_root = self.tmp / "precedence/.codefire-server/projects/org/app"
        objects_root = project_root / "objects"
        project_root.mkdir(parents=True)
        self.write_signature_policy(
            project_root,
            {
                "required": True,
                "require_history": True,
                "keys": {"alice": "signing-secret"},
            },
        )

        def write(record):
            subdir, file_name = self.namespace["exact_object_record_location"](
                record["object_id"]
            )
            path = objects_root / subdir / file_name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(
                json.dumps(record, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            return record["object_id"]

        root_id, root = self.commit([], "precedence root", signer="alice", key="signing-secret")
        write(root)
        invalid_payload = json.loads(json.dumps(root["payload"]))
        invalid_payload["message"] = "precedence invalid"
        invalid = self.namespace["sign_commit_payload"](
            invalid_payload, "alice", "signing-secret", "alice"
        )
        invalid["signature"]["signature"] = "0" * 64
        invalid_id, invalid_record = self.commit_record(invalid)
        write(invalid_record)
        missing_id = "CF-COMMIT-" + "0" * 24

        for label, parents, message in (
            ("invalid-first", [invalid_id, missing_id], "invalid signature"),
            ("missing-first", [missing_id, invalid_id], "object not found"),
        ):
            with self.subTest(order=label):
                head_id, head_record = self.commit(
                    parents, f"precedence {label}", signer="alice", key="signing-secret"
                )
                write(head_record)
                legacy = None
                try:
                    self.namespace[
                        "enforce_remote_commit_signature_policy"
                    ](project_root, objects_root, head_id)
                except self.namespace["CFError"] as error:
                    legacy = str(error)
                ledger = self.ledger(label=f"precedence {label}")
                accounted = None
                try:
                    self.namespace[
                        "enforce_remote_commit_signature_policy_accounted"
                    ](project_root, objects_root, head_id, ledger, None)
                except self.namespace["CFError"] as error:
                    accounted = str(error)
                self.assertIsNotNone(legacy)
                self.assertIn(message, legacy)
                self.assertEqual(accounted, legacy)
                self.assert_closed(ledger)

    def test_new_live_path_replaces_the_reproduced_uncharged_traversals(self):
        project_root = self.bootstrap("live")
        tip, chain = self.chain(self.head, 300, "live")
        chain_ledger = self.ledger(label="live chain")
        status, _ = self.upload(
            "live", tip, chain + self.records, ledger=chain_ledger
        )
        self.assertEqual(status, 200)
        self.assert_closed(chain_ledger)

        # The live route no longer reaches either unaccounted function.
        def forbidden_ancestry(*_args, **_kwargs):
            raise AssertionError("unaccounted ancestry reached the live route")

        def forbidden_signature(*_args, **_kwargs):
            raise AssertionError("unaccounted signature reached the live route")

        ancestor_distances = self.globals["ancestor_distances_from_objects"]
        legacy_signature = self.globals[
            "enforce_remote_commit_signature_policy"
        ]
        owner_calls = []
        ancestry_owner = self.globals["AccountedHTTPUploadAncestry"]
        contains = ancestry_owner.contains
        borrow = self.globals["borrowed_sealed_commit_signature_payload"]

        def observed_contains(inner_self, ancestor, head, **kwargs):
            owner_calls.append(("contains", kwargs))
            return contains(inner_self, ancestor, head, **kwargs)

        def observed_borrow(session, object_id, deadline=None):
            owner_calls.append(("borrow", object_id))
            return borrow(session, object_id, deadline)

        first_id, first = self.commit([tip], "live child")
        self.globals["ancestor_distances_from_objects"] = forbidden_ancestry
        self.globals[
            "enforce_remote_commit_signature_policy"
        ] = forbidden_signature
        ancestry_owner.contains = observed_contains
        self.globals[
            "borrowed_sealed_commit_signature_payload"
        ] = observed_borrow
        self.write_signature_policy(
            project_root, {"required": True, "keys": {"alice": "s"}}
        )
        signed_id, signed = self.commit(
            [tip], "live signed child", signer="alice", key="s"
        )
        ledger = self.ledger(label="live route")
        try:
            status, _ = self.upload(
                "live", signed_id, [signed], ledger=ledger
            )
        finally:
            self.globals[
                "ancestor_distances_from_objects"
            ] = ancestor_distances
            self.globals[
                "enforce_remote_commit_signature_policy"
            ] = legacy_signature
            ancestry_owner.contains = contains
            self.globals[
                "borrowed_sealed_commit_signature_payload"
            ] = borrow
        self.assertEqual(status, 200)
        # The formerly dead owners are the live route: a goal-directed ancestry
        # decision and a borrowed, charged signature payload.
        self.assertIn(("contains", {"goal_directed": True}), owner_calls)
        self.assertIn(("borrow", signed_id), owner_calls)
        self.assert_closed(ledger)
        self.assertEqual(self.branch_head(project_root), signed_id)
        self.assertFalse(first_id == signed_id)

        # Restoring the reproduced production wiring restores both defects.
        (project_root / "server_policy.json").unlink()
        accounted_ancestry = self.globals["is_ancestor_from_objects_accounted"]
        accounted_signature = self.globals[
            "enforce_remote_commit_signature_policy_accounted"
        ]
        observations = []

        def old_ancestry(objects_root, ancestor, commit_id, ledger, deadline):
            before = (
                ledger.cumulative.get("graph_nodes", 0),
                ledger.counters.get("record_reads", 0),
                ledger.counters.get("files_read", 0),
                deadline.poll_count,
            )
            decision, loads = self.legacy_object_loads(
                lambda: self.namespace["is_ancestor_from_objects"](
                    objects_root, ancestor, commit_id
                )
            )
            after = (
                ledger.cumulative.get("graph_nodes", 0),
                ledger.counters.get("record_reads", 0),
                ledger.counters.get("files_read", 0),
                deadline.poll_count,
            )
            observations.append(("ancestry", before, after, len(loads)))
            return decision

        def old_signature(project_root_arg, objects_root, commit_id, *rest):
            ledger, deadline = rest
            before = (
                ledger.cumulative.get("graph_nodes", 0),
                ledger.counters.get("record_reads", 0),
                deadline.poll_count,
            )
            self.namespace["enforce_remote_commit_signature_policy"](
                project_root_arg, objects_root, commit_id
            )
            after = (
                ledger.cumulative.get("graph_nodes", 0),
                ledger.counters.get("record_reads", 0),
                deadline.poll_count,
            )
            observations.append(("signature", before, after, None))

        child_id, child = self.commit([signed_id], "old wiring child")
        clock = InjectedClock()
        deadline = self.deadline(clock)
        old_ledger = self.ledger(label="old wiring")
        self.globals["is_ancestor_from_objects_accounted"] = old_ancestry
        self.globals[
            "enforce_remote_commit_signature_policy_accounted"
        ] = old_signature
        try:
            status, _ = self.upload(
                "live",
                child_id,
                [child],
                ledger=old_ledger,
                deadline=deadline,
            )
        finally:
            self.globals[
                "is_ancestor_from_objects_accounted"
            ] = accounted_ancestry
            self.globals[
                "enforce_remote_commit_signature_policy_accounted"
            ] = accounted_signature
        self.assertEqual(status, 200)
        self.assertEqual(
            [entry[0] for entry in observations], ["signature", "ancestry"]
        )
        for phase, before, after, loads in observations:
            with self.subTest(phase=phase, wiring="reproduced"):
                # Neither the ledger nor the deadline observes any of the work.
                self.assertEqual(before, after)
                if phase == "ancestry":
                    self.assertEqual(loads, 303)
        self.assert_closed(old_ledger)


    # ------------------------------------------- CFR-168 zero-residual repair

    def test_expired_request_cannot_publish_or_bootstrap_a_layout(self):
        """1.0 s elapsed against a 100 ms request must reach no effect at all.

        This is the reproduced BLOCKER-1 atomicity case.  On the reviewed target
        the same injection returned 200, moved ``branches/main.json`` to the
        uploaded tip and made the object durable while the deadline's
        ``poll_count`` never moved between expiry and publication.
        """
        project_root = self.bootstrap("expired")
        tip, child = self.commit([self.head], "expired child")
        cases = (
            (
                "publication-preflight-seal",
                "RemoteJSONPublicationPreflight",
                "seal_for_effect",
            ),
        )
        for label, owner_name, method in cases:
            with self.subTest(fence=label):
                before = self.authority_snapshot(project_root)
                clock = InjectedClock()
                deadline = self.deadline(clock)
                ledger = self.ledger(label=f"expired {label}")
                owner = self.namespace[owner_name]
                real = getattr(owner, method)
                effects = []
                real_write = self.globals["write_object_records"]
                real_publish = self.globals[
                    "execute_remote_json_publication"
                ]

                def expiring(inner, *args, **kwargs):
                    result = real(inner, *args, **kwargs)
                    # Elapse the deadline exactly at the last pre-effect step.
                    clock.expire()
                    return result

                def watched_write(*args, **kwargs):
                    effects.append("write_object_records")
                    return real_write(*args, **kwargs)

                def watched_publish(*args, **kwargs):
                    effects.append("execute_remote_json_publication")
                    return real_publish(*args, **kwargs)

                setattr(owner, method, expiring)
                self.globals["write_object_records"] = watched_write
                self.globals[
                    "execute_remote_json_publication"
                ] = watched_publish
                try:
                    with self.assertRaises(
                        self.namespace["RequestDeadlineExceeded"]
                    ) as caught:
                        self.upload(
                            "expired",
                            tip,
                            [child],
                            ledger=ledger,
                            deadline=deadline,
                        )
                finally:
                    setattr(owner, method, real)
                    self.globals["write_object_records"] = real_write
                    self.globals[
                        "execute_remote_json_publication"
                    ] = real_publish
                # The typed deadline outcome, named for the fence that caught it.
                self.assertEqual(
                    deadline.expired_phase, "upload publication effect"
                )
                self.assertIn("100 ms", str(caught.exception))
                # Zero effect: no object write, no publication, byte-identical
                # authority, branch unmoved, tip absent from the remote store.
                self.assertEqual(effects, [])
                self.assertEqual(
                    self.authority_snapshot(project_root), before
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assertIsNone(
                    self.namespace["object_record_path"](
                        project_root / "objects", tip
                    )
                )
                self.assert_closed(ledger)

        # Same fence on the bootstrap path: a project that has no durable layout
        # must not acquire one from an already-elapsed request.
        clock = InjectedClock()
        deadline = self.deadline(clock)
        ledger = self.ledger(label="expired bootstrap")
        real_verify = self.globals["run_server_verification"]
        real_layout = self.globals["ensure_remote_layout"]
        layouts = []

        def expiring_verify(*args, **kwargs):
            result = real_verify(*args, **kwargs)
            clock.expire()
            return result

        def watched_layout(path):
            layouts.append(path)
            return real_layout(path)

        self.globals["run_server_verification"] = expiring_verify
        self.globals["ensure_remote_layout"] = watched_layout
        try:
            with self.assertRaises(
                self.namespace["RequestDeadlineExceeded"]
            ):
                self.upload(
                    "expired-bootstrap",
                    self.head,
                    self.records,
                    ledger=ledger,
                    deadline=deadline,
                )
        finally:
            self.globals["run_server_verification"] = real_verify
            self.globals["ensure_remote_layout"] = real_layout
        bootstrap_root = self.project_root("expired-bootstrap")
        self.assertEqual(deadline.expired_phase, "upload final subject fence")
        self.assertEqual(layouts, [])
        self.assertFalse(bootstrap_root.exists())
        self.assert_closed(ledger)

    def test_removing_the_final_effect_poll_publishes_an_expired_request(self):
        """Mutation evidence: the final fence is what stops the publication.

        Suppressing only the ``upload publication effect`` poll -- leaving every
        other poll in place -- restores the reviewed target's behaviour exactly:
        an already-elapsed request publishes and answers 200.
        """
        project_root = self.bootstrap("fence-mutation")
        tip, child = self.commit([self.head], "fence mutation child")
        clock = InjectedClock()
        deadline = self.deadline(clock)
        suppressed = []
        real_check = deadline.check

        def lenient_check(phase):
            if phase == "upload publication effect":
                suppressed.append(phase)
                return
            return real_check(phase)

        deadline.check = lenient_check
        owner = self.namespace["RemoteJSONPublicationPreflight"]
        real_seal = owner.seal_for_effect

        def expiring(inner, *args, **kwargs):
            result = real_seal(inner, *args, **kwargs)
            clock.expire()
            return result

        owner.seal_for_effect = expiring
        ledger = self.ledger(label="fence mutation")
        try:
            status, _payload = self.upload(
                "fence-mutation", tip, [child], ledger=ledger, deadline=deadline
            )
        finally:
            owner.seal_for_effect = real_seal
        # Without that one poll the defect is back, byte for byte.
        self.assertEqual(suppressed, ["upload publication effect"])
        self.assertEqual(status, 200)
        self.assertIsNone(deadline.expired_phase)
        self.assertEqual(self.branch_head(project_root), tip)
        self.assertIsNotNone(
            self.namespace["object_record_path"](
                project_root / "objects", tip
            )
        )
        self.assert_closed(ledger)

    def test_one_upload_owns_every_sealed_validation_ledger(self):
        """No ledger is minted inside a request and no root is traversed twice.

        The reviewed target built four sessions per successful upload, two of
        them on fresh ``local_v1()`` ledgers whose 1 GiB retained allowance was
        neither charged to the request nor polled against its deadline.
        """
        project_root = self.bootstrap("single-ledger")
        self.write_signature_policy(
            project_root,
            {"required": True, "require_history": True, "keys": {"alice": "s"}},
        )
        tip, records = self.signed_chain(self.head, 3, "single", "s")
        sessions = []
        traversals = []
        clock = InjectedClock()
        deadline = self.deadline(clock)
        ledger = self.ledger(label="single ledger")
        minted, restore_ledger = self.request_ledger_observer()
        restore_sessions = self.observe_session_owners(sessions)
        restore_roots = self.observe_root_traversals(traversals)
        try:
            status, _payload = self.upload(
                "single-ledger", tip, records, ledger=ledger, deadline=deadline
            )
        finally:
            restore_roots()
            restore_sessions()
            restore_ledger()
        self.assertEqual(status, 200)
        # Not one ledger other than the request's.
        self.assertEqual(minted, [])
        self.assertGreater(len(sessions), 0)
        for observation in sessions:
            with self.subTest(session=observation["ledger_command"]):
                self.assertIs(observation["ledger"], ledger)
                # Every session has an explicit memory owner: the payload owner
                # covers its own pool, the traversal sources their control state.
                self.assertIsNotNone(
                    observation["control_owner"] or observation["memory_owner"]
                )
        # The publication root is traversed exactly once in the whole request.
        self.assertEqual(traversals.count(tip), 1)
        self.assertEqual(len(traversals), 1)
        # The deadline is polled throughout, including inside validation.
        self.assertGreater(deadline.poll_count, len(records))
        self.assertIsNone(deadline.expired_phase)
        self.assert_closed(ledger)

        # Mutation evidence: restoring the nested publication-root validation
        # brings back both a fresh ledger and a second traversal of the same root.
        child_id, child = self.commit(
            [tip], "single ledger child", signer="alice", key="s"
        )
        real_identity = self.globals[
            "validate_remote_json_publication_identity"
        ]

        def nested_identity(*args, **kwargs):
            # The reviewed wiring: no proof, no ledger, no deadline.
            kwargs.pop("root_proof", None)
            kwargs.pop("ledger", None)
            kwargs.pop("deadline", None)
            return real_identity(*args, **kwargs)

        sessions = []
        traversals = []
        minted, restore_ledger = self.request_ledger_observer()
        restore_sessions = self.observe_session_owners(sessions)
        restore_roots = self.observe_root_traversals(traversals)
        self.globals[
            "validate_remote_json_publication_identity"
        ] = nested_identity
        mutated = self.ledger(label="nested restored")
        try:
            status, _payload = self.upload(
                "single-ledger", child_id, [child], ledger=mutated
            )
        finally:
            self.globals[
                "validate_remote_json_publication_identity"
            ] = real_identity
            restore_roots()
            restore_sessions()
            restore_ledger()
        self.assertEqual(status, 200)
        self.assertIn("sealed validation", minted)
        self.assertEqual(traversals.count(child_id), 2)
        uncovered = [
            observation
            for observation in sessions
            if observation["ledger"] is not mutated
        ]
        self.assertGreater(len(uncovered), 0)

    def test_source_session_allocations_prove_an_outstanding_reservation(self):
        """Every control allocation sees a reservation that can own it.

        HIGH-3 measured 24 sites on the ancestry source session that had no
        reservation at all, and MEDIUM-4 measured 78 never-removed descriptor
        owners on the covered session of a 27-record upload.
        """
        project_root = self.bootstrap("source-owner")
        self.write_signature_policy(
            project_root,
            {"required": True, "require_history": True, "keys": {"alice": "s"}},
        )
        tip, records = self.signed_chain(self.head, 6, "source", "s")
        sessions = []
        ledger = self.ledger(label="source owner")
        restore = self.observe_session_owners(sessions)
        try:
            status, _payload = self.upload(
                "source-owner", tip, records, ledger=ledger
            )
        finally:
            restore()
        self.assertEqual(status, 200)
        source_sessions = [
            observation
            for observation in sessions
            if type(observation["control_owner"]).__name__
            == "_AccountedUploadSourceMemory"
        ]
        self.assertGreater(len(source_sessions), 0)
        sites = 0
        for observation in sessions:
            for site in observation["sites"]:
                sites += 1
                with self.subTest(site=site["site"]):
                    # Charge before allocate: the reservation that will own this
                    # structure is outstanding before the structure exists.
                    self.assertGreaterEqual(
                        site["outstanding"], site["needed"]
                    )
        self.assertGreater(sites, 0)
        # Descriptor owners are reused, so the registry is bounded by the
        # simultaneously open descriptors instead of growing once per read.  The
        # reviewed target appended one per record read: 78 on a 27-record upload.
        bound = len(self.namespace["OBJECT_PREFIXES"]) + 3
        for observation in sessions:
            with self.subTest(session=observation["ledger_command"]):
                self.assertLessEqual(
                    max(site["fd_owners"] for site in observation["sites"]),
                    bound,
                )
        self.assert_closed(ledger)

        # Mutation evidence: omitting the source owner leaves every one of those
        # sites with no reservation to name.
        uncovered = self.namespace["SealedCommitValidationSession"](
            project_root / "objects", self.ledger(label="uncovered source")
        )
        try:
            self.assertEqual(
                uncovered.control_reservation_outstanding("retained_bytes"), 0
            )
        finally:
            uncovered.close(None)

    def test_default_profile_record_shape_and_size_curves(self):
        """Record-size and record-shape curves under the unmodified profile.

        HIGH-2 measured this exact route refusing any bundle holding one record
        above 126,976 B in ``semantic_validation``, before either traversal,
        because the reservation carried a ``128 x maximum_record_bytes`` heap
        term that corresponded to no retained claim.  Every size below is now
        admitted, and the first refusal is at a different, documented owner.
        """
        limits = self.ledger(label="curve limits").profile
        request_body_limit = limits.limit("request_body_bytes")
        # Measured on the unmodified ``http_v1`` profile.  The reviewed target
        # refused every size from 131,072 B upwards in ``semantic_validation``;
        # the first refusal is now ``staging``, an owner reached before sealed
        # validation is even prepared::
        #
        #     record bytes  reviewed target                     this repair
        #           65,536  admitted                            200
        #          126,976  admitted (largest)                  200
        #          131,072  refused temporary/semantic_valid.   200
        #          262,144  refused retained/semantic_valid.    200
        #        1,048,576  refused retained/semantic_valid.    200
        #        2,097,152  refused                             200
        #        3,145,728  refused                             200
        #        4,194,304  refused staging/temporary_bytes     refused staging
        sizes = (
            64 * 1024,
            126_976,
            128 * 1024,
            256 * 1024,
            1024 * 1024,
            2 * 1024 * 1024,
            3 * 1024 * 1024,
            # A practical near-request-body boundary: one record occupying a
            # quarter of the whole admissible request body.
            request_body_limit // 4,
        )
        curves = []
        for size in sizes:
            with self.subTest(record_bytes=size):
                app = f"size-{size}"
                project_root = self.bootstrap(app)
                before = self.authority_snapshot(project_root)
                tip, child = self.sized_commit([self.head], "size", size)
                sealed = []
                ledger = self.ledger(label=f"size {size}")
                restore = self.observe_sealed_validation(sealed)
                outcome = None
                diagnostic = None
                try:
                    status, _payload = self.upload(
                        app, tip, [child], ledger=ledger
                    )
                    outcome = status
                except self.namespace["ResourceBudgetExceeded"] as exc:
                    diagnostic = exc.diagnostic()
                finally:
                    restore()
                curves.append(
                    {
                        "size": size,
                        "status": outcome,
                        "diagnostic": diagnostic,
                        "bounds": (
                            None
                            if not sealed
                            else (
                                sealed[0]["record_count"],
                                sealed[0]["aggregate_bytes"],
                                sealed[0]["maximum_record_bytes"],
                            )
                        ),
                    }
                )
                if diagnostic is not None:
                    # A refusal is pre-effect, typed, and at a named owner that
                    # is never the sealed reservation for these sizes.
                    self.assertEqual(
                        self.authority_snapshot(project_root), before
                    )
                    self.assertEqual(
                        self.branch_head(project_root), self.head
                    )
                    self.assertNotEqual(
                        diagnostic["phase"], "semantic_validation"
                    )
                self.assert_closed(ledger)

        admitted = [curve for curve in curves if curve["status"] == 200]
        refused = [curve for curve in curves if curve["status"] is None]
        # Everything the reviewed target refused between 128 KiB and 1 MiB is
        # admitted now, and the recorded ceiling moves if any multiplier returns.
        for curve in curves:
            if curve["size"] <= 3 * 1024 * 1024:
                with self.subTest(record_bytes=curve["size"], curve="admitted"):
                    self.assertEqual(curve["status"], 200)
        self.assertGreaterEqual(len(admitted), 7)
        largest_admitted = max(curve["size"] for curve in admitted)
        self.assertGreaterEqual(largest_admitted, 3 * 1024 * 1024)
        # The recorded ceiling is a real boundary, not an open-ended claim: at
        # least one size in the curve is refused, and it is refused elsewhere.
        self.assertGreater(len(refused), 0)
        for curve in refused:
            with self.subTest(record_bytes=curve["size"], curve="refused"):
                # The only refusals left are earlier resource owners, reached
                # before sealed validation is even prepared.
                self.assertIn(
                    curve["diagnostic"]["phase"],
                    {"staging", "request_body", "encoding"},
                )

        # The sealed reservation's own admissible ceiling, recomputed from the
        # owner constants, is above every size admitted live and far above the
        # 126,976 B the reviewed target stopped at.
        owner = self.namespace["AccountedHTTPUploadSealedValidation"]
        retained_limit = limits.limit("retained_bytes")
        temporary_limit = limits.limit("temporary_bytes")

        def sealed_admits(record_bytes):
            retained, temporary = owner.allocation_upper(
                101, 66_000 + record_bytes, record_bytes
            )
            return retained <= retained_limit and temporary <= temporary_limit

        low, high = 0, 64 * 1024 * 1024
        while low + 1 < high:
            middle = (low + high) // 2
            if sealed_admits(middle):
                low = middle
            else:
                high = middle
        self.assertGreater(low, 3 * 1024 * 1024)
        self.assertGreater(low, largest_admitted)

        # The record-count ceiling is recorded too, so a future fixture between
        # the contracted 1,201 records and that ceiling cannot pass while one
        # just above it fails with no test naming the boundary.
        mean_record_bytes = 805

        def count_admits(count):
            retained, temporary = owner.allocation_upper(
                count, mean_record_bytes * count, mean_record_bytes
            )
            return retained <= retained_limit and temporary <= temporary_limit

        low_count, high_count = 1, 1_000_000
        while low_count + 1 < high_count:
            middle = (low_count + high_count) // 2
            if count_admits(middle):
                low_count = middle
            else:
                high_count = middle
        # Measured 4,959 at 805 B/record.  The contracted 1,201-commit fixture
        # sits at a quarter of it, and the boundary itself is exact.
        self.assertTrue(count_admits(low_count))
        self.assertFalse(count_admits(low_count + 1))
        self.assertGreater(low_count, 4_000)
        self.assertGreater(low_count, 2 * 1_207)
        # Mutation evidence: reinstating the per-record heap multiplier refuses
        # a 256 KiB record on the same unmodified profile.
        reinstated = self.reproduced_max_record_retained_bound(
            101, 66_000 + 256 * 1024, 256 * 1024
        )
        self.assertGreater(reinstated, retained_limit)
        self.assertLess(
            self.owner_lifetime_retained_bound(
                101, 66_000 + 256 * 1024, 256 * 1024
            ),
            retained_limit,
        )

    def test_record_shape_curves_reach_the_same_owner(self):
        """ASCII, CJK, emoji, escape-heavy and control-character records."""
        target = 128 * 1024
        observed = {}
        for name, unit in self.RECORD_SHAPES:
            with self.subTest(shape=name):
                app = f"shape-{name}"
                project_root = self.bootstrap(app)
                tip, child = self.sized_commit(
                    [self.head], f"shape-{name}", target, unit
                )
                sealed = []
                ledger = self.ledger(label=f"shape {name}")
                restore = self.observe_sealed_validation(sealed)
                try:
                    status, _payload = self.upload(
                        app, tip, [child], ledger=ledger
                    )
                finally:
                    restore()
                self.assertEqual(status, 200)
                self.assertEqual(len(sealed), 1)
                bounds = (
                    sealed[0]["record_count"],
                    sealed[0]["aggregate_bytes"],
                    sealed[0]["maximum_record_bytes"],
                )
                # The reservation stays exactly the documented owner-lifetime
                # bound whatever the encoding costs, and the claims stay inside it.
                self.assertEqual(
                    sealed[0]["owner"]._coverage_retained_upper,
                    self.owner_lifetime_retained_bound(*bounds),
                )
                self.assertGreater(bounds[2], target // 2)
                self.assertEqual(self.branch_head(project_root), tip)
                observed[name] = bounds
                self.assert_closed(ledger)
        self.assertEqual(len(observed), len(self.RECORD_SHAPES))

    def test_graph_shape_curves_reach_the_same_owner(self):
        """Wide fan-in, diamond, deep-chain and signed-history graph shapes."""
        cases = []
        fan_tip, fan_records = self.fan_in(self.head, 40, "fan")
        cases.append(("fan-in", fan_tip, fan_records, None))
        diamond_tip, diamond_records = self.diamonds(self.head, 20, "diamond")
        cases.append(("diamond", diamond_tip, diamond_records, None))
        chain_tip, chain_records = self.chain(self.head, 120, "deep")
        cases.append(("deep-chain", chain_tip, chain_records, None))
        signed_tip, signed_records = self.signed_chain(
            self.head, 20, "signed", "s"
        )
        cases.append(
            (
                "signed-history",
                signed_tip,
                signed_records,
                {
                    "required": True,
                    "require_history": True,
                    "keys": {"alice": "s"},
                },
            )
        )
        for name, tip, records, policy in cases:
            with self.subTest(shape=name):
                app = f"graph-{name}"
                project_root = self.bootstrap(app)
                if policy is not None:
                    self.write_signature_policy(project_root, policy)
                sealed = []
                sessions = []
                ledger = self.ledger(label=f"graph {name}")
                restore = self.observe_sealed_validation(sealed)
                restore_sessions = self.observe_session_owners(sessions)
                try:
                    status, _payload = self.upload(
                        app, tip, records, ledger=ledger
                    )
                finally:
                    restore_sessions()
                    restore()
                self.assertEqual(status, 200)
                self.assertEqual(len(sealed), 1)
                owner = sealed[0]["owner"]
                bounds = (
                    sealed[0]["record_count"],
                    sealed[0]["aggregate_bytes"],
                    sealed[0]["maximum_record_bytes"],
                )
                self.assertEqual(
                    owner._coverage_retained_upper,
                    self.owner_lifetime_retained_bound(*bounds),
                )
                self.assertEqual(
                    owner._coverage_temporary_upper,
                    self.owner_lifetime_temporary_bound(bounds[0], bounds[2]),
                )
                # Every claim on this shape fitted inside the reservation, and
                # every session's control state was owned.
                self.assertGreater(
                    owner._claimed_memory_peak["retained_bytes"], 0
                )
                self.assertLessEqual(
                    owner._claimed_memory_peak["retained_bytes"],
                    owner._coverage_retained_upper,
                )
                for observation in sessions:
                    for site in observation["sites"]:
                        self.assertGreaterEqual(
                            site["outstanding"], site["needed"]
                        )
                self.assertEqual(self.branch_head(project_root), tip)
                self.assert_closed(ledger)

    def test_temporary_boundaries_reach_the_streaming_validation_owner(self):
        """Exact / minus-one / plus-one on the streamed temporary upper."""
        depth = 120
        _root, tip, records = self.divergent_fixture(
            "temporary-observe", depth, prefix="temporary"
        )
        sealed = []
        observation = self.ledger(label="temporary observation")
        restore = self.observe_sealed_validation(sealed)
        try:
            status, _payload = self.upload(
                "temporary-observe", tip, records, ledger=observation
            )
        finally:
            restore()
        self.assertEqual(status, 409)
        self.assertEqual(len(sealed), 1)
        owner = sealed[0]["owner"]
        bounds = (
            sealed[0]["record_count"],
            sealed[0]["aggregate_bytes"],
            sealed[0]["maximum_record_bytes"],
        )
        reservation = owner._coverage_temporary_upper
        self.assertEqual(
            reservation,
            self.owner_lifetime_temporary_bound(bounds[0], bounds[2]),
        )
        peak = observation.peak.get("temporary_bytes")
        self.assertLessEqual(reservation, peak)
        self.assert_closed(observation)

        for label, limit in (("exact", peak), ("plus-one", peak + 1)):
            with self.subTest(boundary=label):
                app = f"temporary-{label}"
                project_root, boundary_tip, boundary_records = (
                    self.divergent_fixture(app, depth, prefix="temporary")
                )
                ledger = self.ledger(
                    {"temporary_bytes": ("concurrent", limit)},
                    f"temporary {label}",
                )
                status, payload = self.upload(
                    app, boundary_tip, boundary_records, ledger=ledger
                )
                self.assertEqual(status, 409)
                self.assertEqual(
                    payload,
                    {
                        "error": "upload rejected: remote branch is not an "
                        "ancestor of local branch head"
                    },
                )
                self.assertEqual(
                    ledger.peak.get("temporary_bytes"), peak
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assert_closed(ledger)

        app = "temporary-minus-one"
        project_root, minus_tip, minus_records = self.divergent_fixture(
            app, depth, prefix="temporary"
        )
        before = self.authority_snapshot(project_root)
        ledger = self.ledger(
            {"temporary_bytes": ("concurrent", peak - 1)},
            "temporary minus one",
        )
        with self.assertRaises(
            self.namespace["ResourceBudgetExceeded"]
        ) as caught:
            self.upload(app, minus_tip, minus_records, ledger=ledger)
        diagnostic = caught.exception.diagnostic()
        self.assertEqual(diagnostic["dimension"], "temporary_bytes")
        self.assertEqual(diagnostic["limit"], peak - 1)
        self.assertEqual(diagnostic["observed"], peak)
        self.assertEqual(ledger.terminal_reason, "budget_exceeded")
        # Pre-effect: nothing published, nothing staged, no debt.
        self.assertEqual(self.authority_snapshot(project_root), before)
        self.assertEqual(self.branch_head(project_root), self.head)
        self.assertIsNone(
            self.namespace["object_record_path"](
                project_root / "objects", minus_tip
            )
        )
        self.assert_closed(ledger)

    def test_streamed_canonical_encoder_matches_the_whole_record_encoder(self):
        """The streamed digest is byte-identical and bounded.

        The borrow fingerprint used to reserve ``128 x`` one record's canonical
        bytes for a whole-record encode.  The replacement must produce exactly
        the same bytes with scratch that does not scale with the record.
        """
        canonical_json = self.namespace["canonical_json"]
        stream_digest = self.namespace["canonical_json_stream_digest"]
        samples = [
            {},
            [],
            "",
            0,
            -1,
            True,
            False,
            None,
            1.5,
            {"b": 1, "a": 2, "C": 3, "": 4},
            {"a": {"b": {"c": [1, {"d": None}]}}},
            {"k": ["a" * 5000, "漢" * 5000, "\U0001f600" * 5000]},
            {"k": ['"\\\n\t' * 3000, "\x01\x02" * 4000, "\x7f"]},
            {str(index): index for index in range(500)},
            ["x" * 4095, "y" * 4096, "z" * 4097, "w" * 8192],
        ]
        samples.extend({"cp": chr(point)} for point in range(0, 0x120))
        samples.extend(
            {"cp": chr(point)}
            for point in (0x7F, 0x80, 0xA0, 0xFFFF, 0x10000, 0x10FFFF)
        )
        for index, sample in enumerate(samples):
            with self.subTest(sample=index):
                expected = canonical_json(sample)
                size, digest = stream_digest(sample)
                self.assertEqual(size, len(expected))
                self.assertEqual(
                    digest, hashlib.sha256(expected).hexdigest()
                )

        # The key projection each mapping reports balances back to zero, so the
        # guard the owner installs can prove ownership of every one of them.
        deltas = []
        stream_digest(
            {"a": {"b": [1, 2, {"c": 3}]}, "d": {}}, None, deltas.append
        )
        self.assertTrue(deltas)
        self.assertEqual(sum(deltas), 0)

        # Scratch does not scale with the record: a 16x larger document holds no
        # more than a constant multiple of the smaller one's peak.
        def streamed_peak(text):
            record = {"payload": {"message": text, "parents": []}}
            tracemalloc.start()
            tracemalloc.reset_peak()
            base = tracemalloc.get_traced_memory()[0]
            stream_digest(record)
            peak = tracemalloc.get_traced_memory()[1]
            tracemalloc.stop()
            return peak - base

        small = streamed_peak("q" * 128 * 1024)
        large = streamed_peak("q" * 2048 * 1024)
        self.assertLessEqual(
            large,
            self.namespace["CANONICAL_STREAM_SCRATCH_BYTES"],
        )
        self.assertLessEqual(large, 4 * max(small, 1))


    # -------------------------------------------- CFR-168 review-closure repair

    def test_completed_quarantine_recovery_owns_the_request_budget(self):
        """An expired retry must not complete a quarantined publication.

        The reviewed candidate reached ``recover_remote_pending_quarantine``
        without the request ledger or deadline, minted a fresh ``local_graph``
        ``sealed validation`` ledger, deleted the completed quarantine and
        returned 200 after the clock was advanced to 1.0 s against a 100 ms
        request.
        """
        project_root, tip, records, quarantine = self.completed_quarantine(
            "recovery"
        )
        before = self.recovery_snapshot(project_root, quarantine)
        self.assertNotEqual(before["quarantine"], "absent")

        clock = InjectedClock()
        deadline = self.deadline(clock)
        ledger = self.ledger(label="recovery expired")
        minted, restore_ledger = self.request_ledger_observer()
        real_recover = self.globals["recover_remote_pending_quarantine"]

        def expiring_recover(*args, **kwargs):
            # Elapse the request immediately before the recovered-root work.
            clock.expire()
            return real_recover(*args, **kwargs)

        self.globals[
            "recover_remote_pending_quarantine"
        ] = expiring_recover
        try:
            with self.assertRaises(
                self.namespace["RequestDeadlineExceeded"]
            ) as caught:
                self.upload(
                    "recovery",
                    tip,
                    records,
                    ledger=ledger,
                    deadline=deadline,
                    key="quarantine-key",
                )
        finally:
            self.globals[
                "recover_remote_pending_quarantine"
            ] = real_recover
            restore_ledger()
        self.assertEqual(
            deadline.expired_phase, "upload recovery sealed validation"
        )
        self.assertIn("100 ms", str(caught.exception))
        # No foreign ledger, and every recorded authority byte-identical --
        # including the completed quarantine the candidate deleted.
        self.assertEqual(minted, [])
        after = self.recovery_snapshot(project_root, quarantine)
        self.assertEqual(after, before)
        self.assertTrue(quarantine.is_file())
        self.assert_closed(ledger)

        # The live retry still completes the recovery on the request budget.
        live = self.ledger(label="recovery live")
        live_deadline = self.deadline(InjectedClock())
        minted, restore_ledger = self.request_ledger_observer()
        traversals = []
        restore_roots = self.observe_root_traversals(traversals)
        try:
            status, payload = self.upload(
                "recovery",
                tip,
                records,
                ledger=live,
                deadline=live_deadline,
                key="quarantine-key",
            )
        finally:
            restore_roots()
            restore_ledger()
        self.assertEqual(status, 200)
        self.assertEqual(payload, {"branch": "main", "head": tip})
        self.assertEqual(minted, [])
        # Exactly one traversal of the staged head plus one of the recovered
        # durable root, both on the request ledger.
        self.assertEqual(traversals.count(tip), 2)
        self.assertFalse(quarantine.exists())
        self.assertIsNone(live_deadline.expired_phase)
        self.assert_closed(live)

        # Idempotent replay after the recovery returns the same bytes.
        replay = self.ledger(label="recovery replay")
        replay_status, replay_payload = self.upload(
            "recovery", tip, records, ledger=replay, key="quarantine-key"
        )
        self.assertEqual((replay_status, replay_payload), (status, payload))
        self.assert_closed(replay)

    def test_dropping_the_recovery_handoff_publishes_an_expired_retry(self):
        """Mutation evidence: the recovery handoff is what stops the retry.

        Dropping only the ledger/deadline arguments -- exactly the reviewed
        candidate's wiring -- restores a fresh ``sealed validation`` ledger and a
        200 that clears the quarantine after the deadline elapsed.
        """
        project_root, tip, records, quarantine = self.completed_quarantine(
            "recovery-mutation"
        )
        clock = InjectedClock()
        deadline = self.deadline(clock)
        ledger = self.ledger(label="recovery mutation")
        minted, restore_ledger = self.request_ledger_observer()
        real_recover = self.globals["recover_remote_pending_quarantine"]

        def unthreaded_recover(*args, **kwargs):
            clock.expire()
            kwargs.pop("ledger", None)
            kwargs.pop("deadline", None)
            return real_recover(*args, **kwargs)

        self.globals[
            "recover_remote_pending_quarantine"
        ] = unthreaded_recover
        try:
            status, _payload = self.upload(
                "recovery-mutation",
                tip,
                records,
                ledger=ledger,
                deadline=deadline,
                key="quarantine-key",
            )
        finally:
            self.globals[
                "recover_remote_pending_quarantine"
            ] = real_recover
            restore_ledger()
        # Without the handoff the reviewed defect is back, byte for byte.
        self.assertEqual(status, 200)
        self.assertIn("sealed validation", minted)
        self.assertIsNone(deadline.expired_phase)
        self.assertFalse(quarantine.exists())
        self.assertEqual(self.branch_head(project_root), tip)
        self.assert_closed(ledger)

    def test_caller_owned_sealed_validation_apis_complete(self):
        """All three public caller-owned helpers must finish, not reject.

        The reviewed candidate failed every one of them with
        ``upload commit source exceeded its prepared temporary_bytes
        reservation``, because its source owner reserved no temporary capacity
        for ``finalize``'s ordered projection.
        """
        one_parent = self.write_local_commit([self.head], "caller one")
        many = one_parent
        for index in range(20):
            many = self.write_local_commit([many], f"caller many {index}")
        objects = self.local_objects
        project_root = self.bootstrap("caller-api")
        writes = [({"kind": "generation"}, None, {"current_generation": 99})]

        def sealed(head, ledger, deadline):
            self.namespace["validate_sealed_commit"](
                objects, head, ledger=ledger, deadline=deadline
            )

        def sealed_memoised(head, ledger, deadline):
            # An already-validated closure finalizes with an empty projection.
            self.namespace["validate_sealed_commit"](
                objects,
                head,
                {head},
                ledger=ledger,
                deadline=deadline,
            )

        def identity(head, ledger, deadline):
            self.namespace[
                "validate_remote_json_publication_identity"
            ](
                "remote_upload",
                "alice",
                None,
                [head],
                objects_root=objects,
                ledger=ledger,
                deadline=deadline,
            )

        def preflight(head, ledger, deadline):
            self.namespace["preflight_remote_json_publication"](
                project_root,
                "remote_upload",
                "alice",
                {"subject": "caller"},
                writes,
                {"branch": "main", "head": head},
                None,
                [head],
                validation_objects_root=objects,
                validate_preimages=False,
                ledger=ledger,
                deadline=deadline,
            )

        closures = (
            ("empty-projection", self.head, sealed_memoised),
            ("one-record", self.head, sealed),
            ("multi-record", many, sealed),
            ("identity", self.head, identity),
            ("identity-multi", many, identity),
            ("preflight", self.head, preflight),
            ("preflight-multi", many, preflight),
        )
        for name, head, call in closures:
            for label, use_deadline in (("ledger", False), ("deadline", True)):
                with self.subTest(api=name, budget=label):
                    ledger = self.ledger(label=f"{name} {label}")
                    deadline = (
                        self.deadline(InjectedClock())
                        if use_deadline
                        else None
                    )
                    owners = []
                    restore = self.observe_source_memory(owners)
                    try:
                        call(head, ledger, deadline)
                    finally:
                        restore()
                    self.assertEqual(len(owners), 1)
                    owner = owners[0]
                    # One owner, published then fully released, never detached
                    # with debt and never double-charged.
                    self.assertTrue(owner._closed)
                    self.assertFalse(owner._acquired)
                    self.assertEqual(
                        owner._control_claimed,
                        {"retained_bytes": 0, "temporary_bytes": 0},
                    )
                    self.assertGreater(
                        owner._control_peak["retained_bytes"], 0
                    )
                    self.assert_closed(ledger)
                    self.assertIsNone(ledger.terminal_reason)
                    if deadline is not None:
                        self.assertGreater(deadline.poll_count, 0)
                        self.assertIsNone(deadline.expired_phase)

        # The no-ledger compatibility path and its precedence are untouched.
        self.namespace["validate_remote_json_publication_identity"](
            "remote_upload", "alice", None, [self.head], objects_root=objects
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "remote publication actor is invalid"
        ):
            self.namespace["validate_remote_json_publication_identity"](
                "remote_upload", "", None, [self.head], objects_root=objects
            )

    def test_ordered_projection_claim_is_exact_and_precedes_its_allocation(
        self,
    ):
        """The ordered projection is charged, sized exactly, and released.

        This is the MEDIUM-4 oracle for the projection claim: removing the claim
        while the list allocation and sort remain makes the minus-one case
        succeed, so this test fails.
        """
        head = self.head
        for index in range(400):
            head = self.write_local_commit([head], f"projection {index}")
        objects = self.local_objects
        sizes = []
        real_finalize = self.globals[
            "SealedCommitValidationSession"
        ].finalize

        def observed_finalize(inner, **kwargs):
            sizes.append(len(inner.documents))
            return real_finalize(inner, **kwargs)

        self.globals[
            "SealedCommitValidationSession"
        ].finalize = observed_finalize
        sorted_at = []
        real_sort = self.globals["_sort_strings_polling"]
        owners = []
        restore_owner = self.observe_source_memory(owners)
        ledger = self.ledger(label="projection observation")

        def observed_sort(values, poll=None):
            # Acquisition precedes allocation: by the time the projection is
            # sorted, the caller ledger already carries its exact cost.  Root-key
            # sorts inside payload validation share this helper, so each sample
            # records the list length it was given.
            sorted_at.append(
                (
                    len(values),
                    ledger.current.get("temporary_bytes", 0),
                    owners[0]._control_claimed["temporary_bytes"]
                    if owners
                    else 0,
                )
            )
            return real_sort(values, poll)

        self.globals["_sort_strings_polling"] = observed_sort
        try:
            self.namespace["validate_sealed_commit"](
                objects, head, ledger=ledger
            )
        finally:
            self.globals["_sort_strings_polling"] = real_sort
            self.globals[
                "SealedCommitValidationSession"
            ].finalize = real_finalize
            restore_owner()
        self.assertEqual(len(sizes), 1)
        documents = sizes[0]
        projection = self.ordered_projection_upper(documents)
        # The exact formula, and the review's own runtime observations.
        self.assertEqual(self.ordered_projection_upper(0), 56)
        self.assertEqual(self.ordered_projection_upper(1), 64)
        self.assertEqual(self.ordered_projection_upper(27), 272)
        projection_sorts = [
            sample for sample in sorted_at if sample[0] == documents
        ]
        self.assertEqual(len(projection_sorts), 1)
        for _length, ledger_temporary, owner_temporary in projection_sorts:
            self.assertGreaterEqual(ledger_temporary, projection)
            self.assertEqual(owner_temporary, projection)
        # Peak use, release and terminal zero.
        self.assertEqual(
            owners[0]._control_peak["temporary_bytes"], projection
        )
        self.assertEqual(owners[0]._control_claimed["temporary_bytes"], 0)
        peak = ledger.peak.get("temporary_bytes")
        self.assertGreater(peak, projection)
        self.assert_closed(ledger)

        # Exact / minus-one on the projection itself.  At ``projection - 1``
        # nothing earlier on this route needs that much, so the refusal is the
        # projection claim, named by its own dimension, phase and exact size.
        boundaries = (
            ("projection-minus-one", projection - 1, projection),
            ("peak-minus-one", peak - 1, peak),
        )
        for label, limit, observed in boundaries:
            with self.subTest(boundary=label):
                refused = self.ledger(
                    {"temporary_bytes": ("concurrent", limit)}, label
                )
                with self.assertRaises(
                    self.namespace["ResourceBudgetExceeded"]
                ) as caught:
                    self.namespace["validate_sealed_commit"](
                        objects, head, ledger=refused
                    )
                diagnostic = caught.exception.diagnostic()
                self.assertEqual(diagnostic["dimension"], "temporary_bytes")
                self.assertEqual(
                    diagnostic["phase"], "snapshot_revalidation"
                )
                self.assertEqual(diagnostic["limit"], limit)
                self.assertEqual(diagnostic["observed"], observed)
                self.assert_closed(refused)

        exact = self.ledger({"temporary_bytes": ("concurrent", peak)}, "exact")
        self.namespace["validate_sealed_commit"](objects, head, ledger=exact)
        self.assertEqual(exact.peak.get("temporary_bytes"), peak)
        self.assert_closed(exact)

    def test_publication_root_proof_is_atomically_single_use(self):
        """One proof, one consumption, with an explicit state for every refusal."""
        proof_type = self.namespace["RemoteJSONPublicationRootProof"]
        objects = self.local_objects
        head = self.head

        def proof(deadline=None):
            return proof_type(objects, (head,), deadline)

        # Second use, aliased use, and revoke-after-use.
        subject = proof()
        self.assertEqual(subject.state, proof_type.AVAILABLE)
        self.assertEqual(
            subject.validated_roots(objects, (head,), "identity"), (head,)
        )
        self.assertEqual(subject.state, proof_type.CONSUMED)
        self.assertEqual(subject.consumptions, 1)
        alias = subject
        for label, reference in (("second", subject), ("alias", alias)):
            with self.subTest(reuse=label):
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "already consumed"
                ):
                    reference.validated_roots(objects, (head,), "identity")
                self.assertEqual(reference.consumptions, 1)
        subject.revoke()
        subject.revoke()
        self.assertEqual(subject.state, proof_type.REVOKED)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "is revoked"
        ):
            subject.validated_roots(objects, (head,), "identity")

        # Revoke before use is terminal too.
        early = proof()
        early.revoke()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "is revoked"
        ):
            early.validated_roots(objects, (head,), "identity")
        self.assertEqual(early.consumptions, 0)

        # A refused check consumes nothing and leaves the proof retryable.
        retryable = proof()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "different object store"
        ):
            retryable.validated_roots(
                self.tmp / "elsewhere", (head,), "identity"
            )
        self.assertEqual(retryable.state, proof_type.AVAILABLE)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "does not cover object root"
        ):
            retryable.validated_roots(
                objects, ("CF-COMMIT-absent",), "identity"
            )
        self.assertEqual(retryable.state, proof_type.AVAILABLE)
        self.assertEqual(retryable.consumptions, 0)
        self.assertEqual(
            retryable.validated_roots(objects, (head,), "identity"), (head,)
        )
        self.assertEqual(retryable.consumptions, 1)

        # A deadline constructed before the clock advances refuses without
        # consuming, and keeps refusing.
        clock = InjectedClock()
        expiring = proof(self.deadline(clock))
        clock.expire()
        for _attempt in range(2):
            with self.assertRaises(
                self.namespace["RequestDeadlineExceeded"]
            ):
                expiring.validated_roots(objects, (head,), "identity")
            self.assertEqual(expiring.state, proof_type.AVAILABLE)
            self.assertEqual(expiring.consumptions, 0)

        # Callback reentry through the injected clock cannot consume the proof
        # the outer call is already consuming.
        reentry = []
        holder = {}

        class ReentrantClock:
            now = 0.0

            def __call__(self):
                if holder.get("proof") is not None and not reentry:
                    reentry.append("entered")
                    try:
                        holder["proof"].validated_roots(
                            objects, (head,), "reentry"
                        )
                        reentry.append("accepted")
                    except BaseException as exc:
                        reentry.append(f"{type(exc).__name__}: {exc}")
                return self.now

        holder["proof"] = proof(
            self.namespace["HTTPRequestDeadline"](30_000, ReentrantClock())
        )
        self.assertEqual(
            holder["proof"].validated_roots(objects, (head,), "identity"),
            (head,),
        )
        self.assertEqual(holder["proof"].consumptions, 1)
        self.assertEqual(len(reentry), 2)
        self.assertIn("already consuming", reentry[1])

        # A live upload still consumes exactly one proof and revokes it.
        proofs = []
        real_mint = self.globals[
            "AccountedHTTPUploadSealedValidation"
        ].publication_root_proof

        def observed_mint(inner, ledger, roots):
            minted_proof = real_mint(inner, ledger, roots)
            proofs.append(minted_proof)
            return minted_proof

        project_root = self.bootstrap("proof-live")
        tip, child = self.commit([self.head], "proof live child")
        ledger = self.ledger(label="proof live")
        self.globals[
            "AccountedHTTPUploadSealedValidation"
        ].publication_root_proof = observed_mint
        try:
            status, _payload = self.upload(
                "proof-live", tip, [child], ledger=ledger
            )
        finally:
            self.globals[
                "AccountedHTTPUploadSealedValidation"
            ].publication_root_proof = real_mint
        self.assertEqual(status, 200)
        self.assertEqual(len(proofs), 1)
        self.assertEqual(proofs[0].consumptions, 1)
        self.assertEqual(proofs[0].state, proof_type.REVOKED)
        self.assertEqual(self.branch_head(project_root), tip)
        self.assert_closed(ledger)

    def test_prepared_coverage_predicates_are_load_bearing(self):
        """Each prepared-coverage predicate must change a live upload's result.

        Replacing ``_assert_prepared_coverage`` with a no-op, or removing any one
        of its three predicates, makes one of these subcases stop raising.
        """
        owner_type = self.namespace["AccountedHTTPUploadSealedValidation"]
        project_root = self.bootstrap("coverage")
        before = self.authority_snapshot(project_root)
        real_prepare = self.globals["prepare_http_upload_sealed_validation"]

        def perturbed(perturb):
            """Create the exact state one predicate exists to reject.

            The perturbation runs immediately before the production check and
            then delegates to it, so the check itself -- not the perturbation --
            decides.  Replacing ``_assert_prepared_coverage`` with a no-op makes
            every subcase below stop raising.
            """

            def prepare(ledger, objects_root, head, **kwargs):
                owner = real_prepare(ledger, objects_root, head, **kwargs)
                real_assert = owner._assert_prepared_coverage

                def checked():
                    perturb(owner)
                    return real_assert()

                owner._assert_prepared_coverage = checked
                return owner

            return prepare

        def leave_debt(owner):
            # One release skipped: the pool no longer settles to zero.
            owner._claimed_memory["retained_bytes"] += 1

        def exceed_upper(owner):
            # The recorded high-water no longer fits the admitted reservation.
            owner._claimed_memory_peak["retained_bytes"] = (
                owner._coverage_retained_upper + 1
            )

        def claim_nothing(owner):
            # The covered pool was never used, so the whole check is vacuous.
            owner._claimed_memory_peak["retained_bytes"] = 0

        cases = (
            ("retained debt", leave_debt, "retains retained_bytes debt"),
            ("peak over upper", exceed_upper, "peak exceeded its prepared"),
            ("no covered use", claim_nothing, "claimed no covered retained"),
        )
        for index, (label, perturb, message) in enumerate(cases):
            with self.subTest(predicate=label):
                tip, child = self.commit([self.head], f"coverage {index}")
                ledger = self.ledger(label=f"coverage {label}")
                self.globals[
                    "prepare_http_upload_sealed_validation"
                ] = perturbed(perturb)
                try:
                    with self.assertRaisesRegex(
                        self.namespace["CFError"], message
                    ):
                        self.upload(
                            "coverage", tip, [child], ledger=ledger
                        )
                finally:
                    self.globals[
                        "prepare_http_upload_sealed_validation"
                    ] = real_prepare
                # Pre-effect: the fence runs before the first effect, so the
                # branch never moved and the object never became durable.
                self.assertEqual(
                    self.authority_snapshot(project_root), before
                )
                self.assertEqual(self.branch_head(project_root), self.head)
                self.assertIsNone(
                    self.namespace["object_record_path"](
                        project_root / "objects", tip
                    )
                )
        # The unperturbed upload still succeeds, so the predicates above are
        # detecting the injected condition and not the fixture.
        healthy_tip, healthy = self.commit([self.head], "coverage healthy")
        healthy_ledger = self.ledger(label="coverage healthy")
        status, _payload = self.upload(
            "coverage", healthy_tip, [healthy], ledger=healthy_ledger
        )
        self.assertEqual(status, 200)
        self.assertEqual(self.branch_head(project_root), healthy_tip)
        self.assert_closed(healthy_ledger)
        self.assertTrue(
            callable(
                getattr(owner_type, "_assert_prepared_coverage", None)
            )
        )

    def test_removing_the_layout_bootstrap_poll_creates_the_layout(self):
        """Mutation evidence for the distinct layout-bootstrap fence.

        Suppressing only the ``upload layout bootstrap`` poll, and leaving the
        publication-effect poll in place, changes behaviour: the durable project
        layout of a project that had none is created by an already-elapsed
        request.
        """
        real_writes = self.globals[
            "preflight_remote_json_publication_writes"
        ]
        real_layout = self.globals["ensure_remote_layout"]

        def run(app, suppress):
            clock = InjectedClock()
            deadline = self.deadline(clock)
            if suppress:
                real_check = deadline.check

                def lenient(phase):
                    if phase == "upload layout bootstrap":
                        suppressed.append(phase)
                        return
                    return real_check(phase)

                deadline.check = lenient

            def expiring_writes(*args, **kwargs):
                # Elapse the request after the final subject fence and before
                # the layout fence, so only the layout poll can stop it.
                result = real_writes(*args, **kwargs)
                clock.expire()
                return result

            layouts = []

            def watched_layout(path):
                layouts.append(path)
                return real_layout(path)

            self.globals[
                "preflight_remote_json_publication_writes"
            ] = expiring_writes
            self.globals["ensure_remote_layout"] = watched_layout
            ledger = self.ledger(label=f"layout {app}")
            outcome = None
            try:
                outcome = self.upload(
                    app,
                    self.head,
                    self.records,
                    ledger=ledger,
                    deadline=deadline,
                )
            except self.namespace["RequestDeadlineExceeded"] as exc:
                outcome = exc
            finally:
                self.globals[
                    "preflight_remote_json_publication_writes"
                ] = real_writes
                self.globals["ensure_remote_layout"] = real_layout
            return outcome, layouts, deadline, ledger

        suppressed = []
        outcome, layouts, deadline, ledger = run("layout-fenced", False)
        self.assertIsInstance(
            outcome, self.namespace["RequestDeadlineExceeded"]
        )
        self.assertEqual(deadline.expired_phase, "upload layout bootstrap")
        self.assertEqual(layouts, [])
        self.assertFalse(self.project_root("layout-fenced").exists())
        self.assert_closed(ledger)

        outcome, layouts, deadline, ledger = run("layout-mutated", True)
        # Behavioural difference: the layout of a project that had none now
        # exists, created by a request whose deadline had already elapsed.
        self.assertEqual(suppressed, ["upload layout bootstrap"])
        self.assertEqual(len(layouts), 1)
        self.assertTrue(self.project_root("layout-mutated").exists())
        self.assertTrue(
            (
                self.project_root("layout-mutated") / "remote_project.json"
            ).is_file()
        )
        self.assert_closed(ledger)


    # ------------------------------------ CFR-168 second review-closure repair

    def caller_owned_paths(self, objects, head, project_root):
        """The three public caller-owned entry points, as callables."""
        writes = [({"kind": "generation"}, None, {"current_generation": 7})]
        return (
            (
                "validate_sealed_commit",
                lambda ledger: self.namespace["validate_sealed_commit"](
                    objects, head, ledger=ledger
                ),
            ),
            (
                "validate_remote_json_publication_identity",
                lambda ledger: self.namespace[
                    "validate_remote_json_publication_identity"
                ](
                    "remote_upload",
                    "alice",
                    None,
                    [head],
                    objects_root=objects,
                    ledger=ledger,
                ),
            ),
            (
                "preflight_remote_json_publication",
                lambda ledger: self.namespace[
                    "preflight_remote_json_publication"
                ](
                    project_root,
                    "remote_upload",
                    "alice",
                    {"subject": "release"},
                    writes,
                    {"branch": "main", "head": head},
                    None,
                    [head],
                    validation_objects_root=objects,
                    validate_preimages=False,
                    ledger=ledger,
                ),
            ),
        )

    def observe_source_owners(self, owners, claims=None):
        """Capture each source memory owner and, optionally, its claims."""
        real = self.globals["_AccountedUploadSourceMemory"]

        class Observed(real):
            def __init__(inner, ledger):
                owners.append(inner)
                super().__init__(ledger)

            def _claim_control_memory(inner, dimension, amount, phase=None):
                if claims is not None:
                    claims.append((dimension, amount, phase))
                return super()._claim_control_memory(
                    dimension, amount, phase
                )

        self.globals["_AccountedUploadSourceMemory"] = Observed
        return lambda: self.globals.__setitem__(
            "_AccountedUploadSourceMemory", real
        )

    def test_control_release_is_retryable_and_exactly_once(self):
        """A failed control release must stay owed, not strand on the ledger.

        The reviewed candidate erased the claim, the amount and the
        closed/acquired owner state *before* the fallible
        ``DecisionWorkLedger.release``, so a pre-effect failure stranded 11,456
        retained bytes on each caller-owned entry point -- and the retry, seeing
        nothing owed, could not give them back.
        """
        objects = self.local_objects
        head = self.head
        for index in range(46):
            head = self.write_local_commit([head], f"release {index}")
        project_root = self.bootstrap("release")
        upper = self.namespace["SEALED_SESSION_CONTROL_UPPER"]

        for name, call in self.caller_owned_paths(objects, head, project_root):
            with self.subTest(path=name, dimension="retained_bytes"):
                owners = []
                restore = self.observe_source_owners(owners)
                ledger = self.ledger(label=f"{name} release fault")
                real_release = ledger.release
                fired = []

                def failing_release(dimension, amount):
                    if (
                        not fired
                        and dimension == "retained_bytes"
                        and amount == upper
                    ):
                        fired.append((dimension, amount))
                        raise RuntimeError("injected pre-effect release fault")
                    return real_release(dimension, amount)

                ledger.release = failing_release
                try:
                    with self.assertRaisesRegex(
                        RuntimeError, "pre-effect release fault"
                    ):
                        call(ledger)
                finally:
                    ledger.release = real_release
                    restore()
                self.assertEqual(fired, [("retained_bytes", upper)])
                self.assertEqual(len(owners), 1)
                owner = owners[0]
                # Pre-effect failure: the bytes are still on the ledger and the
                # owner still owns them, so the retry is meaningful.
                self.assertEqual(
                    ledger.current.get("retained_bytes"), upper
                )
                self.assertTrue(owner._acquired)
                self.assertFalse(owner._closed)
                owner.close()
                self.assertFalse(owner._acquired)
                self.assertTrue(owner._closed)
                self.assert_closed(ledger)
                # Exactly once: a further close releases nothing again.
                owner.close()
                self.assert_closed(ledger)

        # The same contract on the ordered-projection temporary claim.
        claims = []
        owners = []
        restore = self.observe_source_owners(owners, claims)
        probe = self.ledger(label="projection probe")
        try:
            self.namespace["validate_sealed_commit"](
                objects, head, ledger=probe
            )
        finally:
            restore()
        projection = next(
            amount
            for dimension, amount, _phase in claims
            if dimension == "temporary_bytes"
        )
        self.assertEqual(
            projection, self.ordered_projection_upper((projection - 56) // 8)
        )
        self.assert_closed(probe)

        owners = []
        restore = self.observe_source_owners(owners)
        ledger = self.ledger(label="projection release fault")
        real_release = ledger.release
        fired = []

        def failing_projection_release(dimension, amount):
            if (
                not fired
                and dimension == "temporary_bytes"
                and amount == projection
            ):
                fired.append((dimension, amount))
                raise RuntimeError("injected pre-effect release fault")
            return real_release(dimension, amount)

        ledger.release = failing_projection_release
        try:
            with self.assertRaisesRegex(
                RuntimeError, "pre-effect release fault"
            ):
                self.namespace["validate_sealed_commit"](
                    objects, head, ledger=ledger
                )
        finally:
            ledger.release = real_release
            restore()
        self.assertEqual(fired, [("temporary_bytes", projection)])
        # The session's own cleanup retried the still-owed release, so no
        # temporary debt survives and no permit is left open.
        for owner in owners:
            owner.close()
        self.assert_closed(ledger)

    def test_ordered_projection_allocation_requires_a_proved_charge(self):
        """The list is allocated only after its charge is proved on the ledger.

        Both required mutations -- removing the projection charge, and charging
        it somewhere the caller's ledger never sees -- must fail *before*
        ``list(self.documents)`` runs, not after.
        """
        objects = self.local_objects
        head = self.head
        for index in range(30):
            head = self.write_local_commit([head], f"boundary {index}")
        session_type = self.globals["SealedCommitValidationSession"]
        source_type = self.globals["_AccountedUploadSourceMemory"]

        def run(label, patch):
            """Run one caller-owned validation with ``patch`` applied.

            ``list(self.documents)`` is the projection allocation, and it is the
            only place this route iterates the memo itself, so a recording
            ``__iter__`` observes exactly that allocation and nothing else.
            """
            allocated = []

            class WatchedDocuments(dict):
                def __iter__(inner):
                    allocated.append(len(inner))
                    return super().__iter__()

            real_init = session_type.__init__

            def watched_init(inner, *args, **kwargs):
                real_init(inner, *args, **kwargs)
                inner.documents = WatchedDocuments()

            ledger = self.ledger(label=label)
            undo = patch()
            session_type.__init__ = watched_init
            try:
                self.namespace["validate_sealed_commit"](
                    objects, head, ledger=ledger
                )
                outcome = None
            except BaseException as exc:
                outcome = exc
            finally:
                session_type.__init__ = real_init
                undo()
            return outcome, allocated, ledger

        # Healthy: the charge is proved and the projection really is allocated.
        outcome, allocated, ledger = run("boundary healthy", lambda: (lambda: None))
        self.assertIsNone(outcome)
        self.assertEqual(len(allocated), 1)
        documents = allocated[0]
        self.assertEqual(
            self.ordered_projection_upper(documents),
            56 + 8 * documents,
        )
        self.assert_closed(ledger)

        # Mutation shape 1: the projection charge is removed.
        def drop_charge():
            real_acquire = session_type._acquire_control

            def without_projection(inner, dimension, amount, phase):
                if (
                    dimension == "temporary_bytes"
                    and phase == "snapshot_revalidation"
                ):
                    return None
                return real_acquire(inner, dimension, amount, phase)

            session_type._acquire_control = without_projection
            return lambda: setattr(
                session_type, "_acquire_control", real_acquire
            )

        outcome, allocated, ledger = run("boundary uncharged", drop_charge)
        self.assertIsInstance(outcome, self.namespace["CFError"])
        self.assertIn(
            "ordered object-id projection is not charged before allocation",
            str(outcome),
        )
        self.assertEqual(allocated, [])
        self.assert_closed(ledger)

        # Mutation shape 2: the charge never reaches the caller's ledger.
        def drop_exact_dimension():
            real_exact = source_type.EXACT_DIMENSIONS
            source_type.EXACT_DIMENSIONS = frozenset()
            return lambda: setattr(
                source_type, "EXACT_DIMENSIONS", real_exact
            )

        outcome, allocated, ledger = run("boundary unwitnessed", drop_exact_dimension)
        self.assertIsInstance(outcome, self.namespace["CFError"])
        self.assertIn(
            "ordered object-id projection is not charged before allocation",
            str(outcome),
        )
        self.assertEqual(allocated, [])
        # The boundary refusal is the primary error, not a cleanup fault raised
        # after it: this mutation charges the wrong pool, so its own owner is
        # left in debt, and that must not replace the decision above.


    # ------------------------------------- CFR-168 third review-closure repair

    def failing_close_syscall(self, armed):
        """Install a real ``close(2)``-shaped failure: the fd closes, the call raises.

        ``armed`` is a mutable flag the caller raises while the region under
        test is on the stack, so the fault lands inside commit-source cleanup
        rather than at an unrelated descriptor.  No ledger call is patched.
        """
        real_close = self.globals["_CODEFIRE_OS_CLOSE"]
        fired = []

        def flaky_close(file_descriptor):
            real_close(file_descriptor)
            if armed and not fired:
                fired.append(file_descriptor)
                raise OSError(5, "injected close(2) failure")

        self.globals["_CODEFIRE_OS_CLOSE"] = flaky_close
        return fired, lambda: self.globals.__setitem__(
            "_CODEFIRE_OS_CLOSE", real_close
        )

    def arm_during_source_close(self, armed, calls=None):
        """Raise ``armed`` only while ``_AccountedUploadCommitSource.close`` runs.

        When ``calls`` is supplied every call also appends the source it was
        made on, so a route can prove *how many times each individual source*
        was closed rather than only that the ledger ended at zero.  Those are
        strong references: while ``calls`` is alive no source on the route is
        collectible, which removes both finalizers from the picture entirely
        and leaves the synchronous retry as the only thing that can have
        drained the ledger.
        """
        source_type = self.globals["_AccountedUploadCommitSource"]
        real_close = source_type.close

        def armed_close(inner):
            if calls is not None:
                calls.append(inner)
            armed.append(True)
            try:
                return real_close(inner)
            finally:
                armed.pop()

        source_type.close = armed_close
        return lambda: setattr(source_type, "close", real_close)

    def close_calls_per_source(self, calls):
        """Close-call counts per distinct source, in first-call order."""
        counted = []
        for source in calls:
            for entry in counted:
                if entry[0] is source:
                    entry[1] += 1
                    break
            else:
                counted.append([source, 1])
        return [entry[1] for entry in counted]

    def assert_one_bounded_retry(self, calls):
        """Exactly one source was closed twice; every other one exactly once.

        This is what bounds the repair.  ``assert_closed`` alone is satisfied by
        any number of retries, so without this a build that looped until the
        teardown succeeded -- or simply retried twice -- would still pass.
        """
        counts = self.close_calls_per_source(calls)
        self.assertEqual(
            sorted(counts),
            sorted([2] + [1] * (len(counts) - 1)),
            f"close calls per source: {counts}",
        )

    def expire_at_first_commit_borrow(self, clock, attribute):
        """Elapse the request deadline inside the first charged commit borrow.

        The borrow succeeds, so the session is genuinely open and holds a real
        descriptor; the traversal's *next* poll then raises the production
        ``RequestDeadlineExceeded``.  That gives a real operation decision error
        arriving before the cleanup, which is the ordering the precedence rule
        is about.
        """
        source_type = self.globals["_AccountedUploadCommitSource"]
        real = getattr(source_type, attribute)
        fired = []

        def borrowing(inner, *args, **kwargs):
            result = real(inner, *args, **kwargs)
            if not fired:
                fired.append(True)
                clock.expire()
            return result

        setattr(source_type, attribute, borrowing)
        return fired, lambda: setattr(source_type, attribute, real)

    def exchange_upload_request(self, app, head, records, arm):
        """Drive one real socket upload through ``make_http_handler``.

        Returns ``(response_bytes, observation, handler_errors)`` where
        ``observation`` is the request loop's own ``_decision_work_observer``
        snapshot -- the state production itself reads in ``handle_one_request``'s
        ``finally`` to decide ``close_connection`` and whether to ``terminate``.
        ``arm`` installs the fault and returns its restore callable; it runs
        with the server already listening so the fault is live for the request
        and nothing else.
        """
        observations = []
        handler = self.namespace["make_http_handler"](
            self.storage_root,
            self.namespace["DecisionWorkProfile"].http_v1(),
            _decision_work_observer=observations.append,
        )
        server = self.namespace["ReusableThreadingHTTPServer"](
            ("127.0.0.1", 0), handler
        )
        # The request-loop error escapes ``handle_one_request`` *after* the
        # observer ran, so capture it instead of letting socketserver print it.
        handler_errors = []
        server.handle_error = lambda request, address: handler_errors.append(
            sys.exc_info()[1]
        )
        body = json.dumps(
            {"branch": "main", "head": head, "objects": records},
            separators=(",", ":"),
        ).encode("utf-8")
        request = (
            f"POST /v1/projects/org/{app}/branches/main/upload HTTP/1.0\r\n"
            f"Content-Length: {len(body)}\r\n\r\n"
        ).encode("utf-8") + body
        thread = threading.Thread(target=server.handle_request, daemon=True)
        gc.collect()
        restore = arm()
        chunks = []
        try:
            thread.start()
            with socket.create_connection(
                server.server_address, timeout=10
            ) as client:
                client.settimeout(10)
                client.sendall(request)
                client.shutdown(socket.SHUT_WR)
                while True:
                    chunk = client.recv(64 * 1024)
                    if not chunk:
                        break
                    chunks.append(chunk)
        finally:
            server.server_close()
            thread.join(timeout=10)
            restore()
        self.assertFalse(thread.is_alive(), "the HTTP handler did not terminate")
        self.assertEqual(len(observations), 1)
        return b"".join(chunks), observations[0], handler_errors

    def record_deadline_errors(self):
        """Record every ``RequestDeadlineExceeded`` production actually raises.

        Lets a route prove the caller received the *same exception object* the
        traversal raised, rather than an equal-looking replacement.
        """
        deadline_type = self.globals["HTTPRequestDeadline"]
        real_check = deadline_type.check
        raised = []

        def checked(inner, *args, **kwargs):
            try:
                return real_check(inner, *args, **kwargs)
            except BaseException as error:
                raised.append(error)
                raise

        deadline_type.check = checked
        return raised, lambda: setattr(deadline_type, "check", real_check)

    def observe_live_close_fault(
        self,
        app,
        tip,
        records,
        ledger,
        *,
        deadline=None,
        calls=None,
        expected=OSError,
    ):
        """Read the request ledger with the raised error still in flight.

        ``unittest.assertRaises`` stores ``exc.with_traceback(None)``.  That
        drops the frames holding the commit source, so a finalizer can settle
        the ledger before any assertion runs and the oracle measures a state
        production is never in.  Production keeps the traceback: the ``OSError``
        carries it out of ``handle_http_upload`` and the request loop accounts
        the request while it still pins the traversal frame that owns the
        source.  This observes exactly that -- inside the live exception's own
        handler, traceback retained -- so the assertions are about production's
        checkpoint rather than about the oracle.
        """
        # Finalize anything still garbage *before* the injector is installed,
        # so the single fault this route is allowed can only land in the source
        # cleanup under test and never in some earlier owner's finalizer.
        gc.collect()
        armed = []
        fired, restore_close = self.failing_close_syscall(armed)
        restore_arm = self.arm_during_source_close(armed, calls)
        observed = None
        try:
            self.upload(app, tip, records, ledger=ledger, deadline=deadline)
        except expected as error:
            observed = {
                # Held deliberately: keeping the error keeps its traceback,
                # and the traceback is what pins the source production could
                # not collect.  Every assertion below runs under that.
                "error": error,
                "cause": error.__cause__,
                "traceback_retained": error.__traceback__ is not None,
                "current": dict(ledger.current),
                "open_permits": ledger.has_open_permits(),
                "terminal_reason": ledger.terminal_reason,
            }
        finally:
            restore_arm()
            restore_close()
        self.assertIsNotNone(
            observed, f"the live route did not raise {expected.__name__}"
        )
        self.assertEqual(len(fired), 1)
        return observed

    def test_commit_source_cleanup_survives_a_failing_close_syscall(self):
        """A failing ``close(2)`` must not strand the request reservation.

        A predecessor of this route nulled both retry handles before the
        fallible cleanup, so one real ``close(2)`` failure -- with no ledger
        fault injected anywhere -- permanently stranded 11,456 retained bytes on
        the live request ledger and made ``source.close()`` a silent no-op.
        Keeping the handles made the wrapper retryable, but the live routes
        never retried, so the debt was still there at production's own
        checkpoint; only the synchronous caller retry drains it there.

        Two deliberate reversals, one per direction.  Remove the one-shot retry
        from either live call site and the faulting source is closed once
        instead of twice, leaving ``retained_bytes`` at 11,456 in
        ``observed["current"]`` -- the raised error's traceback keeps the source
        alive, so no finalizer can run.  Retry twice instead of once and the
        ledger still reaches zero, but ``assert_one_bounded_retry`` fails: that
        assertion is what pins the *bound*, which zero-debt alone never can.
        """
        upper = self.namespace["SEALED_SESSION_CONTROL_UPPER"]
        signature_policy = {
            "required": True,
            "require_history": True,
            "keys": {"alice": "s"},
        }

        def signed_history():
            parent = self.head
            records = []
            for index in range(3):
                parent, record = self.commit(
                    [parent], f"close-signed-{index}", signer="alice", key="s"
                )
                records.append(record)
            return parent, records

        def fast_forward():
            tip, child = self.commit([self.head], "close ancestry child")
            return tip, [child]

        routes = (
            ("ancestry", None, fast_forward),
            ("signature-history", signature_policy, signed_history),
        )
        for name, policy, build in routes:
            with self.subTest(route=name):
                app = f"close2-{name}"
                project_root = self.bootstrap(app)
                if policy is not None:
                    self.write_signature_policy(project_root, policy)
                tip, records = build()
                before = self.authority_snapshot(project_root)
                before_durable = self.durable_snapshot(project_root)
                ledger = self.ledger(label=f"close2 {name}")
                calls = []
                observed = self.observe_live_close_fault(
                    app, tip, records, ledger, calls=calls
                )
                # The failure really came from the descriptor close, not a
                # patched ledger, and it surfaced instead of being swallowed.
                self.assertEqual(observed["error"].errno, 5)
                # With no operation error, the cleanup error is the whole
                # story: it is the outer exception and it has no cause.
                self.assertIsNone(observed["cause"])
                # Exactly two calls on the source that faulted -- the initial
                # close plus one retry -- and exactly one on any other source
                # the request built.  ``calls`` holds every source, so nothing
                # here could have been settled by a finalizer.
                self.assert_one_bounded_retry(calls)
                # The observation is production's state, not the oracle's: the
                # error still carries the traceback that pins the source, so no
                # finalizer can be what settled the ledger below.
                self.assertTrue(observed["traceback_retained"])
                # Zero retained *and* temporary debt, zero permits, measured
                # where the request loop accounts the request.
                self.assertEqual(
                    {
                        key: value
                        for key, value in observed["current"].items()
                        if value
                    },
                    {},
                )
                self.assertFalse(observed["open_permits"])
                # Terminal precedence is unchanged: the failing route neither
                # terminates the ledger itself nor leaves a stale reason.
                self.assertIsNone(observed["terminal_reason"])
                # No durable effect, and the ledger stays settled afterwards.
                self.assert_closed(ledger)
                self.assertEqual(
                    self.authority_snapshot(project_root), before
                )
                self.assertEqual(
                    self.durable_snapshot(project_root), before_durable
                )
                self.assertEqual(self.branch_head(project_root), self.head)

        # The wrapper keeps both handles until their cleanup succeeds, so the
        # retry finishes the release instead of returning over stranded bytes.
        source_type = self.globals["_AccountedUploadCommitSource"]
        ledger = self.ledger(label="close2 wrapper")
        source = source_type(
            self.local_objects, ledger, None, "upload ancestry"
        )
        source.payload(self.head)
        self.assertGreaterEqual(ledger.current.get("retained_bytes"), upper)
        armed = [True]
        fired, restore_close = self.failing_close_syscall(armed)
        try:
            with self.assertRaises(OSError):
                source.close()
        finally:
            restore_close()
        self.assertEqual(len(fired), 1)
        self.assertIsNotNone(source._session)
        self.assertIsNotNone(source._memory)
        self.assertGreaterEqual(ledger.current.get("retained_bytes"), upper)
        self.assertIsNone(source.close())
        self.assert_closed(ledger)
        self.assertIsNone(source._session)
        self.assertIsNone(source._memory)
        # Idempotent: a further close releases nothing again.
        self.assertIsNone(source.close())
        self.assert_closed(ledger)

    def test_request_loop_accounts_a_drained_cleanup_fault(self):
        """The outer production checkpoint: a real socket request, observed.

        ``observe_live_close_fault`` reads the *inner* checkpoint -- inside the
        raised error's own handler, before the request loop has accounted
        anything -- so it necessarily sees ``terminal_reason`` still ``None``.
        This proves the outer one.  A real request goes over a real socket to a
        real ``make_http_handler`` server, and the assertion reads
        ``_decision_work_observer``: the same snapshot
        ``handle_one_request``'s ``finally`` uses to decide ``close_connection``
        and whether to ``terminate``.

        That accounting is exactly what the third-repair review measured as
        ``retained_bytes = 11,456``, ``open_permit_count = 0``,
        ``terminal_reason = None``.  With the debt drained the loop reaches its
        ``terminate`` arm instead, so the request is accounted ``complete``.
        ``complete`` describes the *accounting*, not the request: the upload
        still failed, the ``OSError`` still escapes the loop, and the client
        still gets no response at all.

        Deliberate reversal: removing the retry from either live call site
        restores ``retained_bytes = 11,456`` and ``terminal_reason = None``
        here, which is the exact triple the review held ``HIGH-2`` open on.
        """
        signature_policy = {
            "required": True,
            "require_history": True,
            "keys": {"alice": "s"},
        }

        def signed_history():
            return self.signed_chain(self.head, 3, "loop-signed", "s")

        def fast_forward():
            return self.chain(self.head, 3, "loop-ancestry")

        routes = (
            ("ancestry", None, fast_forward),
            ("signature-history", signature_policy, signed_history),
        )
        for name, policy, build in routes:
            with self.subTest(route=name):
                app = f"loop-{name}"
                project_root = self.bootstrap(app)
                if policy is not None:
                    self.write_signature_policy(project_root, policy)
                tip, records = build()
                before_durable = self.durable_snapshot(project_root)
                armed = []
                calls = []
                state = {}

                def arm():
                    fired, restore_close = self.failing_close_syscall(armed)
                    restore_arm = self.arm_during_source_close(armed, calls)
                    state["fired"] = fired

                    def restore():
                        restore_arm()
                        restore_close()

                    return restore

                response, observation, errors = self.exchange_upload_request(
                    app, tip, records, arm
                )
                snapshot = observation["snapshot"]
                # Exactly one real close(2) failure, and it escaped the request
                # loop after the loop had accounted the request.
                self.assertEqual(len(state["fired"]), 1)
                self.assertEqual(len(errors), 1)
                self.assertIsInstance(errors[0], OSError)
                self.assertEqual(errors[0].errno, 5)
                # The request loop's own view: no debt, no permit, and the
                # accounting closed rather than being left open by the debt.
                self.assertEqual(
                    {
                        key: value
                        for key, value in snapshot["current"].items()
                        if value
                    },
                    {},
                )
                self.assertEqual(observation["open_permit_count"], 0)
                self.assertEqual(snapshot["terminal_reason"], "complete")
                self.assertIsNone(observation["budget_diagnostic"])
                self.assertIsNone(observation["deadline_diagnostic"])
                # Still exactly one retry, and still nothing published: no
                # response bytes, and no durable byte moved.
                self.assert_one_bounded_retry(calls)
                self.assertEqual(response, b"")
                self.assertEqual(
                    self.durable_snapshot(project_root), before_durable
                )
                self.assertEqual(self.branch_head(project_root), self.head)

    def test_live_cleanup_fault_never_displaces_the_operation_decision(self):
        """Operation failure and cleanup failure at once, on both live routes.

        Shared-primitives ``C14``/``B2``: a cleanup fault must never displace
        the primary operation decision.  Both live routes previously did
        displace it -- ``raise cleanup_error from run_error`` handed the caller
        a descriptor ``OSError`` in place of the ancestry or signature decision
        it asked for -- and this closes that inversion rather than preserving
        it.

        A real ``RequestDeadlineExceeded`` is raised mid-traversal, after the
        source opened a real descriptor, and the one-shot ``close(2)`` fault
        then fires in the cleanup that follows.  What the caller must see is the
        *operation decision itself*, the same exception object the traversal
        raised, with the first cleanup error attached as its cause and the
        ledger already drained by the bounded retry.

        Deliberate reversals: restoring ``raise cleanup_error from run_error``
        at either call site fails here, because the outer error becomes the
        ``OSError``.  Dropping the ``from cleanup_error`` link fails too, since
        the cleanup fault would then be silently discarded.
        """
        deadline_type = self.namespace["RequestDeadlineExceeded"]
        signature_policy = {
            "required": True,
            "require_history": True,
            "keys": {"alice": "s"},
        }

        def signed_history():
            return self.signed_chain(self.head, 4, "cause-signed", "s")

        def fast_forward():
            return self.chain(self.head, 4, "cause-ancestry")

        routes = (
            ("ancestry", None, fast_forward, "payload", "upload ancestry"),
            (
                "signature-history",
                signature_policy,
                signed_history,
                "signature_payload",
                "commit signature history load",
            ),
        )
        for name, policy, build, borrow, phase in routes:
            with self.subTest(route=name):
                app = f"cause-{name}"
                project_root = self.bootstrap(app)
                if policy is not None:
                    self.write_signature_policy(project_root, policy)
                tip, records = build()
                before_durable = self.durable_snapshot(project_root)
                clock = InjectedClock()
                deadline = self.deadline(clock)
                ledger = self.ledger(label=f"cause {name}")
                calls = []
                expired, restore_borrow = self.expire_at_first_commit_borrow(
                    clock, borrow
                )
                raised, restore_record = self.record_deadline_errors()
                try:
                    observed = self.observe_live_close_fault(
                        app,
                        tip,
                        records,
                        ledger,
                        deadline=deadline,
                        calls=calls,
                        expected=deadline_type,
                    )
                finally:
                    restore_record()
                    restore_borrow()
                # Both faults really happened: one elapsed deadline, one real
                # descriptor close failure.
                self.assertEqual(len(expired), 1)
                # C14: the operation decision is the outer exception, by type,
                # by message and by object identity -- the very exception the
                # traversal's own deadline poll raised, not a replacement.
                self.assertIsInstance(observed["error"], deadline_type)
                self.assertIn("100 ms", str(observed["error"]))
                self.assertGreaterEqual(len(raised), 1)
                self.assertIs(observed["error"], raised[0])
                self.assertEqual(deadline.expired_phase, phase)
                # The cleanup fault is preserved as the cause, not discarded.
                self.assertIsInstance(observed["cause"], OSError)
                self.assertEqual(observed["cause"].errno, 5)
                # The retry is still exactly one, and still drains everything.
                self.assert_one_bounded_retry(calls)
                self.assertTrue(observed["traceback_retained"])
                self.assertEqual(
                    {
                        key: value
                        for key, value in observed["current"].items()
                        if value
                    },
                    {},
                )
                self.assertFalse(observed["open_permits"])
                self.assertIsNone(observed["terminal_reason"])
                self.assert_closed(ledger)
                # Nothing durable moved on the doubly failing path.
                self.assertEqual(
                    self.durable_snapshot(project_root), before_durable
                )
                self.assertEqual(self.branch_head(project_root), self.head)

    def test_commit_source_finalizer_settles_an_abandoned_retry(self):
        """Deliberate reversal for ``_AccountedUploadCommitSource.__del__``.

        The synchronous caller retry owns the live routes; this pins the
        fallback behind it.  A caller that drops the source without ever
        retrying, and without holding the error, has only the finalizer left,
        and only the *source*'s finalizer can settle it: the session still owes
        its control debt, and the session is what keeps
        ``_AccountedUploadSourceMemory`` reachable through ``control_owner``, so
        the memory owner is not even collectible while that debt exists.
        Deleting ``_AccountedUploadCommitSource.__del__`` must therefore fail
        this test with the full reservation stranded.

        Holding the session over the drop is what makes the reversal
        deterministic rather than a race between two finalizers, and it is the
        real shape of the hazard: the session outlives the wrapper.  The
        constructor-unwind case is not this reversal -- there ``_session`` is
        never built, so the memory owner's own finalizer suffices.
        """
        source_type = self.globals["_AccountedUploadCommitSource"]
        upper = self.namespace["SEALED_SESSION_CONTROL_UPPER"]
        ledger = self.ledger(label="close2 abandoned retry")
        source = source_type(
            self.local_objects, ledger, None, "upload ancestry"
        )
        source.payload(self.head)
        self.assertGreaterEqual(ledger.current.get("retained_bytes"), upper)
        armed = [True]
        fired, restore_close = self.failing_close_syscall(armed)
        try:
            # ``assertRaises`` strips the traceback here deliberately: this is
            # the abandoned-error path, the one case the finalizer can reach.
            with self.assertRaises(OSError):
                source.close()
        finally:
            restore_close()
        self.assertEqual(len(fired), 1)
        # The session teardown is still owed and the caller walks away from it.
        session = source._session
        self.assertIsNotNone(session)
        self.assertIsNotNone(source._memory)
        self.assertGreaterEqual(ledger.current.get("retained_bytes"), upper)
        del source
        gc.collect()
        self.assert_closed(ledger)
        # The retained session was really the live one, and it is now closed:
        # the fallback drained it rather than abandoning an open handle.
        self.assertTrue(session.closed)
        del session
        gc.collect()
        self.assert_closed(ledger)

    def test_commit_source_constructor_unwind_keeps_the_primary_error(self):
        """A cleanup fault must not replace the constructor's own decision."""
        source_type = self.globals["_AccountedUploadCommitSource"]
        session_type = self.globals["SealedCommitValidationSession"]
        memory_type = self.globals["_AccountedUploadSourceMemory"]

        class FailingSession(session_type):
            def __init__(inner, *args, **kwargs):
                super().__init__(*args, **kwargs)
                raise ValueError("PRIMARY constructor decision")

        one_shot = []

        class OnceFailingMemory(memory_type):
            def close(inner):
                if not one_shot:
                    one_shot.append(True)
                    raise RuntimeError("SECONDARY cleanup fault")
                return memory_type.close(inner)

        def unwind(label):
            ledger = self.ledger(label=label)
            source = source_type(
                self.local_objects, ledger, None, "upload ancestry"
            )
            one_shot.clear()
            self.globals["SealedCommitValidationSession"] = FailingSession
            self.globals["_AccountedUploadSourceMemory"] = OnceFailingMemory
            try:
                with self.assertRaises(ValueError) as caught:
                    source.payload(self.head)
            finally:
                self.globals[
                    "SealedCommitValidationSession"
                ] = session_type
                self.globals[
                    "_AccountedUploadSourceMemory"
                ] = memory_type
            # The constructor's decision survives the cleanup fault.
            self.assertIn("PRIMARY constructor decision", str(caught.exception))
            self.assertEqual(len(one_shot), 1)
            return source, ledger

        # Explicit retry through the surface the caller already holds.
        source, ledger = unwind("ctor unwind retry")
        self.assertIsNotNone(source._memory)
        self.assertGreater(ledger.current.get("retained_bytes"), 0)
        source.close()
        self.assert_closed(ledger)

        # Dropped without a retry: the idempotent finalizer still settles it.
        source, ledger = unwind("ctor unwind finalizer")
        self.assertGreater(ledger.current.get("retained_bytes"), 0)
        del source
        gc.collect()
        self.assert_closed(ledger)

        # The same precedence in ``validate_sealed_commit``'s constructor unwind.
        one_shot.clear()
        ledger = self.ledger(label="sealed ctor unwind")
        self.globals["SealedCommitValidationSession"] = FailingSession
        self.globals["_AccountedUploadSourceMemory"] = OnceFailingMemory
        try:
            with self.assertRaises(ValueError) as caught:
                self.namespace["validate_sealed_commit"](
                    self.local_objects, self.head, ledger=ledger
                )
        finally:
            self.globals["SealedCommitValidationSession"] = session_type
            self.globals["_AccountedUploadSourceMemory"] = memory_type
        self.assertIn("PRIMARY constructor decision", str(caught.exception))
        self.assertEqual(len(one_shot), 1)
        gc.collect()
        self.assert_closed(ledger)

    def test_c18_cleanup_origin_joins_the_one_shared_error_taxonomy(self):
        """C18: the composed cleanup origin is decided by the owner's mark.

        The error-origin taxonomy arrives only with the MR-list input and the
        synchronous commit-source drain arrives only with the Upload input, so
        no accepted tree ever executed one against the other.  This is the
        direct cross-input oracle for the composed contract: three exhaustive
        origins on one shared taxonomy, the cleanup origin marked at the drain
        owner, refused by the route classifier exactly as transport is, and
        decided by mark identity rather than errno, message or exception class.
        """
        drain = self.namespace["_drained_commit_source_cleanup_error"]
        origin = self.namespace["os_error_origin"]
        classify = self.namespace["classify_route_os_error"]
        mark_transport = self.namespace["mark_transport_origin"]

        # Every probe below is built from exactly these bytes, so errno,
        # message, filename and class are constant across the whole oracle and
        # cannot carry the decision.  5 is EIO.
        def build():
            return OSError(5, "commit source teardown refused", "/objects/pack")

        produced, pristine = build(), build()
        transport, filesystem = build(), build()
        for other in (pristine, transport, filesystem):
            self.assertIs(type(produced), type(other))
            self.assertEqual(produced.errno, other.errno)
            self.assertEqual(str(produced), str(other))
            self.assertEqual(produced.filename, other.filename)

        # Only the production drain marks, and only at the owner.
        closes = []

        def close():
            closes.append(None)
            if len(closes) == 1:
                raise produced

        self.assertIs(drain(close, lambda exc: None), produced)
        self.assertEqual(len(closes), 2)

        # 1. The taxonomy is exhaustive over the three origins.
        mark_transport(transport)
        self.assertIsNotNone(classify(filesystem))
        self.assertEqual(origin(produced), "commit-cleanup")
        self.assertEqual(origin(transport), "transport")
        self.assertEqual(origin(filesystem), "filesystem")
        self.assertEqual(origin(build()), "unclassified")
        self.assertEqual(
            {origin(produced), origin(transport), origin(filesystem)},
            {"commit-cleanup", "transport", "filesystem"},
        )

        # 2/3. The classifier refuses the cleanup origin and still types an
        # otherwise byte-for-byte identical unmarked error.  Identity of the
        # production mark is the only difference between these two calls.
        self.assertIsNone(classify(produced))
        self.assertIsNotNone(classify(pristine))

        # 4. C14 precedence is untouched: a cleanup error carried as the cause
        # of a failed operation decision never reclassifies the outer decision.
        outer = build()
        try:
            try:
                raise produced
            except OSError as cause:
                raise outer from cause
        except OSError as raised:
            self.assertIs(raised, outer)
            self.assertIs(raised.__cause__, produced)
            self.assertIsNotNone(classify(raised))
            self.assertEqual(origin(raised), "filesystem")
            self.assertEqual(origin(raised.__cause__), "commit-cleanup")


if __name__ == "__main__":
    unittest.main()
