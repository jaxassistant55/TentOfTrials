#!/usr/bin/env python3
# Tests for diagnostic retention report functionality

import json
import tempfile
import unittest
from pathlib import Path

import build


class RetentionReportTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.diag_dir = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def _create_file(self, name, size=10):
        path = self.diag_dir / name
        path.write_bytes(b"x" * size)
        return path

    def test_empty_directory_returns_empty_report(self):
        report = build.get_retention_report(self.diag_dir)
        self.assertEqual(report["current_artifacts"], [])
        self.assertEqual(report["older_artifacts"], [])
        self.assertEqual(report["total_count"], 0)
        self.assertEqual(report["total_bytes"], 0)

    def test_non_diagnostic_files_are_ignored(self):
        self._create_file("notes.txt")
        self._create_file("readme.md")
        report = build.get_retention_report(self.diag_dir)
        self.assertEqual(report["total_count"], 0)

    def test_reports_only_diagnostic_artifacts(self):
        self._create_file("build-11111111.logd")
        self._create_file("build-11111111.json")
        self._create_file("notes.txt")
        report = build.get_retention_report(self.diag_dir)
        self.assertEqual(report["total_count"], 2)

    def test_artifact_entry_contains_name_size_and_path(self):
        self._create_file("build-aaaaaaaa.logd", 42)
        report = build.get_retention_report(self.diag_dir)
        all_a = report["current_artifacts"] + report["older_artifacts"]
        self.assertGreater(len(all_a), 0)
        entry = all_a[0]
        self.assertIn("name", entry)
        self.assertIn("size", entry)
        self.assertIn("path", entry)

    def test_includes_chunked_logd_parts(self):
        self._create_file("build-11111111-part001.logd")
        self._create_file("build-11111111-part002.logd")
        report = build.get_retention_report(self.diag_dir)
        self.assertEqual(report["total_count"], 2)

    def test_total_bytes_across_multiple_artifacts(self):
        self._create_file("build-11111111.logd", 10)
        self._create_file("build-11111111.json", 20)
        self._create_file("build-22222222.logd", 30)
        report = build.get_retention_report(self.diag_dir)
        self.assertEqual(report["total_bytes"], 60)

    def test_does_not_modify_files_readonly(self):
        self._create_file("build-11111111.logd", 100)
        self._create_file("build-22222222.logd", 100)
        before = {p.name: p.read_bytes() for p in self.diag_dir.iterdir()}
        _ = build.get_retention_report(self.diag_dir)
        after = {p.name: p.read_bytes() for p in self.diag_dir.iterdir()}
        self.assertEqual(before, after)


if __name__ == "__main__":
    unittest.main()
