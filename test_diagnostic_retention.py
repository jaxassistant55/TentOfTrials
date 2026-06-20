"""
Unit tests for the build diagnostic retention report option in build.py.
Verifies correct identification, count, and size calculations of current and older artifacts.
"""

import json
import unittest
import tempfile
from pathlib import Path
from unittest.mock import patch

# Import functions to test from build.py
import build


class TestDiagnosticRetentionReport(unittest.TestCase):
    """
    Test suite for build diagnostic retention reports.
    Covers correct grouping, counting, and bytes calculation.
    """

    def setUp(self):
        """Set up a temporary diagnostic directory and patch build.DIAGNOSTIC_DIR."""
        self.test_dir = tempfile.TemporaryDirectory()
        self.diagnostic_dir = Path(self.test_dir.name)
        
        # Patch build.DIAGNOSTIC_DIR
        self.patcher_dir = patch("build.DIAGNOSTIC_DIR", self.diagnostic_dir)
        self.patcher_dir.start()
        
        # Mock current commit ID to return a constant test ID
        self.test_commit = "abcd1234"
        self.patcher_commit = patch("build.current_commit_id", return_value=self.test_commit)
        self.patcher_commit.start()

    def tearDown(self):
        """Clean up patches and temporary directory."""
        patch.stopall()
        self.test_dir.cleanup()

    def test_empty_retention_report(self):
        """Verify report returns empty counts and zero bytes when no diagnostic artifacts exist."""
        report = build.get_retention_report()
        self.assertEqual(report["current_commit_artifacts"], [])
        self.assertEqual(report["older_artifacts"], [])
        self.assertEqual(report["total_artifact_count"], 0)
        self.assertEqual(report["total_bytes"], 0)

    def test_populated_retention_report(self):
        """Verify report properly identifies current/stale artifacts and aggregates count and bytes."""
        # 1. Create a current commit artifact (10 bytes)
        current_file = self.diagnostic_dir / f"build-{self.test_commit}.logd"
        current_file.write_text("1234567890")
        
        # 2. Create another current commit artifact (15 bytes)
        current_meta = self.diagnostic_dir / f"build-{self.test_commit}-metadata.json"
        current_meta.write_text("123456789012345")
        
        # 3. Create an older commit artifact (20 bytes)
        older_file = self.diagnostic_dir / "build-eeee9999.logd"
        older_file.write_text("12345678901234567890")
        
        # 4. Create an unrelated file (should not be counted)
        unrelated = self.diagnostic_dir / "random.txt"
        unrelated.write_text("some random content")
        
        report = build.get_retention_report()
        
        # Check current artifacts
        self.assertEqual(
            report["current_commit_artifacts"],
            sorted([f"build-{self.test_commit}.logd", f"build-{self.test_commit}-metadata.json"])
        )
        
        # Check older artifacts
        self.assertEqual(report["older_artifacts"], ["build-eeee9999.logd"])
        
        # Total counts should only include files starting with "build-"
        self.assertEqual(report["total_artifact_count"], 3)
        
        # Total bytes should be 10 + 15 + 20 = 45 bytes
        self.assertEqual(report["total_bytes"], 45)


if __name__ == "__main__":
    unittest.main()
