import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from evoagent.nameaudit import (
    NameAuditRun,
    NameCiJson,
    NameCiJsonRun,
    NameCiNote,
    NameCiNoteMd,
    NameCiNoteMdRun,
    NameCiNoteSec,
    NameCiNoteSecRun,
    NameCiNoteTop,
    NameCiNoteTopRun,
    NameCiNoteRun,
    NameCiPack,
    NameCiPackRun,
    NameCiRun,
    NameCiStatusJson,
    NameCiStatusRun,
    NameCiSummary,
    NameCiSummaryRun,
    NameDash,
    NameDashJson,
    NameDashJsonRun,
    NameDashRun,
    NameFiles,
    NameFixJson,
    NameFixJsonRun,
    NameFull,
    NameFullRun,
    NameGate,
    NameGateExit,
    NameGateExitRun,
    NameGateJson,
    NameGateJsonRun,
    NameGateLine,
    NameGateLineRun,
    NameGateMd,
    NameGatePack,
    NameGatePackRun,
    NameGateRun,
    NameJson,
    NameJsonRun,
    NameKindMd,
    NameKinds,
    NameLenAudit,
    NamePlan,
    NameOpsGuide,
    NameProjOpsDoc,
    NamePjDashJson,
    NameProjCiJsonRun,
    NameProjCiNoteMdRun,
    NameProjCiNoteSecRun,
    NameProjCiNoteRun,
    NameProjCiNoteTopRun,
    NameProjCiPackRun,
    NameProjCiStatusRun,
    NameProjCiSummaryRun,
    NameProjCiRun,
    NameProjGate,
    NameProjGateExit,
    NameProjGateJson,
    NameProjGateLine,
    NameProjGatePack,
    NameProjDash,
    NameProjFixJson,
    NameProjFull,
    NameProjJson,
    NameProjTop,
    NameProjTopJson,
    NameProjRun,
    NameReport,
    NameStatJson,
    NameStatMd,
    NameStats,
    NameTop,
    NameTopJson,
    NameTopJsonRun,
    NameTopMd,
    NameTopRun,
)


class AuditTests(unittest.TestCase):
    # cf-atom: TEST-name-len-audit-flags-long-names
    def test_long_names(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        overly_long_local_name = 1
        return overly_long_local_name + very_long_argument_name
"""

        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        self.assertEqual(
            [hit.name for hit in hits],
            [
                "ExcessivelyLongClassName",
                "excessively_long_method",
                "very_long_argument_name",
                "overly_long_local_name",
            ],
        )
        self.assertEqual(hits[0].path, "sample.py")
        self.assertEqual(hits[0].kind, "ClassDef")

        default_src = """
class ABCDEFGHIJKLMNOPQRSTUVWXY:
    pass
class ABCDEFGHIJKLMNOPQRSTUVWXYZ:
    pass
"""
        default_hits = NameLenAudit().scan_text(default_src, path="sample.py")

        self.assertEqual([hit.name for hit in default_hits], ["ABCDEFGHIJKLMNOPQRSTUVWXYZ"])

    def test_limit_ok(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "ok.py"
            path.write_text("class ShortName:\n    pass\n", encoding="utf-8")

            hits = NameLenAudit(max_len=15).scan_paths((path,))

        self.assertEqual(hits, ())

    # cf-atom: TEST-name-plan-suggests-short-names
    def test_plan(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        return very_long_argument_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        fixes = NamePlan(max_len=15).plan(hits)

        self.assertEqual([fix.new_name for fix in fixes], ["excLoClNa", "excLoMe", "verLoArNa"])
        self.assertTrue(all(len(fix.new_name) <= 15 for fix in fixes))
        self.assertEqual(fixes[0].reason, "24>15")

    # cf-atom: TEST-name-report-renders-table
    def test_report(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        fixes = NamePlan(max_len=15).plan(hits)

        report = NameReport().render(fixes)

        self.assertIn("| Path | Line | Kind | Name | Suggestion | Reason |", report)
        self.assertIn("| sample.py | 1 | ClassDef | ExcessivelyLongClassName | excLoClNa | 24>15 |", report)

    # cf-atom: TEST-name-fix-json-renders-fixes
    def test_fix_json(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        fixes = NamePlan(max_len=15).plan(hits)

        data = json.loads(NameFixJson().render(fixes))

        self.assertEqual(data[0]["path"], "sample.py")
        self.assertEqual(data[0]["kind"], "ClassDef")
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["suggestion"], "excLoClNa")
        self.assertEqual(data[0]["reason"], "24>15")

    # cf-atom: TEST-name-fix-json-run-renders-paths
    def test_fj_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameFixJsonRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(data[0]["path"], str(path))
        self.assertEqual(data[0]["kind"], "ClassDef")
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["suggestion"], "excLoClNa")

    # cf-atom: TEST-name-proj-fix-json-renders-root
    def test_proj_fj(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjFixJson(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(data[0]["path"], str(path))
        self.assertEqual(data[0]["kind"], "ClassDef")
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["suggestion"], "excLoClNa")

    # cf-atom: TEST-name-audit-run-renders-paths
    def test_run(self) -> None:
        with TemporaryDirectory() as tmp:
            first = Path(tmp) / "one.py"
            second = Path(tmp) / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            report = NameAuditRun(max_len=15).render((first, second))

        self.assertIn("ExcessivelyLongClassName", report)
        self.assertIn("excessively_long_method", report)
        self.assertIn("excLoClNa", report)
        self.assertIn("excLoMe", report)

    # cf-atom: TEST-name-files-discovers-python
    def test_files(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            keep = root / "pkg"
            skip = root / "__pycache__"
            keep.mkdir()
            skip.mkdir()
            first = keep / "one.py"
            second = root / "two.txt"
            cached = skip / "cached.py"
            first.write_text("x = 1\n", encoding="utf-8")
            second.write_text("x = 1\n", encoding="utf-8")
            cached.write_text("x = 1\n", encoding="utf-8")

            files = NameFiles().discover((root,))

        self.assertEqual(files, (first,))

    # cf-atom: TEST-name-proj-run-renders-root
    def test_proj_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            report = NameProjRun(max_len=15).render((root,))

        self.assertIn("ExcessivelyLongClassName", report)
        self.assertIn("excLoClNa", report)

    # cf-atom: TEST-name-stats-summarizes-hits
    def test_stats(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        stat = NameStats().summarize(hits)

        self.assertEqual(stat.hits, 2)
        self.assertEqual(stat.files, 1)
        self.assertEqual(stat.max_len, 24)
        self.assertEqual(stat.max_name, "ExcessivelyLongClassName")

    # cf-atom: TEST-name-gate-blocks-hits
    def test_gate(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        res = NameGate().check(hits)
        clean = NameGate().check(())

        self.assertFalse(res.ok)
        self.assertEqual(res.stat.hits, 1)
        self.assertTrue(clean.ok)
        self.assertEqual(clean.stat.hits, 0)

    # cf-atom: TEST-name-gate-exit-codes-result
    def test_gate_exit(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        fail = NameGate().check(hits)
        clean = NameGate().check(())

        self.assertEqual(NameGateExit().code(fail), 1)
        self.assertEqual(NameGateExit().code(clean), 0)

    # cf-atom: TEST-name-gate-exit-run-codes-paths
    def test_gate_exit_run(self) -> None:
        with TemporaryDirectory() as tmp:
            bad = Path(tmp) / "bad.py"
            ok = Path(tmp) / "ok.py"
            bad.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            ok.write_text("class ShortName:\n    pass\n", encoding="utf-8")

            runner = NameGateExitRun(max_len=15)
            fail = runner.code((bad,))
            clean = runner.code((ok,))

        self.assertEqual(fail, 1)
        self.assertEqual(clean, 0)

    # cf-atom: TEST-name-gate-json-renders-result
    def test_gate_json(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        res = NameGate().check(hits)

        data = json.loads(NameGateJson().render(res))

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["summary"]["max_len"], 24)

    # cf-atom: TEST-name-gate-pack-renders-hits
    def test_gate_pack(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        data = json.loads(NameGatePack().render(hits))

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["hits"][0]["path"], "sample.py")
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-gate-md-renders-result
    def test_gate_md(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        res = NameGate().check(hits)

        text = NameGateMd().render(res)

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| status | fail |", text)
        self.assertIn("| hits | 1 |", text)
        self.assertIn("| max_name | ExcessivelyLongClassName |", text)

    # cf-atom: TEST-name-gate-line-renders-result
    def test_gate_line(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        res = NameGate().check(hits)

        text = NameGateLine().render(res)

        self.assertEqual(text, "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName")
        self.assertEqual(NameGateLine().render(NameGate().check(())), "status=pass hits=0 files=0 max_len=0 max_name=none")

    # cf-atom: TEST-name-gate-run-renders-paths
    def test_gate_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameGateRun(max_len=15).render((path,))

        self.assertIn("| status | fail |", text)
        self.assertIn("| hits | 1 |", text)
        self.assertIn("| max_name | ExcessivelyLongClassName |", text)

    # cf-atom: TEST-name-gate-line-run-renders-paths
    def test_gate_line_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameGateLineRun(max_len=15).render((path,))

        self.assertEqual(text, f"status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName")

    # cf-atom: TEST-name-ci-run-returns-code-line
    def test_ci_run(self) -> None:
        with TemporaryDirectory() as tmp:
            bad = Path(tmp) / "bad.py"
            ok = Path(tmp) / "ok.py"
            bad.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            ok.write_text("class ShortName:\n    pass\n", encoding="utf-8")

            runner = NameCiRun(max_len=15)
            fail = runner.run((bad,))
            clean = runner.run((ok,))

        self.assertEqual(fail, (1, "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName"))
        self.assertEqual(clean, (0, "status=pass hits=0 files=0 max_len=0 max_name=none"))

    # cf-atom: TEST-name-ci-json-renders-result
    def test_ci_json(self) -> None:
        text = NameCiJson().render(1, "status=fail hits=1")
        data = json.loads(text)

        self.assertEqual(data, {"exit_code": 1, "line": "status=fail hits=1"})
        self.assertEqual(text, '{"exit_code": 1, "line": "status=fail hits=1"}')

    # cf-atom: TEST-name-ci-status-json-renders-ok
    def test_ci_status_json(self) -> None:
        fail = NameCiStatusJson().render(1, "status=fail hits=1")
        clean = NameCiStatusJson().render(0, "status=pass hits=0")

        self.assertEqual(json.loads(fail), {"exit_code": 1, "line": "status=fail hits=1", "ok": False})
        self.assertEqual(json.loads(clean), {"exit_code": 0, "line": "status=pass hits=0", "ok": True})
        self.assertEqual(fail, '{"exit_code": 1, "line": "status=fail hits=1", "ok": false}')

    # cf-atom: TEST-name-ci-pack-renders-hits
    def test_ci_pack(self) -> None:
        hit = NameLenAudit(max_len=15).scan_text("class ExcessivelyLongClassName:\n    pass\n", path="sample.py")[0]

        text = NameCiPack().render(1, "status=fail hits=1", (hit,))
        data = json.loads(text)

        self.assertEqual(data["exit_code"], 1)
        self.assertFalse(data["ok"])
        self.assertEqual(data["line"], "status=fail hits=1")
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data["hits"][0]["length"], 24)

    # cf-atom: TEST-name-ci-summary-renders-aggregate-json
    def test_ci_summary(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        return very_long_argument_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        data = json.loads(NameCiSummary().render(hits))

        self.assertEqual(data["summary"]["hits"], 3)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["files"], [{"count": 3, "path": "sample.py"}])
        self.assertEqual(data["kinds"][0], {"count": 1, "kind": "ClassDef"})
        self.assertEqual(data["longest"]["name"], "ExcessivelyLongClassName")
        self.assertIsNone(json.loads(NameCiSummary().render(()))["longest"])

    # cf-atom: TEST-name-ci-note-renders-hits
    def test_ci_note(self) -> None:
        hit = NameLenAudit(max_len=15).scan_text("class ExcessivelyLongClassName:\n    pass\n", path="sample.py")[0]

        text = NameCiNote().render((hit,))

        self.assertEqual(text, "sample.py:1: ClassDef ExcessivelyLongClassName length=24")
        self.assertEqual(NameCiNote().render(()), "clean")

    # cf-atom: TEST-name-ci-note-top-limits-hits
    def test_ci_note_top(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        overly_long_local_name = 1
        return overly_long_local_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        text = NameCiNoteTop().render(hits, limit=1)

        self.assertEqual(text, "sample.py:2: ClassDef ExcessivelyLongClassName length=24\nomitted=2")
        self.assertEqual(NameCiNoteTop().render(()), "clean")

    # cf-atom: TEST-name-ci-note-md-renders-fence
    def test_ci_note_md(self) -> None:
        text = NameCiNoteMd().render("sample.py:1: ClassDef LongName length=24")

        self.assertEqual(text, "```text\nsample.py:1: ClassDef LongName length=24\n```")
        self.assertEqual(NameCiNoteMd().render("clean\n"), "```text\nclean\n```")

    # cf-atom: TEST-name-ci-note-sec-renders-section
    def test_ci_note_sec(self) -> None:
        text = NameCiNoteSec().render("Naming audit", "sample.py:1: ClassDef LongName length=24")

        self.assertEqual(text, "### Naming audit\n\n```text\nsample.py:1: ClassDef LongName length=24\n```")
        self.assertEqual(NameCiNoteSec().render("  ", "clean"), "### Identifier naming notes\n\n```text\nclean\n```")

    # cf-atom: TEST-name-ci-status-run-renders-paths
    def test_ci_status_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameCiStatusRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(
            data,
            {
                "exit_code": 1,
                "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName",
                "ok": False,
            },
        )
        self.assertEqual(
            text,
            '{"exit_code": 1, "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName", "ok": false}',
        )

    # cf-atom: TEST-name-ci-pack-run-renders-paths
    def test_ci_pack_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameCiPackRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(data["exit_code"], 1)
        self.assertFalse(data["ok"])
        self.assertEqual(data["line"], "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName")
        self.assertEqual(data["hits"][0]["path"], str(path))
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-ci-summary-run-renders-paths
    def test_ci_summary_run(self) -> None:
        with TemporaryDirectory() as tmp:
            first = Path(tmp) / "one.py"
            second = Path(tmp) / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            text = NameCiSummaryRun(max_len=15).render((first, second))
            data = json.loads(text)

        self.assertEqual(data["summary"]["hits"], 2)
        self.assertEqual(data["summary"]["files"], 2)
        self.assertEqual(data["files"][0]["count"], 1)
        self.assertEqual(data["longest"]["path"], str(first))

    # cf-atom: TEST-name-ci-note-run-renders-paths
    def test_ci_note_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameCiNoteRun(max_len=15).render((path,))

        self.assertEqual(text, f"{path}:1: ClassDef ExcessivelyLongClassName length=24")

    # cf-atom: TEST-name-ci-note-top-run-renders-paths
    def test_ci_note_top_run(self) -> None:
        src = (
            "class ExcessivelyLongClassName:\n"
            "    def excessively_long_method(self):\n"
            "        overly_long_local_name = 1\n"
        )
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text(src, encoding="utf-8")

            text = NameCiNoteTopRun(max_len=15).render((path,), limit=1)

        self.assertEqual(text, f"{path}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=2")

    # cf-atom: TEST-name-ci-note-md-run-renders-paths
    def test_ci_note_md_run(self) -> None:
        src = (
            "class ExcessivelyLongClassName:\n"
            "    def excessively_long_method(self):\n"
            "        overly_long_local_name = 1\n"
        )
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text(src, encoding="utf-8")

            text = NameCiNoteMdRun(max_len=15).render((path,), limit=1)

        self.assertEqual(text, f"```text\n{path}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=2\n```")

    # cf-atom: TEST-name-ci-note-sec-run-renders-paths
    def test_ci_note_sec_run(self) -> None:
        src = (
            "class ExcessivelyLongClassName:\n"
            "    def excessively_long_method(self):\n"
            "        overly_long_local_name = 1\n"
        )
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text(src, encoding="utf-8")

            text = NameCiNoteSecRun(max_len=15).render((path,), "Naming audit", limit=1)

        self.assertEqual(
            text,
            f"### Naming audit\n\n```text\n{path}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=2\n```",
        )

    # cf-atom: TEST-name-ci-json-run-renders-paths
    def test_ci_json_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameCiJsonRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(
            data,
            {
                "exit_code": 1,
                "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName",
            },
        )
        self.assertEqual(
            text,
            '{"exit_code": 1, "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName"}',
        )

    # cf-atom: TEST-name-proj-ci-json-run-renders-root
    def test_pj_ci_json_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjCiJsonRun(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(
            data,
            {
                "exit_code": 1,
                "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName",
            },
        )
        self.assertEqual(
            text,
            '{"exit_code": 1, "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName"}',
        )

    # cf-atom: TEST-name-proj-ci-status-run-renders-root
    def test_pj_ci_status_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjCiStatusRun(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(
            data,
            {
                "exit_code": 1,
                "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName",
                "ok": False,
            },
        )
        self.assertEqual(
            text,
            '{"exit_code": 1, "line": "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName", "ok": false}',
        )

    # cf-atom: TEST-name-proj-ci-pack-run-renders-root
    def test_pj_ci_pack_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjCiPackRun(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(data["exit_code"], 1)
        self.assertFalse(data["ok"])
        self.assertEqual(data["line"], "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName")
        self.assertEqual(data["hits"][0]["path"], str(path))
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-proj-ci-summary-run-renders-root
    def test_pj_ci_summary_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            first = pkg / "one.py"
            second = root / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            text = NameProjCiSummaryRun(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(data["summary"]["hits"], 2)
        self.assertEqual(data["summary"]["files"], 2)
        self.assertEqual(data["longest"]["name"], "ExcessivelyLongClassName")
        self.assertEqual({item["path"] for item in data["files"]}, {str(first), str(second)})

    # cf-atom: TEST-name-proj-ci-note-run-renders-root
    def test_pj_ci_note_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjCiNoteRun(max_len=15).render((root,))

        self.assertEqual(text, f"{path}:1: ClassDef ExcessivelyLongClassName length=24")

    # cf-atom: TEST-name-proj-ci-note-top-run-renders-root
    def test_pj_ci_note_top_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            first = root / "one.py"
            second = root / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            text = NameProjCiNoteTopRun(max_len=15).render((root,), limit=1)

        self.assertEqual(text, f"{first}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=1")

    # cf-atom: TEST-name-proj-ci-note-md-run-renders-root
    def test_pj_ci_note_md_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            first = root / "one.py"
            second = root / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            text = NameProjCiNoteMdRun(max_len=15).render((root,), limit=1)

        self.assertEqual(text, f"```text\n{first}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=1\n```")

    # cf-atom: TEST-name-proj-ci-note-sec-run-renders-root
    def test_pj_ci_note_sec_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            first = root / "one.py"
            second = root / "two.py"
            first.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")
            second.write_text("def excessively_long_method():\n    return 1\n", encoding="utf-8")

            text = NameProjCiNoteSecRun(max_len=15).render((root,), "Naming audit", limit=1)

        self.assertEqual(
            text,
            f"### Naming audit\n\n```text\n{first}:1: ClassDef ExcessivelyLongClassName length=24\nomitted=1\n```",
        )

    # cf-atom: TEST-name-ops-guide-renders-workflow
    def test_ops_guide(self) -> None:
        text = NameOpsGuide().render(max_len=18, limit=3)

        self.assertIn("# Identifier Naming Audit", text)
        self.assertIn("- Maximum identifier length: 18", text)
        self.assertIn("- Review note limit: 3", text)
        self.assertIn("NameProjCiNoteSecRun", text)

    # cf-atom: TEST-name-proj-ops-doc-renders-current-section
    def test_pj_ops_doc(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjOpsDoc(max_len=15).render((root,), limit=1)

        self.assertIn("# Identifier Naming Audit", text)
        self.assertIn("### Current naming findings", text)
        self.assertIn(f"{path}:1: ClassDef ExcessivelyLongClassName length=24", text)

    # cf-atom: TEST-name-gate-json-run-renders-paths
    def test_gj_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameGateJsonRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["summary"]["max_name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-gate-pack-run-renders-paths
    def test_pack_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameGatePackRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["hits"][0]["path"], str(path))
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-proj-gate-renders-root
    def test_proj_gate(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjGate(max_len=15).render((root,))

        self.assertIn("| status | fail |", text)
        self.assertIn("| hits | 1 |", text)
        self.assertIn("| max_name | ExcessivelyLongClassName |", text)

    # cf-atom: TEST-name-proj-gate-json-renders-root
    def test_pj_gate_json(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjGateJson(max_len=15).render((root,))
            data = json.loads(text)

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["summary"]["max_name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-proj-gate-pack-renders-root
    def test_pj_gate_pack(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjGatePack(max_len=15).render((root,))
            data = json.loads(text)

        self.assertFalse(data["ok"])
        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["hits"][0]["path"], str(path))
        self.assertEqual(data["hits"][0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-proj-gate-exit-codes-root
    def test_pj_gate_exit(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            fail = NameProjGateExit(max_len=15).code((root,))
            clean = NameProjGateExit(max_len=25).code((root,))

        self.assertEqual(fail, 1)
        self.assertEqual(clean, 0)

    # cf-atom: TEST-name-proj-gate-line-renders-root
    def test_pj_gate_line(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjGateLine(max_len=15).render((root,))

        self.assertEqual(text, "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName")

    # cf-atom: TEST-name-proj-ci-run-returns-code-line
    def test_pj_ci_run(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            fail = NameProjCiRun(max_len=15).run((root,))
            clean = NameProjCiRun(max_len=25).run((root,))

        self.assertEqual(fail, (1, "status=fail hits=1 files=1 max_len=24 max_name=ExcessivelyLongClassName"))
        self.assertEqual(clean, (0, "status=pass hits=0 files=0 max_len=0 max_name=none"))

    # cf-atom: TEST-name-stat-md-renders-summary
    def test_stat_md(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        stat = NameStats().summarize(hits)

        text = NameStatMd().render(stat)

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| hits | 1 |", text)
        self.assertIn("| max_name | ExcessivelyLongClassName |", text)

    # cf-atom: TEST-name-stat-json-renders-summary
    def test_stat_json(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        stat = NameStats().summarize(hits)

        data = json.loads(NameStatJson().render(stat))

        self.assertEqual(data["hits"], 1)
        self.assertEqual(data["files"], 1)
        self.assertEqual(data["max_len"], 24)
        self.assertEqual(data["max_name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-kinds-groups-hits
    def test_kinds(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        overly_long_local_name = 1
        return overly_long_local_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        rows = NameKinds().group(hits)

        self.assertEqual(
            rows,
            (
                ("ClassDef", 1),
                ("FunctionDef", 1),
                ("arg", 1),
                ("name", 1),
            ),
        )

    # cf-atom: TEST-name-kind-md-renders-table
    def test_kind_md(self) -> None:
        rows = (("name", 3), ("arg", 1))

        text = NameKindMd().render(rows)

        self.assertIn("| Kind | Count |", text)
        self.assertIn("| name | 3 |", text)
        self.assertIn("| arg | 1 |", text)

    # cf-atom: TEST-name-dash-renders-summary
    def test_dash(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        text = NameDash().render(hits)

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| Kind | Count |", text)
        self.assertIn("| ClassDef | 1 |", text)

    # cf-atom: TEST-name-dash-json-renders-summary
    def test_dash_json(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        data = json.loads(NameDashJson().render(hits))

        self.assertEqual(data["summary"]["hits"], 2)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["summary"]["max_len"], 24)
        self.assertEqual(data["kinds"][0]["kind"], "ClassDef")

    # cf-atom: TEST-name-dash-json-run-renders-paths
    def test_dj_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameDashJsonRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["kinds"][0]["kind"], "ClassDef")

    # cf-atom: TEST-name-dash-run-renders-paths
    def test_dash_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameDashRun(max_len=15).render((path,))

        self.assertIn("| hits | 1 |", text)
        self.assertIn("| ClassDef | 1 |", text)

    # cf-atom: TEST-name-proj-dash-renders-root
    def test_proj_dash(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjDash(max_len=15).render((root,))

        self.assertIn("| hits | 1 |", text)
        self.assertIn("| ClassDef | 1 |", text)

    # cf-atom: TEST-name-pj-dash-json-renders-root
    def test_pj_dj(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NamePjDashJson(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(data["summary"]["hits"], 1)
        self.assertEqual(data["summary"]["files"], 1)
        self.assertEqual(data["kinds"][0]["kind"], "ClassDef")

    # cf-atom: TEST-name-top-ranks-hits
    def test_top(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        overly_long_local_name = 1
        return overly_long_local_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        rows = NameTop().rank(hits, limit=2)

        self.assertEqual([row.name for row in rows], ["ExcessivelyLongClassName", "excessively_long_method"])

    # cf-atom: TEST-name-top-md-renders-table
    def test_top_md(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")
        rows = NameTop().rank(hits)

        text = NameTopMd().render(rows)

        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("| sample.py | 1 | ClassDef | ExcessivelyLongClassName | 24 |", text)

    # cf-atom: TEST-name-top-json-renders-ranked-hits
    def test_top_json(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self, very_long_argument_name):
        return very_long_argument_name
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        data = json.loads(NameTopJson().render(hits, limit=2))

        self.assertEqual(len(data), 2)
        self.assertEqual([row["name"] for row in data], ["ExcessivelyLongClassName", "excessively_long_method"])
        self.assertEqual(data[0]["length"], 24)

    # cf-atom: TEST-name-top-json-run-renders-paths
    def test_tj_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameTopJsonRun(max_len=15, limit=1).render((path,))
            data = json.loads(text)

        self.assertEqual(len(data), 1)
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["length"], 24)

    # cf-atom: TEST-name-top-run-renders-paths
    def test_top_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameTopRun(max_len=15, limit=1).render((path,))

        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("ExcessivelyLongClassName", text)

    # cf-atom: TEST-name-proj-top-renders-root
    def test_proj_top(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjTop(max_len=15, limit=1).render((root,))

        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("ExcessivelyLongClassName", text)

    # cf-atom: TEST-name-proj-top-json-renders-root
    def test_pj_top(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjTopJson(max_len=15, limit=1).render((root,))
            data = json.loads(text)

        self.assertEqual(len(data), 1)
        self.assertEqual(data[0]["path"], str(path))
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")

    # cf-atom: TEST-name-full-renders-report
    def test_full(self) -> None:
        src = """
class ExcessivelyLongClassName:
    def excessively_long_method(self):
        return 1
"""
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        text = NameFull(limit=1).render(hits)

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| Kind | Count |", text)
        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("ExcessivelyLongClassName", text)

    # cf-atom: TEST-name-full-run-renders-paths
    def test_full_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameFullRun(max_len=15, limit=1).render((path,))

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("ExcessivelyLongClassName", text)

    # cf-atom: TEST-name-proj-full-renders-root
    def test_proj_full(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjFull(max_len=15, limit=1).render((root,))

        self.assertIn("| Metric | Value |", text)
        self.assertIn("| Path | Line | Kind | Name | Length |", text)
        self.assertIn("ExcessivelyLongClassName", text)

    # cf-atom: TEST-name-json-renders-hits
    def test_json(self) -> None:
        src = "class ExcessivelyLongClassName:\n    pass\n"
        hits = NameLenAudit(max_len=15).scan_text(src, path="sample.py")

        text = NameJson().render(hits)
        data = json.loads(text)

        self.assertEqual(data[0]["path"], "sample.py")
        self.assertEqual(data[0]["line"], 1)
        self.assertEqual(data[0]["kind"], "ClassDef")
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["length"], 24)

    # cf-atom: TEST-name-json-run-renders-paths
    def test_json_run(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameJsonRun(max_len=15).render((path,))
            data = json.loads(text)

        self.assertEqual(data[0]["path"], str(path))
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["length"], 24)

    # cf-atom: TEST-name-proj-json-renders-root
    def test_proj_json(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            pkg = root / "pkg"
            pkg.mkdir()
            path = pkg / "mod.py"
            path.write_text("class ExcessivelyLongClassName:\n    pass\n", encoding="utf-8")

            text = NameProjJson(max_len=15).render((root,))
            data = json.loads(text)

        self.assertEqual(data[0]["path"], str(path))
        self.assertEqual(data[0]["name"], "ExcessivelyLongClassName")
        self.assertEqual(data[0]["length"], 24)


if __name__ == "__main__":
    unittest.main()
