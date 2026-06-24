"""Tests for the diagnostic retention report feature in build.py."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

# Import the retention_report function directly from build.py
# We add the project root to sys.path so we can import build as a module
ROOT = Path(__file__).resolve().parent.parent  # project root (one level up from tests/)
sys.path.insert(0, str(ROOT))

import build


class TestRetentionReport(unittest.TestCase):
    """Test the retention_report function with a temporary diagnostic directory."""

    def setUp(self):
        """Create a temporary directory with fake diagnostic artifacts."""
        self.tmpdir = tempfile.mkdtemp(prefix="tent-retention-test-")
        self.diag_dir = Path(self.tmpdir) / "diagnostic"
        self.diag_dir.mkdir()

        # Create a "current commit" artifact pair
        (self.diag_dir / "build-abcdef01.logd").write_bytes(b"x" * 100)
        (self.diag_dir / "build-abcdef01.json").write_text('{"ok": true}')

        # Create an "older commit" artifact pair
        (self.diag_dir / "build-deadbeef.logd").write_bytes(b"y" * 200)
        (self.diag_dir / "build-deadbeef.json").write_text('{"ok": false}')

        # Create a chunked artifact from another old commit
        (self.diag_dir / "build-cafebabe-part001.logd").write_bytes(b"z" * 50)

    def tearDown(self):
        """Remove the temporary directory."""
        import shutil
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_report_lists_current_and_older_artifacts(self):
        """When commit_id matches, current artifacts are separated from older ones."""
        report = build.retention_report(self.diag_dir)
        # We pass a real dir but the commit_id comes from git; we just
        # verify the structure is correct.
        self.assertIn("current_commit", report)
        self.assertIn("current_commit_artifacts", report)
        self.assertIn("older_artifacts", report)
        self.assertIn("total_artifacts", report)
        self.assertIn("total_bytes", report)

    def test_total_artifacts_count(self):
        """Total artifact count should include all build-*.{logd,json} files."""
        report = build.retention_report(self.diag_dir)
        # We created 5 diagnostic files total
        self.assertEqual(report["total_artifacts"], 5)

    def test_total_bytes_sum(self):
        """Total bytes should be the sum of all artifact file sizes."""
        report = build.retention_report(self.diag_dir)
        # 100 + len('{"ok": true}') + 200 + len('{"ok": false}') + 50
        expected = 100 + 12 + 200 + 13 + 50
        self.assertEqual(report["total_bytes"], expected)

    def test_empty_directory(self):
        """An empty diagnostic directory should return zero counts."""
        empty_dir = Path(self.tmpdir) / "empty"
        empty_dir.mkdir()
        report = build.retention_report(empty_dir)
        self.assertEqual(report["total_artifacts"], 0)
        self.assertEqual(report["total_bytes"], 0)
        self.assertEqual(report["current_commit_artifacts"], [])
        self.assertEqual(report["older_artifacts"], [])

    def test_nonexistent_directory(self):
        """A nonexistent directory should return zero counts gracefully."""
        report = build.retention_report(Path("/nonexistent/path"))
        self.assertEqual(report["total_artifacts"], 0)
        self.assertEqual(report["total_bytes"], 0)

    def test_current_commit_artifacts_matched(self):
        """When the current commit ID matches a prefix, those files are current."""
        # Monkey-patch current_commit_id to return our test commit
        original = build.current_commit_id
        build.current_commit_id = lambda: "abcdef01"
        try:
            report = build.retention_report(self.diag_dir)
            self.assertIn("build-abcdef01.logd", report["current_commit_artifacts"])
            self.assertIn("build-abcdef01.json", report["current_commit_artifacts"])
            self.assertEqual(len(report["current_commit_artifacts"]), 2)
            # The other 3 should be older
            self.assertEqual(len(report["older_artifacts"]), 3)
        finally:
            build.current_commit_id = original

    def test_all_older_when_no_current_commit(self):
        """When current_commit_id returns 00000000, all artifacts are older."""
        original = build.current_commit_id
        build.current_commit_id = lambda: "00000000"
        try:
            report = build.retention_report(self.diag_dir)
            self.assertEqual(len(report["current_commit_artifacts"]), 0)
            self.assertEqual(len(report["older_artifacts"]), 5)
            self.assertEqual(report["current_commit"], "unknown")
        finally:
            build.current_commit_id = original

    def test_cli_retention_report(self):
        """The --retention-report CLI flag should output valid JSON."""
        result = subprocess.run(
            [sys.executable, str(ROOT / "build.py"), "--retention-report",
             "--retention-dir", str(self.diag_dir)],
            capture_output=True, text=True, cwd=str(ROOT),
            env={**os.environ, "PYTHONIOENCODING": "utf-8", "PYTHONUTF8": "1"},
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        data = json.loads(result.stdout)
        self.assertIn("total_artifacts", data)
        self.assertIn("total_bytes", data)


if __name__ == "__main__":
    unittest.main()
