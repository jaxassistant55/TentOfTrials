#!/usr/bin/env python3

import tempfile
import unittest
from pathlib import Path

import build


class DiagnosticCleanupTests(unittest.TestCase):
    def test_dry_run_reports_stale_artifacts_without_deleting(self):
        with tempfile.TemporaryDirectory() as tmp:
            diagnostic_dir = Path(tmp)
            stale_names = [
                "build-22222222.logd",
                "build-22222222-part001.logd",
                "build-22222222.json",
                "build-22222222-metadata.json",
            ]
            current_names = [
                "build-11111111.logd",
                "build-11111111-part001.logd",
                "build-11111111.json",
            ]
            for name in stale_names + current_names + ["notes.txt"]:
                (diagnostic_dir / name).write_text(name, encoding="utf-8")

            result = build.cleanup_stale_diagnostic_artifacts(
                diagnostic_dir,
                current_commit="11111111",
                apply=False,
            )

            self.assertEqual(
                [entry.path.name for entry in result.stale],
                sorted(stale_names),
            )
            self.assertFalse(result.deleted)
            for name in stale_names + current_names + ["notes.txt"]:
                self.assertTrue((diagnostic_dir / name).exists())

    def test_apply_deletes_only_stale_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            diagnostic_dir = Path(tmp)
            stale = diagnostic_dir / "build-22222222-part001.logd"
            stale_json = diagnostic_dir / "build-22222222.json"
            current = diagnostic_dir / "build-11111111-part001.logd"
            ignored = diagnostic_dir / "manual.logd"
            for path in [stale, stale_json, current, ignored]:
                path.write_text(path.name, encoding="utf-8")

            result = build.cleanup_stale_diagnostic_artifacts(
                diagnostic_dir,
                current_commit="11111111",
                apply=True,
            )

            self.assertEqual({path.name for path in result.deleted}, {stale.name, stale_json.name})
            self.assertFalse(stale.exists())
            self.assertFalse(stale_json.exists())
            self.assertTrue(current.exists())
            self.assertTrue(ignored.exists())

    def test_missing_diagnostic_directory_is_empty_result(self):
        with tempfile.TemporaryDirectory() as tmp:
            missing_dir = Path(tmp) / "diagnostic"

            result = build.cleanup_stale_diagnostic_artifacts(
                missing_dir,
                current_commit="11111111",
                apply=True,
            )

            self.assertEqual(result.current_commit, "11111111")
            self.assertFalse(result.stale)
            self.assertFalse(result.deleted)

    def test_scan_can_preserve_diagnostic_commit_target(self):
        with tempfile.TemporaryDirectory() as tmp:
            diagnostic_dir = Path(tmp)
            implementation = diagnostic_dir / "build-22222222.logd"
            head = diagnostic_dir / "build-11111111.json"
            stale = diagnostic_dir / "build-33333333.logd"
            for path in [implementation, head, stale]:
                path.write_text(path.name, encoding="utf-8")

            result = build.scan_stale_diagnostic_artifacts(
                diagnostic_dir,
                current_commit="11111111",
                protected_commits={"11111111", "22222222"},
            )

            self.assertEqual([entry.path.name for entry in result.stale], [stale.name])
            self.assertEqual(
                {path.name for path in result.preserved_current},
                {implementation.name, head.name},
            )


if __name__ == "__main__":
    unittest.main()
