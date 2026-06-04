import json
import hashlib
import hmac
import os
import shutil
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"
INSTALL = ROOT / "install.sh"


class CodeFireCliTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.processes = []

    def tearDown(self):
        for proc in self.processes:
            if proc.poll() is None:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)
            if proc.stdout:
                proc.stdout.close()
            if proc.stderr:
                proc.stderr.close()
        shutil.rmtree(self.tmp)

    def run_cf(self, *args, cwd=None, check=True, env=None):
        proc_env = os.environ.copy()
        if env:
            proc_env.update(env)
        proc = subprocess.run(
            [str(CODEFIRE), *args],
            cwd=cwd or self.tmp,
            env=proc_env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if check and proc.returncode != 0:
            self.fail(f"codefire {' '.join(args)} failed\nstdout={proc.stdout}\nstderr={proc.stderr}")
        return proc

    def object_digest(self, type_tag, payload):
        data = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
        h = hashlib.sha256()
        h.update(type_tag.encode("utf-8"))
        h.update(b"\0")
        h.update(data)
        return h.hexdigest()

    def remote_object_id(self, type_tag, payload):
        digest = self.object_digest(type_tag, payload)
        prefixes = {"commit": "CF-COMMIT", "policy": "CF-POLICY"}
        return f"{prefixes.get(type_tag, 'CF-OBJECT')}-{digest[:12]}"

    def write_demo_files(self, open_dir: Path, minutes: int = 30):
        (open_dir / "docs/spec").mkdir(parents=True, exist_ok=True)
        (open_dir / "docs/design").mkdir(parents=True, exist_ok=True)
        (open_dir / "src").mkdir(exist_ok=True)
        (open_dir / "tests").mkdir(exist_ok=True)
        (open_dir / "docs/spec/auth.md").write_text(
            "## REQ-AUTH-001: Session expiration\n\n"
            f"Sessions expire after {minutes} minutes.\n",
            encoding="utf-8",
        )
        (open_dir / "docs/design/auth.md").write_text(
            "## DES-AUTH-001: Session policy\n\n"
            f"SessionPolicy returns {minutes} minutes.\n",
            encoding="utf-8",
        )
        (open_dir / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            f"        return {minutes}\n",
            encoding="utf-8",
        )
        (open_dir / "tests/test_auth.py").write_text(
            "from src.auth import SessionPolicy\n\n"
            "# cf-atom: TEST-session-expiration\n"
            "def test_session_expiration():\n"
            f"    assert SessionPolicy().expires_after_minutes() == {minutes}\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - path: docs/spec/**/*.md\n"
            "      kind: requirement_document\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: REQ-AUTH-001\n"
            "    to: DES-AUTH-001\n"
            "    type: refined_by\n"
            "  - from: DES-AUTH-001\n"
            "    to: CODE-SessionPolicy\n"
            "    type: implemented_by\n"
            "  - from: REQ-AUTH-001\n"
            "    to: TEST-session-expiration\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        (open_dir / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: true\n"
            "verification:\n"
            "  required:\n"
            "    - id: unit-tests\n"
            "      command: \"python3 -m unittest discover -s tests\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )

    def commit_demo_version(self, open_dir: Path, minutes: int, message: str):
        self.write_demo_files(open_dir, minutes)
        scan = self.run_cf("scan", cwd=open_dir)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", message, cwd=open_dir)
        self.run_cf("verify", cwd=open_dir)
        self.run_cf("commit", "-m", message, cwd=open_dir)

    def generate_self_signed_cert(self):
        cert = self.tmp / "codefire-test.crt"
        key = self.tmp / "codefire-test.key"
        proc = subprocess.run(
            [
                "openssl",
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-keyout",
                str(key),
                "-out",
                str(cert),
                "-days",
                "1",
                "-subj",
                "/CN=127.0.0.1",
                "-addext",
                "subjectAltName=IP:127.0.0.1",
            ],
            cwd=self.tmp,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return cert, key

    def start_http_remote(self, tls=False):
        cmd = [str(CODEFIRE), "serve", str(self.tmp / ("https-storage" if tls else "http-storage")), "--host", "127.0.0.1", "--port", "0"]
        marker = "cf+http://"
        if tls:
            cert, key = self.generate_self_signed_cert()
            cmd.extend(["--tls-cert", str(cert), "--tls-key", str(key)])
            marker = "cf+https://"
        proc = subprocess.Popen(
            cmd,
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.processes.append(proc)
        line = proc.stdout.readline()
        if proc.poll() is not None:
            self.fail(f"codefire serve exited early\nstdout={line}\nstderr={proc.stderr.read()}")
        if marker not in line:
            self.fail(f"unexpected codefire serve output: {line}")
        return line[line.index(marker) :].strip()

    def replace_main_head_with_mutated_commit(self, mutate):
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        mutate(bad_payload)
        bad_commit_id = self.remote_object_id("commit", bad_payload)
        bad_record = {
            "object_id": bad_commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", bad_payload),
            "payload": bad_payload,
        }
        bad_path = self.tmp / ".codefire/objects/commits" / f"{bad_commit_id}.json"
        bad_path.write_text(json.dumps(bad_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return bad_commit_id, bad_payload

    def write_commit_object(self, payload):
        commit_id = self.remote_object_id("commit", payload)
        record = {
            "object_id": commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", payload),
            "payload": payload,
        }
        path = self.tmp / ".codefire/objects/commits" / f"{commit_id}.json"
        path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return commit_id

    def write_commit_object_to(self, objects_root: Path, payload):
        commit_id = self.remote_object_id("commit", payload)
        record = {
            "object_id": commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", payload),
            "payload": payload,
        }
        path = objects_root / "commits" / f"{commit_id}.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return commit_id

    def object_record_for(self, objects_root: Path, oid: str):
        matches = list(objects_root.glob(f"*/*{oid}.json"))
        self.assertEqual(len(matches), 1, oid)
        record = json.loads(matches[0].read_text(encoding="utf-8"))
        return {
            "object_id": record["object_id"],
            "type": record["type"],
            "hash": record["hash"],
            "payload": record["payload"],
        }

    def reachable_object_records(self, objects_root: Path, root_oid: str):
        records = []
        seen = set()
        stack = [root_oid]
        while stack:
            oid = stack.pop()
            if oid in seen:
                continue
            seen.add(oid)
            record = self.object_record_for(objects_root, oid)
            records.append(record)
            payload = record["payload"]
            if payload.get("type") == "commit":
                stack.extend(parent for parent in payload.get("parents", []) if isinstance(parent, str))
                stack.extend(value for value in payload.get("roots", {}).values() if isinstance(value, str))
            elif payload.get("type") == "content_manifest":
                stack.extend(entry["blob"] for entry in payload.get("entries", []) if isinstance(entry, dict) and entry.get("blob"))
        return sorted(records, key=lambda record: record["object_id"])

    def remote_request_signature(self, actor, operation, target, key_id, key, nonce="fixed-nonce"):
        signature = {
            "type": "remote_request_signature",
            "version": 1,
            "algorithm": "hmac-sha256",
            "actor": actor,
            "operation": operation,
            "target": target,
            "key_id": key_id,
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "nonce": nonce,
        }
        data = json.dumps(signature, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
        signature["signature"] = hmac.new(key.encode("utf-8"), data, hashlib.sha256).hexdigest()
        return signature

    def test_init_creates_main_and_object_store(self):
        self.run_cf("init")
        self.assertTrue((self.tmp / ".codefire/repo.json").exists())
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text())
        self.assertEqual(branch["name"], "main")
        self.assertTrue(branch["head"].startswith("CF-COMMIT-"))
        self.run_cf("doctor")

    def test_doctor_detects_missing_object_references(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        manifest_record = json.loads((self.tmp / ".codefire/objects/content_manifests" / f"{manifest_id}.json").read_text(encoding="utf-8"))
        blob_id = manifest_record["payload"]["entries"][0]["blob"]

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        (self.tmp / ".codefire/objects/blobs" / f"{blob_id}.json").unlink()
        local_doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(local_doctor.returncode, 0)
        self.assertIn("missing object references: 1", local_doctor.stdout)
        self.assertIn(f"missing object: {blob_id}", local_doctor.stdout)

        remote_blob = server / ".codefire-server/projects/org/app/objects/blobs" / f"{blob_id}.json"
        remote_blob.unlink()
        remote_doctor = self.run_cf("doctor", project_url, check=False)
        self.assertNotEqual(remote_doctor.returncode, 0)
        self.assertIn("missing object references: 1", remote_doctor.stdout)
        self.assertIn(f"missing object: {blob_id}", remote_doctor.stdout)

    def test_doctor_detects_object_record_integrity_errors(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_path = self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json"
        commit_record = json.loads(commit_path.read_text(encoding="utf-8"))
        commit_record["hash"] = "bad-hash"
        commit_path.write_text(json.dumps(commit_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("object hash errors: 1", doctor.stdout)
        self.assertIn("bad object:", doctor.stdout)

        self.run_cf("init", str(self.tmp / "rename-repo"))
        renamed = next((self.tmp / "rename-repo/.codefire/objects/commits").glob("CF-COMMIT-*.json"))
        renamed.rename(renamed.with_name("CF-COMMIT-wrongname.json"))
        doctor_renamed = self.run_cf("doctor", cwd=self.tmp / "rename-repo", check=False)
        self.assertNotEqual(doctor_renamed.returncode, 0)
        self.assertIn("object hash errors: 1", doctor_renamed.stdout)

    def test_doctor_detects_invalid_local_sealed_commit_references(self):
        self.run_cf("init")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        branch["head"] = manifest_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 1", doctor.stdout)
        self.assertIn(f"invalid sealed ref: branch main: {manifest_id}: not a commit object", doctor.stdout)

    def test_open_and_clone_reject_invalid_local_branch_head(self):
        self.run_cf("init")
        _, bad_payload = self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        open_result = self.run_cf("open", "main", str(self.tmp / "main"), check=False)
        self.assertNotEqual(open_result.returncode, 0)
        self.assertIn("certificate is not consistent", open_result.stderr)

        clone_result = self.run_cf("clone", "main", "copy", check=False)
        self.assertNotEqual(clone_result.returncode, 0)
        self.assertIn("certificate is not consistent", clone_result.stderr)
        self.assertEqual(bad_payload["certificate"]["result"], "inconsistent")

    def test_open_directory_commands_reject_invalid_current_base_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        branch_path = self.tmp / ".codefire/branches/main.json"
        original_branch = json.loads(branch_path.read_text(encoding="utf-8"))
        bad_commit_id, _ = self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        branch_path.write_text(json.dumps(original_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        registry_path = self.tmp / ".codefire/opened/main.json"
        registry = json.loads(registry_path.read_text(encoding="utf-8"))
        registry["open"]["current_base_commit"] = bad_commit_id
        registry_path.write_text(json.dumps(registry, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        status = self.run_cf("status", cwd=main, check=False)
        self.assertNotEqual(status.returncode, 0)
        self.assertIn("open registry current_base_commit is invalid", status.stderr)
        self.assertIn("certificate is not consistent", status.stderr)

    def test_open_directory_commands_reject_invalid_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))

        status = self.run_cf("status", cwd=main, check=False)
        self.assertNotEqual(status.returncode, 0)
        self.assertIn("open branch head is invalid", status.stderr)
        self.assertIn("certificate is not consistent", status.stderr)

    def test_branch_list_rejects_invalid_local_branch_head(self):
        self.run_cf("init")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        listing = self.run_cf("branch", "list", check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("certificate is not consistent", listing.stderr)

    def test_object_graph_ignores_non_reference_cf_strings(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "verification:\n"
            "  required:\n"
            "    - id: cf-string-output\n"
            "      command: \"echo CF-NOT-A-REAL-OBJECT\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf("commit", "-m", "Message mentions CF-NOT-A-REAL-OBJECT", cwd=main)
        doctor = self.run_cf("doctor")
        self.assertIn("missing object references: 0", doctor.stdout)
        self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main")

    def test_demo_scan_fire_extinguish_verify_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        first_verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(first_verify.returncode, 0)
        self.assertIn("Open fires:", first_verify.stdout)
        scan = self.run_cf("scan", cwd=main)
        fire_ids = [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]
        self.assertGreaterEqual(len(fire_ids), 1)
        for fire_id in fire_ids:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        commit = self.run_cf("commit", "-m", "Initial consistent auth sample", cwd=main)
        self.assertIn("Sealed commit created.", commit.stdout)
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text())
        self.assertTrue(branch["head"].startswith("CF-COMMIT-"))

    def test_verify_runs_multiple_policy_commands(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "verification:\n"
            "  required:\n"
            "    - id: marker-one\n"
            "      command: \"printf ok > check_one.txt\"\n"
            "      cwd: \".\"\n"
            "    - id: marker-two\n"
            "      command: \"printf ok > check_two.txt\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.assertEqual((main / "check_one.txt").read_text(encoding="utf-8"), "ok")
        self.assertEqual((main / "check_two.txt").read_text(encoding="utf-8"), "ok")

    def test_commit_policy_can_allow_failed_verification_commands(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_verification_success: false\n"
            "verification:\n"
            "  required:\n"
            "    - id: allowed-failure\n"
            "      command: \"echo allowed failure && exit 9\"\n"
            "      cwd: \".\"\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "initial import", cwd=main)
        verify = self.run_cf("verify", cwd=main)
        self.assertIn("Verification passed.", verify.stdout)
        commit = self.run_cf("commit", "-m", "Commit with allowed verification failure", cwd=main)
        commit_id = next(line.split(":", 1)[1].strip() for line in commit.stdout.splitlines() if line.startswith("Commit:"))
        record = json.loads((self.tmp / ".codefire/objects/commits" / f"{commit_id}.json").read_text(encoding="utf-8"))
        self.assertEqual(record["payload"]["certificate"]["failed_checks"], 1)
        self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main")

    def test_commit_can_store_hmac_signature(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed import", cwd=main)
        self.run_cf("verify", cwd=main)
        commit = self.run_cf(
            "commit",
            "-m",
            "Signed commit",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        commit_id = next(line.split(":", 1)[1].strip() for line in commit.stdout.splitlines() if line.startswith("Commit:"))
        record = json.loads((self.tmp / ".codefire/objects/commits" / f"{commit_id}.json").read_text(encoding="utf-8"))
        signature = record["payload"]["signature"]
        self.assertEqual(signature["algorithm"], "hmac-sha256")
        self.assertEqual(signature["signer"], "alice")
        self.assertEqual(signature["key_id"], "alice")
        self.assertEqual(len(signature["signature"]), 64)
        show = self.run_cf("show", "main")
        self.assertIn("Signature: alice (hmac-sha256)", show.stdout)

    def test_remote_requires_and_verifies_commit_signature(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial unsigned sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)

        self.write_demo_files(main, 45)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed session update", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Signed session update",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        self.run_cf("upload", "main", main_url)
        remote_show = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", remote_show.stdout)

        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        signed_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(signed_record["payload"]))
        bad_payload["signature"]["signature"] = "0" * 64
        bad_commit_id = self.write_commit_object(bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        bad_server = self.tmp / "bad-server"
        bad_project_root = bad_server / ".codefire-server/projects/org/app"
        bad_project_root.mkdir(parents=True)
        (bad_project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        invalid = self.run_cf("upload", "main", f"cf://{bad_server}/org/app/main", check=False)
        self.assertNotEqual(invalid.returncode, 0)
        self.assertIn("invalid signature", invalid.stderr)

        remote_branch_path = project_root / "branches/main.json"
        remote_branch = json.loads(remote_branch_path.read_text(encoding="utf-8"))
        remote_branch["head"] = signed_record["payload"]["parents"][0]
        remote_branch_path.write_text(json.dumps(remote_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        doctor = self.run_cf("doctor", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid commit signature references: 1", doctor.stdout)
        gc = self.run_cf("gc", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("invalid commit signatures", gc.stderr)

    def test_remote_commit_signature_key_rotation_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "old signed import", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Old signed import",
            "--signer",
            "alice",
            "--key-id",
            "alice-2026-01",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "old-secret"},
        )

        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        policy = {
            "commit_signatures": {
                "required": True,
                "require_history": True,
                "keys": {
                    "alice-2026-01": {"secret": "old-secret", "signers": ["alice"], "not_after": "2099-01-01T00:00:00Z"},
                    "alice-2026-06": {"secret": "new-secret", "signers": ["alice"], "not_before": "2000-01-01T00:00:00Z"},
                },
            }
        }
        (project_root / "server_policy.json").write_text(json.dumps(policy), encoding="utf-8")
        main_url = f"cf://{server}/org/app/main"
        self.run_cf("upload", "main", main_url)
        show_old = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", show_old.stdout)

        self.write_demo_files(main, 55)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "rotated signing key", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Rotated signing key",
            "--signer",
            "alice",
            "--key-id",
            "alice-2026-06",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "new-secret"},
        )
        self.run_cf("upload", "main", main_url)

        policy["commit_signatures"]["keys"]["alice-2026-01"]["status"] = "revoked"
        (project_root / "server_policy.json").write_text(json.dumps(policy), encoding="utf-8")
        doctor = self.run_cf("doctor", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("key is revoked", doctor.stdout)
        gc = self.run_cf("gc", f"cf://{server}/org/app", check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("invalid commit signatures", gc.stderr)

    def test_http_remote_requires_commit_signature_on_upload(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial unsigned sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"commit_signatures": {"required": True, "keys": {"alice": "signing-secret"}}}),
            encoding="utf-8",
        )
        main_url = f"{endpoint}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)

        self.write_demo_files(main, 50)
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "signed HTTP update", cwd=main)
        self.run_cf("verify", cwd=main)
        self.run_cf(
            "commit",
            "-m",
            "Signed HTTP update",
            "--signer",
            "alice",
            cwd=main,
            env={"CODEFIRE_SIGNING_KEY": "signing-secret"},
        )
        self.run_cf("upload", "main", main_url)
        show = self.run_cf("show", main_url)
        self.assertIn("Signature: alice (hmac-sha256)", show.stdout)

    def test_obsolete_auto_fire_after_revert(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 15\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("FIRE-", scan.stdout)
        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 30\n",
            encoding="utf-8",
        )
        reverted = self.run_cf("scan", cwd=main)
        self.assertNotIn("FIRE-", reverted.stdout)

    def test_manual_fire_survives_scan_until_extinguished(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        created = self.run_cf("fire", "CODE-SessionPolicy", "--reason", "manual-review", cwd=main)
        self.assertIn("created FIRE-", created.stdout)
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("manual-review", self.run_cf("status", cwd=main).stdout)
        self.assertIn("Open fires:", scan.stdout)
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.run_cf("extinguish", "FIRE-001", "--resolution", "verified", "--evidence", "manual review complete", cwd=main)
        self.run_cf("verify", cwd=main)

    def test_stale_resolution_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        (main / "src/auth.py").write_text(
            "# cf-atom: CODE-SessionPolicy\n"
            "class SessionPolicy:\n"
            "    def expires_after_minutes(self):\n"
            "        return 15\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        self.run_cf(
            "extinguish",
            fire_id,
            "--resolution",
            "no-change-required",
            "--rationale",
            "design still intentionally describes the public contract",
            cwd=main,
        )
        (main / "docs/design/auth.md").write_text(
            "## DES-AUTH-001: Session policy\n\n"
            "SessionPolicy now needs another review.\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Stale resolutions: 1", verify.stdout)

    def test_duplicate_atom_id_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "docs/spec/duplicate.md").write_text(
            "## REQ-AUTH-001: Duplicate requirement\n\n"
            "This duplicates an Atom ID.\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Duplicate atom ids: 1", verify.stdout)

    def test_policy_required_links_can_be_customized(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/spec").mkdir(parents=True)
        (main / "docs/design").mkdir(parents=True)
        (main / "docs/spec/adr.md").write_text(
            "## ADR-CUSTOM-001: Custom decision\n\n"
            "This decision must point to design.\n",
            encoding="utf-8",
        )
        (main / "docs/design/decision.md").write_text(
            "## DES-CUSTOM-001: Custom design\n\n"
            "Design linked from the decision.\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - path: docs/spec/**/*.md\n"
            "      kind: requirement_document\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "required_links:\n"
            "  adr:\n"
            "    - type: refined_by\n"
            "      target_kind: design\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Missing required links: 1", verify.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: ADR-CUSTOM-001\n"
            "    to: DES-CUSTOM-001\n"
            "    type: refined_by\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        for fire_id in [line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "custom policy link added", cwd=main)
        self.run_cf("verify", cwd=main)

    def test_adr_and_ops_markdown_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "adr").mkdir(parents=True)
        (main / "docs/ops").mkdir(parents=True)
        (main / "adr/platform.md").write_text(
            "## ADR-PLATFORM-001: Runtime choice\n\n"
            "The service runs on the managed runtime.\n",
            encoding="utf-8",
        )
        (main / "docs/ops/deploy.md").write_text(
            "## OPS-DEPLOY-001: Runtime deployment\n\n"
            "Deploy the managed runtime release.\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  adrs:\n"
            "    - path: adr/**/*.md\n"
            "      kind: adr_document\n"
            "  ops:\n"
            "    - path: docs/ops/**/*.md\n"
            "      kind: ops_document\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  adr:\n"
            "    - type: operated_by\n"
            "      target_kind: ops\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("ADR-PLATFORM-001", scan.stdout)
        self.assertIn("OPS-DEPLOY-001", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: ADR-PLATFORM-001\n"
            "    to: OPS-DEPLOY-001\n"
            "    type: operated_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_openapi_json_operations_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/api").mkdir(parents=True)
        (main / "tests").mkdir(parents=True)
        (main / "docs/api/openapi.json").write_text(
            json.dumps(
                {
                    "openapi": "3.0.0",
                    "paths": {
                        "/sessions": {
                            "post": {
                                "operationId": "createSession",
                                "x-codefire-atom-id": "API-AUTH-CREATE-SESSION",
                                "summary": "Create a session",
                            }
                        }
                    },
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        (main / "tests/test_api.py").write_text(
            "# cf-atom: TEST-api-create-session\n"
            "def test_api_create_session_contract():\n"
            "    assert True\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  apis:\n"
            "    - path: docs/api/**/*.json\n"
            "      kind: openapi_json\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  api:\n"
            "    - type: verified_by\n"
            "      target_kind: test\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("API-AUTH-CREATE-SESSION", scan.stdout)
        self.assertIn("TEST-api-create-session", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: API-AUTH-CREATE-SESSION\n"
            "    to: TEST-api-create-session\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_openapi_yaml_operations_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/api").mkdir(parents=True)
        (main / "tests").mkdir(parents=True)
        (main / "docs/api/openapi.yaml").write_text(
            "openapi: 3.0.0\n"
            "paths:\n"
            "  /sessions:\n"
            "    post:\n"
            "      operationId: createSession\n"
            "      x-codefire-atom-id: API-AUTH-CREATE-SESSION-YAML\n"
            "      summary: Create a session\n"
            "    get:\n"
            "      operationId: listSessions\n",
            encoding="utf-8",
        )
        (main / "tests/test_api.py").write_text(
            "# cf-atom: TEST-api-create-session-yaml\n"
            "def test_api_create_session_contract():\n"
            "    assert True\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  apis:\n"
            "    - path: docs/api/**/*.yaml\n"
            "      kind: openapi_yaml\n"
            "  tests:\n"
            "    - path: tests/**/*.py\n"
            "      kind: pytest_test\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  api:\n"
            "    - type: verified_by\n"
            "      target_kind: test\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("API-AUTH-CREATE-SESSION-YAML", scan.stdout)
        self.assertIn("API-listSessions", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 2", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: API-AUTH-CREATE-SESSION-YAML\n"
            "    to: TEST-api-create-session-yaml\n"
            "    type: verified_by\n"
            "  - from: API-listSessions\n"
            "    to: TEST-api-create-session-yaml\n"
            "    type: verified_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_db_schema_tables_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "db").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "db/schema.sql").write_text(
            "CREATE TABLE IF NOT EXISTS sessions (\n"
            "  id TEXT PRIMARY KEY,\n"
            "  user_id TEXT NOT NULL,\n"
            "  expires_at TIMESTAMP,\n"
            "  CONSTRAINT sessions_user_id_fk FOREIGN KEY (user_id) REFERENCES users(id)\n"
            ");\n"
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);\n"
            "CREATE VIEW active_sessions AS\n"
            "  SELECT id, user_id FROM sessions WHERE expires_at > CURRENT_TIMESTAMP;\n"
            "CREATE TRIGGER sessions_touch AFTER UPDATE ON sessions BEGIN\n"
            "  SELECT NEW.id;\n"
            "END;\n"
            "CREATE FUNCTION calculate_session_count() RETURNS INTEGER AS $$\n"
            "BEGIN\n"
            "  RETURN 1;\n"
            "END;\n"
            "$$ LANGUAGE plpgsql;\n"
            "CREATE PROCEDURE prune_sessions() AS $$\n"
            "BEGIN\n"
            "  DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP;\n"
            "END;\n"
            "$$ LANGUAGE SQL;\n"
            "CREATE SEQUENCE IF NOT EXISTS session_events_seq START WITH 1;\n"
            "CREATE MATERIALIZED VIEW session_daily_counts AS\n"
            "  SELECT user_id, count(*) AS total FROM sessions GROUP BY user_id;\n"
            "CREATE TYPE session_status AS ENUM ('active', 'expired');\n",
            encoding="utf-8",
        )
        (main / "src/session_repo.py").write_text(
            "# cf-atom: CODE-SessionRepository\n"
            "class SessionRepository:\n"
            "    pass\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  db:\n"
            "    - path: db/**/*.sql\n"
            "      kind: sql_schema\n"
            "  code:\n"
            "    - path: src/**/*.py\n"
            "      kind: python_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  db:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DB-sessions", scan.stdout)
        self.assertIn("DB-sessions.id", scan.stdout)
        self.assertIn("DB-sessions.user_id", scan.stdout)
        self.assertIn("DB-sessions.expires_at", scan.stdout)
        self.assertIn("DB-idx_sessions_user_id", scan.stdout)
        self.assertIn("DB-active_sessions", scan.stdout)
        self.assertIn("DB-sessions_touch", scan.stdout)
        self.assertIn("DB-calculate_session_count", scan.stdout)
        self.assertIn("DB-prune_sessions", scan.stdout)
        self.assertIn("DB-session_events_seq", scan.stdout)
        self.assertIn("DB-session_daily_counts", scan.stdout)
        self.assertIn("DB-session_status", scan.stdout)
        self.assertNotIn("DB-sessions.sessions_user_id_fk", scan.stdout)
        self.assertIn("CODE-SessionRepository", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 12", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DB-sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.user_id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions.expires_at\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-idx_sessions_user_id\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-active_sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-sessions_touch\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-calculate_session_count\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-prune_sessions\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_events_seq\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_daily_counts\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n"
            "  - from: DB-session_status\n"
            "    to: CODE-SessionRepository\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_typescript_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/frontend.md").write_text(
            "## DES-FRONTEND-001: Session frontend\n\n"
            "The frontend shows the current session status.\n",
            encoding="utf-8",
        )
        (main / "src/session.ts").write_text(
            "// cf-atom: CODE-SessionViewModel\n"
            "export class SessionViewModel {\n"
            "  status(): string {\n"
            "    return 'active';\n"
            "  }\n"
            "}\n\n"
            "export const formatSessionStatus = (status: string): string => {\n"
            "  return status.toUpperCase();\n"
            "};\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.ts\n"
            "      kind: typescript_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-FRONTEND-001", scan.stdout)
        self.assertIn("CODE-SessionViewModel", scan.stdout)
        self.assertIn("CODE:src/session.ts::function:formatSessionStatus", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-FRONTEND-001\n"
            "    to: CODE-SessionViewModel\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_go_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-BACKEND-001: Session backend\n\n"
            "The backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session.go").write_text(
            "package session\n\n"
            "// cf-atom: CODE-GoSessionStore\n"
            "type Store struct {\n"
            "  sessions map[string]string\n"
            "}\n\n"
            "func NewStore() *Store {\n"
            "  return &Store{sessions: map[string]string{}}\n"
            "}\n\n"
            "func (s *Store) Find(id string) string {\n"
            "  return s.sessions[id]\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.go\n"
            "      kind: go_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-BACKEND-001", scan.stdout)
        self.assertIn("CODE-GoSessionStore", scan.stdout)
        self.assertIn("CODE:src/session.go::function:NewStore", scan.stdout)
        self.assertIn("CODE:src/session.go::method:Store.Find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-BACKEND-001\n"
            "    to: CODE-GoSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_java_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-JAVA-BACKEND-001: Java session backend\n\n"
            "The Java backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.java").write_text(
            "package session;\n\n"
            "import java.util.Map;\n\n"
            "// cf-atom: CODE-JavaSessionStore\n"
            "public class SessionStore {\n"
            "  private final Map<String, String> sessions;\n\n"
            "  public SessionStore(Map<String, String> sessions) {\n"
            "    this.sessions = sessions;\n"
            "  }\n\n"
            "  public String find(String id) {\n"
            "    return sessions.get(id);\n"
            "  }\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.java\n"
            "      kind: java_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-JAVA-BACKEND-001", scan.stdout)
        self.assertIn("CODE-JavaSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.java::method:SessionStore.find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-JAVA-BACKEND-001\n"
            "    to: CODE-JavaSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_csharp_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-CSHARP-BACKEND-001: CSharp session backend\n\n"
            "The CSharp backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.cs").write_text(
            "namespace Session;\n\n"
            "// cf-atom: CODE-CSharpSessionStore\n"
            "public class SessionStore\n"
            "{\n"
            "    private readonly Dictionary<string, string> sessions;\n\n"
            "    public SessionStore(Dictionary<string, string> sessions)\n"
            "    {\n"
            "        this.sessions = sessions;\n"
            "    }\n\n"
            "    public string? Find(string id)\n"
            "    {\n"
            "        return sessions.GetValueOrDefault(id);\n"
            "    }\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.cs\n"
            "      kind: csharp_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-CSHARP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-CSharpSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.cs::method:SessionStore.Find", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-CSHARP-BACKEND-001\n"
            "    to: CODE-CSharpSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_rust_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-RUST-BACKEND-001: Rust session backend\n\n"
            "The Rust backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session.rs").write_text(
            "use std::collections::HashMap;\n\n"
            "// cf-atom: CODE-RustSessionStore\n"
            "pub struct SessionStore {\n"
            "    sessions: HashMap<String, String>,\n"
            "}\n\n"
            "impl SessionStore {\n"
            "    pub fn new() -> Self {\n"
            "        Self { sessions: HashMap::new() }\n"
            "    }\n\n"
            "    pub fn find(&self, id: &str) -> Option<&String> {\n"
            "        self.sessions.get(id)\n"
            "    }\n"
            "}\n\n"
            "pub fn normalize_session_id(id: &str) -> String {\n"
            "    id.trim().to_string()\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.rs\n"
            "      kind: rust_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-RUST-BACKEND-001", scan.stdout)
        self.assertIn("CODE-RustSessionStore", scan.stdout)
        self.assertIn("CODE:src/session.rs::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session.rs::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-RUST-BACKEND-001\n"
            "    to: CODE-RustSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_kotlin_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-KOTLIN-BACKEND-001: Kotlin session backend\n\n"
            "The Kotlin backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.kt").write_text(
            "package session\n\n"
            "// cf-atom: CODE-KotlinSessionStore\n"
            "class SessionStore(private val sessions: Map<String, String>) {\n"
            "    fun find(id: String): String? {\n"
            "        return sessions[id]\n"
            "    }\n"
            "}\n\n"
            "fun normalizeSessionId(id: String): String {\n"
            "    return id.trim()\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.kt\n"
            "      kind: kotlin_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-KOTLIN-BACKEND-001", scan.stdout)
        self.assertIn("CODE-KotlinSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.kt::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.kt::function:normalizeSessionId", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-KOTLIN-BACKEND-001\n"
            "    to: CODE-KotlinSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_php_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-PHP-BACKEND-001: PHP session backend\n\n"
            "The PHP backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.php").write_text(
            "<?php\n\n"
            "namespace Session;\n\n"
            "// cf-atom: CODE-PhpSessionStore\n"
            "class SessionStore {\n"
            "    private array $sessions;\n\n"
            "    public function __construct(array $sessions) {\n"
            "        $this->sessions = $sessions;\n"
            "    }\n\n"
            "    public function find(string $id): ?string {\n"
            "        return $this->sessions[$id] ?? null;\n"
            "    }\n"
            "}\n\n"
            "function normalize_session_id(string $id): string {\n"
            "    return trim($id);\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.php\n"
            "      kind: php_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-PHP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-PhpSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.php::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.php::function:normalize_session_id", scan.stdout)
        self.assertNotIn("CODE:src/SessionStore.php::method:SessionStore.__construct", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-PHP-BACKEND-001\n"
            "    to: CODE-PhpSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_ruby_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-RUBY-BACKEND-001: Ruby session backend\n\n"
            "The Ruby backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session_store.rb").write_text(
            "module Session\n"
            "  # cf-atom: CODE-RubySessionStore\n"
            "  class SessionStore\n"
            "    def initialize(sessions)\n"
            "      @sessions = sessions\n"
            "    end\n\n"
            "    def find(id)\n"
            "      if id.nil?\n"
            "        nil\n"
            "      else\n"
            "        @sessions[id]\n"
            "      end\n"
            "    end\n"
            "  end\n"
            "end\n\n"
            "def normalize_session_id(id)\n"
            "  id.strip\n"
            "end\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.rb\n"
            "      kind: ruby_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-RUBY-BACKEND-001", scan.stdout)
        self.assertIn("CODE-RubySessionStore", scan.stdout)
        self.assertIn("CODE:src/session_store.rb::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session_store.rb::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-RUBY-BACKEND-001\n"
            "    to: CODE-RubySessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_swift_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-SWIFT-BACKEND-001: Swift session backend\n\n"
            "The Swift backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/SessionStore.swift").write_text(
            "import Foundation\n\n"
            "// cf-atom: CODE-SwiftSessionStore\n"
            "struct SessionStore {\n"
            "    let sessions: [String: String]\n\n"
            "    func find(id: String) -> String? {\n"
            "        return sessions[id]\n"
            "    }\n"
            "}\n\n"
            "func normalizeSessionId(_ id: String) -> String {\n"
            "    return id.trimmingCharacters(in: .whitespaces)\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.swift\n"
            "      kind: swift_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-SWIFT-BACKEND-001", scan.stdout)
        self.assertIn("CODE-SwiftSessionStore", scan.stdout)
        self.assertIn("CODE:src/SessionStore.swift::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/SessionStore.swift::function:normalizeSessionId", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-SWIFT-BACKEND-001\n"
            "    to: CODE-SwiftSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_cpp_code_atoms_are_indexed(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        (main / "docs/design").mkdir(parents=True)
        (main / "src").mkdir(parents=True)
        (main / "docs/design/backend.md").write_text(
            "## DES-CPP-BACKEND-001: C++ session backend\n\n"
            "The C++ backend loads session state.\n",
            encoding="utf-8",
        )
        (main / "src/session_store.cpp").write_text(
            "#include <string>\n"
            "#include <unordered_map>\n\n"
            "// cf-atom: CODE-CppSessionStore\n"
            "class SessionStore {\n"
            "public:\n"
            "    std::string find(const std::string& id) const {\n"
            "        return sessions.at(id);\n"
            "    }\n\n"
            "private:\n"
            "    std::unordered_map<std::string, std::string> sessions;\n"
            "};\n\n"
            "std::string normalize_session_id(const std::string& id) {\n"
            "    return id;\n"
            "}\n",
            encoding="utf-8",
        )
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  designs:\n"
            "    - path: docs/design/**/*.md\n"
            "      kind: design_document\n"
            "  code:\n"
            "    - path: src/**/*.cpp\n"
            "      kind: cpp_code\n",
            encoding="utf-8",
        )
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "commit_policy:\n"
            "  require_no_required_fires: false\n"
            "required_links:\n"
            "  design:\n"
            "    - type: implemented_by\n"
            "      target_kind: code\n"
            "      min: 1\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("DES-CPP-BACKEND-001", scan.stdout)
        self.assertIn("CODE-CppSessionStore", scan.stdout)
        self.assertIn("CODE:src/session_store.cpp::method:SessionStore.find", scan.stdout)
        self.assertIn("CODE:src/session_store.cpp::function:normalize_session_id", scan.stdout)
        missing = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("Missing required links: 1", missing.stdout)

        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: DES-CPP-BACKEND-001\n"
            "    to: CODE-CppSessionStore\n"
            "    type: implemented_by\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 0, verify.stderr)

    def test_invalid_codefire_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.yaml").write_text(
            "version: 1\n"
            "artifacts:\n"
            "  requirements:\n"
            "    - kind: requirement_document\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.yaml", verify.stderr)
        self.assertIn("must start with 'path:'", verify.stderr)

    def test_codefire_yaml_ignores_future_top_level_sections(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        with (main / "codefire.yaml").open("a", encoding="utf-8") as f:
            f.write(
                "\n"
                "atom_id:\n"
                "  require_explicit_ids_for:\n"
                "    - requirement\n"
                "    - design\n"
            )
        scan = self.run_cf("scan", cwd=main)
        self.assertIn("FIRE-", scan.stdout)

    def test_invalid_links_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.links.yaml").write_text(
            "version: 1\n"
            "links:\n"
            "  - from: REQ-AUTH-001\n"
            "    to: DES-AUTH-001\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.links.yaml", verify.stderr)
        self.assertIn("missing type", verify.stderr)

    def test_invalid_policy_yaml_reports_configuration_error(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "required_links:\n"
            "  requirement:\n"
            "    - type: refined_by\n"
            "      target_kind: design\n"
            "      min: many\n",
            encoding="utf-8",
        )
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertEqual(verify.returncode, 2)
        self.assertIn("invalid codefire.policy.yaml", verify.stderr)
        self.assertIn("min must be an integer", verify.stderr)

    def test_repo_lock_blocks_structural_changes(self):
        self.run_cf("init")
        lock = self.tmp / ".codefire/locks/repo.lock"
        lock.write_text("held by test\n", encoding="utf-8")
        proc = self.run_cf("clone", "main", "blocked", check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("lock is already held", proc.stderr)

    def test_close_and_close_discard(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.run_cf("close", "main")
        self.assertFalse(main.exists())

        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        self.run_cf("scan", cwd=main)
        failed = self.run_cf("close", "main", check=False)
        self.assertNotEqual(failed.returncode, 0)
        self.run_cf("close", "main", "--discard")
        self.assertFalse(main.exists())

    def test_discard_alias_closes_burning_branch(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        self.run_cf("scan", cwd=main)
        self.run_cf("discard", "main")
        self.assertFalse(main.exists())

    def test_show_and_diff_local_branches(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")

        show = self.run_cf("show", "feature-session")
        self.assertIn("Object: feature-session@CF-COMMIT-", show.stdout)
        self.assertIn("Certificate: consistent", show.stdout)

        diff = self.run_cf("diff", "main", "feature-session")
        self.assertIn("--- main@CF-COMMIT-", diff.stdout)
        self.assertIn("+++ feature-session@CF-COMMIT-", diff.stdout)
        self.assertIn("-        return 30", diff.stdout)
        self.assertIn("+        return 15", diff.stdout)

    def test_show_and_diff_reject_invalid_local_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))

        show = self.run_cf("show", "main", check=False)
        self.assertNotEqual(show.returncode, 0)
        self.assertIn("certificate is not consistent", show.stderr)
        diff = self.run_cf("diff", "main", "main", check=False)
        self.assertNotEqual(diff.returncode, 0)
        self.assertIn("certificate is not consistent", diff.stderr)

    def test_clone_and_merge_reject_open_burning_source(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.write_demo_files(feature, 15)
        self.run_cf("scan", cwd=feature)

        clone = self.run_cf("clone", "feature-session", "copy-of-burning", check=False)
        self.assertNotEqual(clone.returncode, 0)
        self.assertIn("cannot clone from branch 'feature-session' while it is open-burning", clone.stderr)

        merge = self.run_cf("merge", "feature-session", "--into", "main", check=False)
        self.assertNotEqual(merge.returncode, 0)
        self.assertIn("cannot merge from branch 'feature-session' while it is open-burning", merge.stderr)

    def test_merge_rejects_invalid_local_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        self.replace_main_head_with_mutated_commit(lambda payload: payload["certificate"].__setitem__("result", "inconsistent"))
        merge = self.run_cf("merge", "feature-session", "--into", "main", check=False)
        self.assertNotEqual(merge.returncode, 0)
        self.assertIn("certificate is not consistent", merge.stderr)

    def test_branch_names_with_slashes_do_not_collide(self):
        self.run_cf("init")
        self.run_cf("clone", "main", "feature/session")
        self.run_cf("clone", "main", "feature__session")
        listing = self.run_cf("branch", "list")
        self.assertIn("feature/session\t", listing.stdout)
        self.assertIn("feature__session\t", listing.stdout)
        self.assertTrue((self.tmp / ".codefire/branches/feature%2Fsession.json").exists())
        self.assertTrue((self.tmp / ".codefire/branches/feature__session.json").exists())

        feature = self.tmp / "feature-session-open"
        self.run_cf("open", "feature/session", str(feature))
        self.assertTrue((self.tmp / ".codefire/opened/feature%2Fsession.json").exists())
        status = self.run_cf("status", cwd=feature)
        self.assertIn("Branch: feature/session", status.stdout)
        scan = self.run_cf("scan", cwd=feature)
        self.assertIn("Branch state:", scan.stdout)
        self.run_cf("close", "feature/session")
        self.assertFalse(feature.exists())

    def test_branch_lock_names_with_slashes_do_not_collide(self):
        self.run_cf("init")
        stale_collision_lock = self.tmp / ".codefire/locks/branch-feature__slash.lock"
        stale_collision_lock.write_text("unrelated stale lock\n", encoding="utf-8")
        clone = self.run_cf("clone", "main", "feature/slash")
        self.assertIn("cloned main -> feature/slash", clone.stdout)
        self.assertTrue((self.tmp / ".codefire/branches/feature%2Fslash.json").exists())

    def test_remote_branch_names_with_slashes_use_encoded_urls(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature/session")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        encoded_url = f"{project_url}/feature%2Fsession"
        self.run_cf("upload", "feature/session", encoded_url)
        self.assertTrue((server / ".codefire-server/projects/org/app/branches/feature%2Fsession.json").exists())
        listing = self.run_cf("list", project_url)
        self.assertIn("feature/session\t", listing.stdout)
        show = self.run_cf("show", encoded_url)
        self.assertIn(f"Object: {encoded_url}@CF-COMMIT-", show.stdout)

        clone_repo = self.tmp / "encoded-clone-repo"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.run_cf("clone", encoded_url, "remote/feature-session", cwd=clone_repo)
        self.assertTrue((clone_repo / ".codefire/branches/remote%2Ffeature-session.json").exists())

        self.run_cf("upload", "main", f"{project_url}/main")
        mr = self.run_cf("request-merge", f"{project_url}/main", encoded_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        requests = self.run_cf("request-list", project_url)
        self.assertIn(encoded_url, requests.stdout)
        self.run_cf("request-review", project_url, mr_id, "--reviewer", "alice", "--decision", "approve")
        applied = self.run_cf("request-apply", project_url, mr_id)
        self.assertIn("target: feature/session@CF-COMMIT-", applied.stdout)
        self.assertIn("Certificate: consistent", self.run_cf("show", encoded_url).stdout)

    def test_remote_branch_lock_names_with_slashes_do_not_collide(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature/slash")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        project_root = server / ".codefire-server/projects/org/app"
        locks = project_root / "locks"
        locks.mkdir(parents=True, exist_ok=True)
        (locks / "branch-feature__slash.lock").write_text("unrelated stale lock\n", encoding="utf-8")
        upload = self.run_cf("upload", "feature/slash", f"{project_url}/feature%2Fslash")
        self.assertIn("uploaded feature/slash@CF-COMMIT-", upload.stdout)

    def test_upload_list_remote_clone_and_request_merge(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)
        remote_doctor = self.run_cf("doctor", project_url)
        self.assertIn("CodeFire remote doctor", remote_doctor.stdout)
        self.assertIn("object hash errors: 0", remote_doctor.stdout)
        listing = self.run_cf("list", project_url)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)

        clone_repo = self.tmp / "clone-repo"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-remote", cwd=clone_repo)
        self.assertIn("main-from-remote", clone.stdout)
        clone_show = self.run_cf("show", "main-from-remote", cwd=clone_repo)
        self.assertIn("Certificate: consistent", clone_show.stdout)

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")
        feature_url = f"{project_url}/feature-session"
        self.run_cf("upload", "feature-session", feature_url)

        remote_show = self.run_cf("show", feature_url)
        self.assertIn(f"Object: {feature_url}@CF-COMMIT-", remote_show.stdout)
        outside = self.tmp / "outside"
        outside.mkdir()
        remote_show_outside = self.run_cf("show", feature_url, cwd=outside)
        self.assertIn("Certificate: consistent", remote_show_outside.stdout)
        remote_diff = self.run_cf("diff", main_url, feature_url, cwd=outside)
        self.assertIn(f"--- {main_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn(f"+++ {feature_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn("+        return 15", remote_diff.stdout)

        mr = self.run_cf("request-merge", feature_url, main_url)
        self.assertIn("created MR-", mr.stdout)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        requests = self.run_cf("request-list", project_url)
        self.assertIn("open", requests.stdout)
        self.assertIn(feature_url, requests.stdout)
        review = self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
            "--comment",
            "sealed source is ready",
        )
        self.assertIn(f"approved {mr_id} by alice", review.stdout)
        approved_requests = self.run_cf("request-list", project_url)
        self.assertIn("approved", approved_requests.stdout)
        applied = self.run_cf("request-apply", project_url, mr_id)
        self.assertIn(f"applied {mr_id}", applied.stdout)
        target_after_apply = self.run_cf("show", main_url)
        feature_after_apply = self.run_cf("show", feature_url)
        self.assertEqual(
            next(line for line in target_after_apply.stdout.splitlines() if line.startswith("Commit:")),
            next(line for line in feature_after_apply.stdout.splitlines() if line.startswith("Commit:")),
        )

        # Advancing the target branch makes the existing merge request stale.
        self.run_cf("clone", main_url, "target-advance")
        target_advance = self.tmp / "target-advance"
        self.run_cf("open", "target-advance", str(target_advance))
        self.commit_demo_version(target_advance, 45, "Advance target branch")
        self.run_cf("upload", "target-advance", main_url)
        stale_mr = self.run_cf("request-merge", feature_url, main_url)
        stale_mr_id = next(line.split()[1] for line in stale_mr.stdout.splitlines() if line.startswith("created MR-"))
        self.run_cf(
            "request-review",
            project_url,
            stale_mr_id,
            "--reviewer",
            "alice",
            "--decision",
            "approve",
        )
        stale_requests = self.run_cf("request-list", project_url)
        self.assertIn("approved", stale_requests.stdout)
        # Move target again after approval; apply must mark the MR stale.
        self.run_cf("clone", main_url, "target-advance-again")
        target_advance_again = self.tmp / "target-advance-again"
        self.run_cf("open", "target-advance-again", str(target_advance_again))
        self.commit_demo_version(target_advance_again, 60, "Advance target branch again")
        self.run_cf("upload", "target-advance-again", main_url)
        stale_apply = self.run_cf("request-apply", project_url, stale_mr_id, check=False)
        self.assertNotEqual(stale_apply.returncode, 0)
        self.assertIn("merge request is stale", stale_apply.stderr)
        stale_review = self.run_cf(
            "request-review",
            project_url,
            stale_mr_id,
            "--reviewer",
            "bob",
            "--decision",
            "approve",
            check=False,
        )
        self.assertNotEqual(stale_review.returncode, 0)
        self.assertIn("merge request is stale", stale_review.stderr)

    def test_http_remote_upload_list_and_clone(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP remote sample")

        base_url = self.start_http_remote()
        project_url = f"{base_url}/org/app"
        main_url = f"{project_url}/main"
        upload = self.run_cf("upload", "main", main_url)
        self.assertIn("uploaded main@CF-COMMIT-", upload.stdout)

        listing = self.run_cf("list", project_url)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)
        remote_doctor = self.run_cf("doctor", project_url)
        self.assertIn("CodeFire remote doctor", remote_doctor.stdout)
        self.assertIn("object hash errors: 0", remote_doctor.stdout)

        clone_repo = self.tmp / "http-clone"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-http", cwd=clone_repo)
        self.assertIn("main-from-http", clone.stdout)
        show = self.run_cf("show", "main-from-http", cwd=clone_repo)
        self.assertIn("Initial HTTP remote sample", show.stdout)

        self.run_cf("clone", "main", "feature-http")
        feature = self.tmp / "feature-http"
        self.run_cf("open", "feature-http", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration over HTTP")
        feature_url = f"{project_url}/feature-http"
        self.run_cf("upload", "feature-http", feature_url)

        outside = self.tmp / "http-outside"
        outside.mkdir()
        remote_show = self.run_cf("show", feature_url, cwd=outside)
        self.assertIn(f"Object: {feature_url}@CF-COMMIT-", remote_show.stdout)
        self.assertIn("Change session expiration over HTTP", remote_show.stdout)
        remote_diff = self.run_cf("diff", main_url, feature_url, cwd=outside)
        self.assertIn(f"--- {main_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn(f"+++ {feature_url}@CF-COMMIT-", remote_diff.stdout)
        self.assertIn("+        return 15", remote_diff.stdout)

        mr = self.run_cf("request-merge", feature_url, main_url)
        self.assertIn("created MR-", mr.stdout)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        requests = self.run_cf("request-list", project_url)
        self.assertIn("open", requests.stdout)
        self.assertIn(feature_url, requests.stdout)
        review = self.run_cf("request-review", project_url, mr_id, "--reviewer", "alice", "--decision", "approve")
        self.assertIn(f"approved {mr_id} by alice", review.stdout)
        applied = self.run_cf("request-apply", project_url, mr_id)
        self.assertIn(f"applied {mr_id}", applied.stdout)

        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"orphan": "http"}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        dry_run = self.run_cf("gc", project_url, "--dry-run")
        self.assertIn(f"would remove {orphan_id}", dry_run.stdout)
        removed = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 1", removed.stdout)
        self.assertFalse(orphan_path.exists())
        self.assertTrue((project_root / "audit/gc.jsonl").exists())

        post_apply_repo = self.tmp / "http-post-apply"
        post_apply_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=post_apply_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.run_cf("clone", main_url, "main-after-http-apply", cwd=post_apply_repo)
        post_apply_show = self.run_cf("show", "main-after-http-apply", cwd=post_apply_repo)
        self.assertIn("Change session expiration over HTTP", post_apply_show.stdout)

    def test_https_remote_upload_list_clone_and_show(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTPS remote sample")

        base_url = self.start_http_remote(tls=True)
        self.assertTrue(base_url.startswith("cf+https://"))
        project_url = f"{base_url}/org/app"
        main_url = f"{project_url}/main"
        tls_env = {"CODEFIRE_TLS_INSECURE": "1"}
        upload = self.run_cf("upload", "main", main_url, env=tls_env)
        self.assertIn("uploaded main@CF-COMMIT-", upload.stdout)
        listing = self.run_cf("list", project_url, env=tls_env)
        self.assertIn("main\tCF-COMMIT-", listing.stdout)
        remote_show = self.run_cf("show", main_url, env=tls_env)
        self.assertIn("Initial HTTPS remote sample", remote_show.stdout)

        clone_repo = self.tmp / "https-clone"
        clone_repo.mkdir()
        subprocess.run([str(CODEFIRE), "init"], cwd=clone_repo, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        clone = self.run_cf("clone", main_url, "main-from-https", cwd=clone_repo, env=tls_env)
        self.assertIn("main-from-https", clone.stdout)

    def test_show_and_diff_reject_invalid_remote_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)

        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        bad_payload["certificate"]["result"] = "inconsistent"
        bad_commit_id = self.write_commit_object_to(project_root / "objects", bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        show = self.run_cf("show", main_url, check=False)
        self.assertNotEqual(show.returncode, 0)
        self.assertIn("certificate is not consistent", show.stderr)
        diff = self.run_cf("diff", main_url, main_url, check=False)
        self.assertNotEqual(diff.returncode, 0)
        self.assertIn("certificate is not consistent", diff.stderr)

    def test_remote_list_rejects_invalid_branch_head(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)

        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = json.loads(json.dumps(commit_record["payload"]))
        bad_payload["certificate"]["result"] = "inconsistent"
        bad_commit_id = self.write_commit_object_to(project_root / "objects", bad_payload)
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        listing = self.run_cf("list", project_url, check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("certificate is not consistent", listing.stderr)

    def test_request_list_and_review_reject_invalid_merge_request_refs(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        feature_url = f"{project_url}/feature-session"
        self.run_cf("upload", "main", main_url)
        self.run_cf("upload", "feature-session", feature_url)
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))

        project_root = server / ".codefire-server/projects/org/app"
        feature_branch = json.loads((project_root / "branches/feature-session.json").read_text(encoding="utf-8"))
        feature_commit = json.loads((project_root / "objects/commits" / f"{feature_branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = feature_commit["payload"]["roots"]["content_manifest"]
        mr_path = project_root / "merge_requests" / f"{mr_id}.json"
        mr_record = json.loads(mr_path.read_text(encoding="utf-8"))
        mr_record["source_head"] = manifest_id
        mr_path.write_text(json.dumps(mr_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        listing = self.run_cf("request-list", project_url, check=False)
        self.assertNotEqual(listing.returncode, 0)
        self.assertIn("merge request validation failed: source_head: not a commit object", listing.stderr)
        review = self.run_cf("request-review", project_url, mr_id, "--reviewer", "alice", "--decision", "approve", check=False)
        self.assertNotEqual(review.returncode, 0)
        self.assertIn("merge request validation failed: source_head: not a commit object", review.stderr)

    def test_upload_rejects_non_fast_forward(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        main_url = f"cf://{server}/org/app/main"
        self.run_cf("upload", "main", main_url)

        # Advance the remote branch through another local branch.
        self.run_cf("clone", "main", "remote-advance")
        remote_advance = self.tmp / "remote-advance"
        self.run_cf("open", "remote-advance", str(remote_advance))
        self.commit_demo_version(remote_advance, 15, "Remote branch advances")
        self.run_cf("upload", "remote-advance", main_url)

        # Try uploading the older main head again.
        rejected = self.run_cf("upload", "main", main_url, check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("remote branch is not an ancestor", rejected.stderr)

    def test_upload_rejects_inconsistent_sealed_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        commit_path = self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json"
        record = json.loads(commit_path.read_text(encoding="utf-8"))
        record["payload"]["certificate"]["result"] = "inconsistent"
        commit_path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("object hash mismatch", rejected.stderr)

    def test_upload_rejects_sealed_commit_with_wrong_root_type(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        bad_payload = dict(commit_record["payload"])
        bad_payload["roots"] = dict(commit_record["payload"]["roots"])
        bad_payload["roots"]["content_manifest"] = bad_payload["roots"]["verification"]
        bad_commit_id = self.remote_object_id("commit", bad_payload)
        bad_record = {
            "object_id": bad_commit_id,
            "type": "commit",
            "hash": self.object_digest("commit", bad_payload),
            "payload": bad_payload,
        }
        bad_path = self.tmp / ".codefire/objects/commits" / f"{bad_commit_id}.json"
        bad_path.write_text(json.dumps(bad_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        branch["head"] = bad_commit_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("root content_manifest has type verification, expected content_manifest", rejected.stderr)
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 1", doctor.stdout)
        self.assertIn("root content_manifest has type verification, expected content_manifest", doctor.stdout)

    def test_upload_rejects_sealed_commit_with_malformed_payload_fields(self):
        cases = [
            ("parents", lambda payload: payload.__setitem__("parents", "not-a-list"), "parents must be a list"),
            ("roots", lambda payload: payload.__setitem__("roots", []), "roots must be an object"),
            ("certificate", lambda payload: payload.__setitem__("certificate", []), "certificate must be an object"),
        ]
        for name, mutate, message in cases:
            with self.subTest(name=name):
                repo = self.tmp / name
                repo.mkdir()
                subprocess.run([str(CODEFIRE), "init"], cwd=repo, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                old_tmp = self.tmp
                self.tmp = repo
                try:
                    main = repo / "main"
                    self.run_cf("open", "main", str(main), cwd=repo)
                    self.commit_demo_version(main, 30, f"Initial sample for {name}")
                    self.replace_main_head_with_mutated_commit(mutate)
                    rejected = self.run_cf("upload", "main", f"cf://{repo / 'server'}/org/app/main", cwd=repo, check=False)
                    self.assertNotEqual(rejected.returncode, 0)
                    self.assertIn(message, rejected.stderr)
                    doctor = self.run_cf("doctor", cwd=repo, check=False)
                    self.assertNotEqual(doctor.returncode, 0)
                    self.assertIn(message, doctor.stdout)
                finally:
                    self.tmp = old_tmp

    def test_upload_rejects_sealed_commit_with_invalid_parent_history(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        branch_path = self.tmp / ".codefire/branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        head_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        parent_id = head_record["payload"]["parents"][0]
        parent_record = json.loads((self.tmp / ".codefire/objects/commits" / f"{parent_id}.json").read_text(encoding="utf-8"))

        bad_parent_payload = json.loads(json.dumps(parent_record["payload"]))
        bad_parent_payload["certificate"]["result"] = "inconsistent"
        bad_parent_id = self.write_commit_object(bad_parent_payload)

        bad_head_payload = json.loads(json.dumps(head_record["payload"]))
        bad_head_payload["parents"] = [bad_parent_id]
        bad_head_id = self.write_commit_object(bad_head_payload)
        branch["head"] = bad_head_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        rejected = self.run_cf("upload", "main", f"cf://{self.tmp / 'server'}/org/app/main", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("certificate is not consistent", rejected.stderr)
        doctor = self.run_cf("doctor", check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("certificate is not consistent", doctor.stdout)

    def test_upload_runs_server_side_verification_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "commit-env-present",
                                "command": "test -n \"$CODEFIRE_REMOTE_COMMIT\"",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("upload", "main", f"cf://{server}/org/app/main")

        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "reject-all",
                                "command": "echo server rejected && exit 7",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "server-verify-fail")
        failing = self.tmp / "server-verify-fail"
        self.run_cf("open", "server-verify-fail", str(failing))
        self.commit_demo_version(failing, 15, "Commit rejected by server policy")
        rejected = self.run_cf("upload", "server-verify-fail", f"cf://{server}/org/app/server-verify-fail", check=False)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("server verification failed: reject-all", rejected.stderr)

    def test_server_side_verification_isolates_cwd_env_and_timeout(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial server isolation sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        checks_dir = project_root / "checks"
        checks_dir.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "isolated-env-cwd",
                                "cwd": "checks",
                                "timeout_seconds": 5,
                                "env": {"CUSTOM_ALLOWED": "yes"},
                                "command": "test \"$CUSTOM_ALLOWED\" = yes && test -z \"$SECRET_PARENT\" && pwd > marker.txt",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("upload", "main", f"cf://{server}/org/app/main", env={"SECRET_PARENT": "should-not-leak"})
        self.assertTrue((checks_dir / "marker.txt").exists())

        (project_root / "server_policy.json").write_text(
            json.dumps({"verification": {"required": [{"id": "bad-cwd", "cwd": "..", "command": "true"}]}}),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "bad-cwd")
        bad_cwd = self.tmp / "bad-cwd"
        self.run_cf("open", "bad-cwd", str(bad_cwd))
        self.commit_demo_version(bad_cwd, 31, "Server verification cwd escape")
        rejected_cwd = self.run_cf("upload", "bad-cwd", f"cf://{server}/org/app/bad-cwd", check=False)
        self.assertNotEqual(rejected_cwd.returncode, 0)
        self.assertIn("cwd escapes remote project", rejected_cwd.stderr)

        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "verification": {
                        "required": [
                            {
                                "id": "timeout-check",
                                "timeout_seconds": 0.1,
                                "command": "python3 -c 'import time; time.sleep(2)'",
                            }
                        ]
                    }
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "timeout-check")
        timeout_branch = self.tmp / "timeout-check"
        self.run_cf("open", "timeout-check", str(timeout_branch))
        self.commit_demo_version(timeout_branch, 32, "Server verification timeout")
        rejected_timeout = self.run_cf("upload", "timeout-check", f"cf://{server}/org/app/timeout-check", check=False)
        self.assertNotEqual(rejected_timeout.returncode, 0)
        self.assertIn("timed out after", rejected_timeout.stderr)

    def test_remote_permissions_gate_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps({"permissions": {"upload": ["alice"], "gc": ["admin"]}}),
            encoding="utf-8",
        )
        denied = self.run_cf("upload", "main", f"cf://{server}/org/app/main", "--actor", "bob", check=False)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn("permission denied", denied.stderr)
        self.run_cf("upload", "main", f"cf://{server}/org/app/main", "--actor", "alice")
        denied_gc = self.run_cf("gc", f"cf://{server}/org/app", "--actor", "alice", check=False)
        self.assertNotEqual(denied_gc.returncode, 0)
        self.assertIn("permission denied", denied_gc.stderr)

    def test_remote_branch_protection_gates_upload_and_apply(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "branch_protection": {
                        "main": {
                            "upload": ["alice"],
                            "apply": ["admin"],
                        }
                    }
                }
            ),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        denied_upload = self.run_cf("upload", "main", main_url, "--actor", "bob", check=False)
        self.assertNotEqual(denied_upload.returncode, 0)
        self.assertIn("branch protection denied", denied_upload.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice")

        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")
        feature_url = f"cf://{server}/org/app/feature"
        self.run_cf("upload", "feature", feature_url, "--actor", "bob")
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        self.run_cf("request-review", f"cf://{server}/org/app", mr_id, "--reviewer", "reviewer", "--decision", "approve")

        denied_apply = self.run_cf("request-apply", f"cf://{server}/org/app", mr_id, "--actor", "alice", check=False)
        self.assertNotEqual(denied_apply.returncode, 0)
        self.assertIn("branch protection denied", denied_apply.stderr)
        applied = self.run_cf("request-apply", f"cf://{server}/org/app", mr_id, "--actor", "admin")
        self.assertIn(f"applied {mr_id}", applied.stdout)

    def test_remote_token_auth_required_for_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "permissions": {"upload": ["alice"]},
                    "auth": {"required": True, "tokens": {"alice": "secret"}},
                }
            ),
            encoding="utf-8",
        )
        main_url = f"cf://{server}/org/app/main"
        missing = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("token is required", missing.stderr)
        wrong = self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "wrong", check=False)
        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("invalid token", wrong.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "secret")

    def test_remote_token_auth_accepts_hashed_tokens(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "salted")
        salted = self.tmp / "salted"
        self.run_cf("open", "salted", str(salted))
        self.commit_demo_version(salted, 45, "Salted hashed token branch")

        server = self.tmp / "server"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        alice_hash = json.loads(self.run_cf("token-hash", "secret").stdout)
        bob_hash = json.loads(self.run_cf("token-hash", "sensitive", "--salt", "pepper").stdout)
        self.assertTrue(alice_hash.startswith("sha256:"))
        self.assertEqual(bob_hash["algorithm"], "sha256")
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "permissions": {"upload": ["alice", "bob"]},
                    "auth": {"required": True, "tokens": {"alice": alice_hash, "bob": bob_hash}},
                }
            ),
            encoding="utf-8",
        )

        main_url = f"cf://{server}/org/app/main"
        wrong = self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "wrong", check=False)
        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("invalid token", wrong.stderr)
        self.run_cf("upload", "main", main_url, "--actor", "alice", "--token", "secret")
        salted_url = f"cf://{server}/org/app/salted"
        self.run_cf("upload", "salted", salted_url, "--actor", "bob", "--token", "sensitive")

    def test_remote_request_signatures_gate_mutating_operations(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial request signature sample")
        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 45, "Signed request feature")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        project_root = server / ".codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "max_skew_seconds": 300,
                        "keys": {
                            "alice-req": {"secret": "alice-request-secret", "signers": ["alice"]},
                            "reviewer-req": {"secret": "review-request-secret", "signers": ["reviewer"]},
                            "admin-req": {"secret": "admin-request-secret", "signers": ["admin"]},
                        },
                    }
                }
            ),
            encoding="utf-8",
        )

        main_url = f"{project_url}/main"
        unsigned = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)
        self.run_cf(
            "upload",
            "main",
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        feature_url = f"{project_url}/feature"
        self.run_cf(
            "upload",
            "feature",
            feature_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        mr = self.run_cf(
            "request-merge",
            feature_url,
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))
        unsigned_review = self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "reviewer",
            "--decision",
            "approve",
            "--actor",
            "reviewer",
            check=False,
        )
        self.assertNotEqual(unsigned_review.returncode, 0)
        self.assertIn("signature is required", unsigned_review.stderr)
        self.run_cf(
            "request-review",
            project_url,
            mr_id,
            "--reviewer",
            "reviewer",
            "--decision",
            "approve",
            "--actor",
            "reviewer",
            "--request-key-id",
            "reviewer-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "review-request-secret"},
        )
        self.run_cf(
            "request-apply",
            project_url,
            mr_id,
            "--actor",
            "admin",
            "--request-key-id",
            "admin-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "admin-request-secret"},
        )
        gc = self.run_cf(
            "gc",
            project_url,
            "--dry-run",
            "--actor",
            "admin",
            "--request-key-id",
            "admin-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "admin-request-secret"},
        )
        self.assertIn("reachable_objects:", gc.stdout)

    def test_http_remote_request_signature_required_on_upload(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP request signature sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "keys": {"alice-req": {"secret": "alice-request-secret", "signers": ["alice"]}},
                    }
                }
            ),
            encoding="utf-8",
        )
        main_url = f"{endpoint}/org/app/main"
        unsigned = self.run_cf("upload", "main", main_url, "--actor", "alice", check=False)
        self.assertNotEqual(unsigned.returncode, 0)
        self.assertIn("signature is required", unsigned.stderr)
        signed = self.run_cf(
            "upload",
            "main",
            main_url,
            "--actor",
            "alice",
            "--request-key-id",
            "alice-req",
            env={"CODEFIRE_REQUEST_SIGNING_KEY": "alice-request-secret"},
        )
        self.assertIn("uploaded main@CF-COMMIT-", signed.stdout)

    def test_http_remote_request_signature_rejects_replayed_nonce(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial HTTP replay signature sample")
        endpoint = self.start_http_remote()
        project_root = self.tmp / "http-storage/.codefire-server/projects/org/app"
        project_root.mkdir(parents=True)
        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "request_signatures": {
                        "required": True,
                        "max_skew_seconds": 300,
                        "nonce_ttl_seconds": 300,
                        "keys": {"alice-req": {"secret": "alice-request-secret", "signers": ["alice"]}},
                    }
                }
            ),
            encoding="utf-8",
        )
        branch = json.loads((self.tmp / ".codefire/branches/main.json").read_text(encoding="utf-8"))
        objects_root = self.tmp / ".codefire/objects"
        payload = {
            "branch": "main",
            "head": branch["head"],
            "objects": self.reachable_object_records(objects_root, branch["head"]),
            "actor": "alice",
            "request_signature": self.remote_request_signature(
                "alice",
                "upload",
                "org/app/branches/main/upload",
                "alice-req",
                "alice-request-secret",
                nonce="replay-nonce",
            ),
        }
        upload_url = endpoint.replace("cf+http://", "http://") + "/v1/projects/org/app/branches/main/upload"
        request = urllib.request.Request(
            upload_url,
            data=json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8"),
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(request, timeout=10) as response:
            self.assertEqual(response.status, 200)
        replay = urllib.request.Request(
            upload_url,
            data=json.dumps(payload, ensure_ascii=False, sort_keys=True).encode("utf-8"),
            method="POST",
            headers={"Content-Type": "application/json"},
        )
        with self.assertRaises(urllib.error.HTTPError) as ctx:
            urllib.request.urlopen(replay, timeout=10)
        body = ctx.exception.read().decode("utf-8")
        self.assertIn("nonce has already been used", body)
        cache = json.loads((project_root / "request_nonce_cache.json").read_text(encoding="utf-8"))
        self.assertEqual(len(cache["entries"]), 1)

    def test_remote_doctor_detects_invalid_sealed_commit_references(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        self.run_cf("clone", "main", "feature")
        feature = self.tmp / "feature"
        self.run_cf("open", "feature", str(feature))
        self.commit_demo_version(feature, 15, "Feature commit")

        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        feature_url = f"{project_url}/feature"
        self.run_cf("upload", "main", main_url)
        self.run_cf("upload", "feature", feature_url)
        mr = self.run_cf("request-merge", feature_url, main_url)
        mr_id = next(line.split()[1] for line in mr.stdout.splitlines() if line.startswith("created MR-"))

        project_root = server / ".codefire-server/projects/org/app"
        feature_branch_path = project_root / "branches/feature.json"
        feature_branch = json.loads(feature_branch_path.read_text(encoding="utf-8"))
        feature_commit_record = json.loads((project_root / "objects/commits" / f"{feature_branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = feature_commit_record["payload"]["roots"]["content_manifest"]

        main_branch_path = project_root / "branches/main.json"
        main_branch = json.loads(main_branch_path.read_text(encoding="utf-8"))
        main_branch["head"] = manifest_id
        main_branch_path.write_text(json.dumps(main_branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        mr_path = project_root / "merge_requests" / f"{mr_id}.json"
        mr_record = json.loads(mr_path.read_text(encoding="utf-8"))
        mr_record["source_head"] = manifest_id
        mr_path.write_text(json.dumps(mr_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        doctor = self.run_cf("doctor", project_url, check=False)
        self.assertNotEqual(doctor.returncode, 0)
        self.assertIn("invalid sealed commit references: 2", doctor.stdout)
        self.assertIn(f"invalid sealed ref: branch main: {manifest_id}: not a commit object", doctor.stdout)
        self.assertIn(f"invalid sealed ref: merge_request {mr_id} source_head: {manifest_id}: not a commit object", doctor.stdout)

    def test_remote_gc_removes_unreachable_objects(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        dry = self.run_cf("gc", project_url, "--dry-run")
        self.assertIn(f"would remove {orphan_id}", dry.stdout)
        self.assertTrue(orphan_path.exists())
        gc = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 1", gc.stdout)
        self.assertFalse(orphan_path.exists())

    def test_remote_gc_writes_audit_log(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        orphan_payload = {"type": "policy", "version": 1, "policy": {"audited_orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        dry = self.run_cf("gc", project_url, "--dry-run", "--actor", "auditor")
        self.assertIn("audit_log: audit/gc.jsonl", dry.stdout)
        self.assertTrue(orphan_path.exists())
        removed = self.run_cf("gc", project_url, "--actor", "auditor")
        self.assertIn("audit_log: audit/gc.jsonl", removed.stdout)
        self.assertFalse(orphan_path.exists())

        audit_path = project_root / "audit/gc.jsonl"
        entries = [json.loads(line) for line in audit_path.read_text(encoding="utf-8").splitlines()]
        self.assertEqual(len(entries), 2)
        self.assertTrue(entries[0]["dry_run"])
        self.assertFalse(entries[1]["dry_run"])
        self.assertEqual(entries[0]["actor"], "auditor")
        self.assertEqual(entries[1]["actor"], "auditor")
        self.assertEqual(entries[0]["removed_objects"][0]["object_id"], orphan_id)
        self.assertEqual(entries[1]["removed_objects"][0]["object_id"], orphan_id)
        self.assertEqual(entries[1]["objects_removed"], 1)

    def test_remote_gc_blocks_on_unhealthy_object_graph(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"

        orphan_payload = {"type": "policy", "version": 1, "policy": {"must_not_delete": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(
            json.dumps(
                {
                    "object_id": orphan_id,
                    "type": "policy",
                    "hash": self.object_digest("policy", orphan_payload),
                    "payload": orphan_payload,
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

        branch = json.loads((project_root / "branches/main.json").read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        manifest_record = json.loads((project_root / "objects/content_manifests" / f"{manifest_id}.json").read_text(encoding="utf-8"))
        blob_id = manifest_record["payload"]["entries"][0]["blob"]
        (project_root / "objects/blobs" / f"{blob_id}.json").unlink()

        gc = self.run_cf("gc", project_url, check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("gc blocked: missing object references: 1", gc.stderr)
        self.assertTrue(orphan_path.exists())

    def test_remote_gc_blocks_on_invalid_sealed_refs(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        branch_path = project_root / "branches/main.json"
        branch = json.loads(branch_path.read_text(encoding="utf-8"))
        commit_record = json.loads((project_root / "objects/commits" / f"{branch['head']}.json").read_text(encoding="utf-8"))
        manifest_id = commit_record["payload"]["roots"]["content_manifest"]
        branch["head"] = manifest_id
        branch_path.write_text(json.dumps(branch, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        gc = self.run_cf("gc", project_url, check=False)
        self.assertNotEqual(gc.returncode, 0)
        self.assertIn("gc blocked: invalid sealed commit references: 1", gc.stderr)

    def test_remote_gc_respects_retention_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        self.run_cf("upload", "main", f"{project_url}/main")
        project_root = server / ".codefire-server/projects/org/app"
        (project_root / "server_policy.json").write_text(
            json.dumps({"gc": {"retention_seconds": 3600}}),
            encoding="utf-8",
        )
        orphan_payload = {"type": "policy", "version": 1, "policy": {"retained_orphan": True}}
        orphan_id = self.remote_object_id("policy", orphan_payload)
        orphan_record = {
            "object_id": orphan_id,
            "type": "policy",
            "hash": self.object_digest("policy", orphan_payload),
            "payload": orphan_payload,
        }
        orphan_path = project_root / "objects/policies" / f"{orphan_id}.json"
        orphan_path.parent.mkdir(parents=True, exist_ok=True)
        orphan_path.write_text(json.dumps(orphan_record, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        retained = self.run_cf("gc", project_url)
        self.assertIn("objects_protected: 1", retained.stdout)
        self.assertIn("objects_removed: 0", retained.stdout)
        self.assertTrue(orphan_path.exists())

        old = time.time() - 7200
        os.utime(orphan_path, (old, old))
        removed = self.run_cf("gc", project_url)
        self.assertIn("objects_protected: 0", removed.stdout)
        self.assertIn("objects_removed: 1", removed.stdout)
        self.assertFalse(orphan_path.exists())

    def test_remote_gc_respects_generation_retention_policy(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")
        server = self.tmp / "server"
        project_url = f"cf://{server}/org/app"
        main_url = f"{project_url}/main"
        self.run_cf("upload", "main", main_url)
        project_root = server / ".codefire-server/projects/org/app"

        (project_root / "server_policy.json").write_text(
            json.dumps(
                {
                    "gc": {"retention_generations": 1},
                    "verification": {"required": [{"id": "reject-failed-generation", "command": "exit 9"}]},
                }
            ),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "rejected-generation")
        rejected = self.tmp / "rejected-generation"
        self.run_cf("open", "rejected-generation", str(rejected))
        self.commit_demo_version(rejected, 15, "Rejected generation commit")
        failed_upload = self.run_cf("upload", "rejected-generation", f"{project_url}/rejected-generation", check=False)
        self.assertNotEqual(failed_upload.returncode, 0)
        self.assertIn("server verification failed", failed_upload.stderr)

        protected = self.run_cf("gc", project_url)
        self.assertIn("objects_removed: 0", protected.stdout)
        protected_count = int(next(line.split(": ")[1] for line in protected.stdout.splitlines() if line.startswith("objects_protected:")))
        self.assertGreater(protected_count, 0)

        (project_root / "server_policy.json").write_text(
            json.dumps({"gc": {"retention_generations": 1}}),
            encoding="utf-8",
        )
        self.run_cf("clone", "main", "advance-one")
        advance_one = self.tmp / "advance-one"
        self.run_cf("open", "advance-one", str(advance_one))
        self.commit_demo_version(advance_one, 31, "Advance generation one")
        self.run_cf("upload", "advance-one", main_url)

        self.run_cf("clone", "advance-one", "advance-two")
        advance_two = self.tmp / "advance-two"
        self.run_cf("open", "advance-two", str(advance_two))
        self.commit_demo_version(advance_two, 32, "Advance generation two")
        self.run_cf("upload", "advance-two", main_url)

        removed = self.run_cf("gc", project_url)
        removed_count = int(next(line.split(": ")[1] for line in removed.stdout.splitlines() if line.startswith("objects_removed:")))
        self.assertGreater(removed_count, 0)

    def test_install_script_installs_codefire(self):
        prefix = self.tmp / "prefix"
        proc = subprocess.run(
            [str(INSTALL), "--prefix", str(prefix)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        installed = prefix / "bin/codefire"
        self.assertTrue(installed.exists())
        help_proc = subprocess.run(
            [str(installed), "--help"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(help_proc.returncode, 0, help_proc.stderr)
        self.assertIn("usage: codefire", help_proc.stdout)

    def test_pyproject_installs_codefire_script(self):
        venv = self.tmp / "venv"
        subprocess.run(
            [sys.executable, "-m", "venv", "--system-site-packages", str(venv)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
        )
        pip = venv / "bin/pip"
        installed = venv / "bin/codefire"
        proc = subprocess.run(
            [str(pip), "install", "--no-build-isolation", "--no-deps", str(ROOT)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertTrue(installed.exists())
        help_proc = subprocess.run(
            [str(installed), "--help"],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(help_proc.returncode, 0, help_proc.stderr)
        self.assertIn("usage: codefire", help_proc.stdout)

    def test_completion_command_and_install_completion(self):
        bash = self.run_cf("completion", "bash")
        self.assertIn("_codefire_complete", bash.stdout)
        self.assertIn("complete -F _codefire_complete codefire", bash.stdout)
        self.assertIn("request-apply", bash.stdout)
        self.assertIn("serve", bash.stdout)

        zsh = self.run_cf("completion", "zsh")
        self.assertIn("#compdef codefire", zsh.stdout)
        self.assertIn("_codefire", zsh.stdout)
        self.assertIn("completion\\:completion", zsh.stdout)

        prefix = self.tmp / "prefix"
        completion_dir = self.tmp / "completions"
        proc = subprocess.run(
            [str(INSTALL), "--prefix", str(prefix), "--completion", "bash", "--completion-dir", str(completion_dir)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        installed_completion = completion_dir / "codefire"
        self.assertTrue(installed_completion.exists())
        self.assertIn("_codefire_complete", installed_completion.read_text(encoding="utf-8"))

    def test_no_change_required_requires_rationale(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale", proc.stderr)

    def test_extinguish_policy_can_allow_no_change_required_evidence_only(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        (main / "codefire.policy.yaml").write_text(
            "version: 1\n"
            "extinguish_policy:\n"
            "  no-change-required:\n"
            "    requires_rationale: false\n",
            encoding="utf-8",
        )
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale or --evidence", proc.stderr)
        self.run_cf("extinguish", fire_id, "--resolution", "no-change-required", "--evidence", "reviewed linked artifacts", cwd=main)

    def test_extinguish_requires_rationale_or_evidence(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.write_demo_files(main, 30)
        scan = self.run_cf("scan", cwd=main)
        fire_id = next(line.strip().split()[0] for line in scan.stdout.splitlines() if line.strip().startswith("FIRE-"))
        proc = self.run_cf("extinguish", fire_id, "--resolution", "changed", cwd=main, check=False)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("requires --rationale or --evidence", proc.stderr)
        self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "updated linked artifact", cwd=main)

    def test_clone_open_merge_and_merge_commit(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Change session expiration to 15 minutes")

        self.run_cf("merge", "feature-session", "--into", "main")
        status = self.run_cf("status", cwd=main)
        self.assertIn("State: open-burning", status.stdout)
        merge_scan = self.run_cf("scan", cwd=main)
        self.assertIn("merge_changed", merge_scan.stdout)
        for fire_id in [line.strip().split()[0] for line in merge_scan.stdout.splitlines() if line.strip().startswith("FIRE-")]:
            self.run_cf("extinguish", fire_id, "--resolution", "changed", "--evidence", "merged feature-session", cwd=main)
        self.run_cf("verify", cwd=main)
        merge_commit = self.run_cf("commit", "-m", "Merge feature-session", cwd=main)
        merge_id = next(line.split(":", 1)[1].strip() for line in merge_commit.stdout.splitlines() if line.startswith("Commit:"))
        commit_file = next((self.tmp / ".codefire/objects/commits").glob(f"{merge_id}.json"))
        payload = json.loads(commit_file.read_text())["payload"]
        self.assertEqual(len(payload["parents"]), 2)

    def test_merge_conflict_blocks_verify(self):
        self.run_cf("init")
        main = self.tmp / "main"
        self.run_cf("open", "main", str(main))
        self.commit_demo_version(main, 30, "Initial consistent auth sample")

        self.run_cf("clone", "main", "feature-session")
        feature = self.tmp / "feature-session"
        self.run_cf("open", "feature-session", str(feature))
        self.commit_demo_version(feature, 15, "Feature changes session expiration to 15")

        self.commit_demo_version(main, 45, "Main changes session expiration to 45")

        merge = self.run_cf("merge", "feature-session", "--into", "main")
        self.assertIn("with conflicts", merge.stdout)
        self.assertIn("<<<<<<< target", (main / "src/auth.py").read_text(encoding="utf-8"))
        verify = self.run_cf("verify", cwd=main, check=False)
        self.assertNotEqual(verify.returncode, 0)
        self.assertIn("Failed checks:", verify.stdout)


if __name__ == "__main__":
    unittest.main()
