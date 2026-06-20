"""
Unit tests for the stale diagnostic cleanup functionality in build.py.
Verifies dry-run behavior, apply deletion, and preservation of the current commit's artifacts.
"""

import unittest
import tempfile
from pathlib import Path
from unittest.mock import patch, io

# Import functions to test from build.py
import build


class TestStaleDiagnosticCleanup(unittest.TestCase):
    """
    Test suite for stale diagnostic artifact cleanup.
    Covers dry-run safety, apply deletion execution, and preservation constraints.
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

        # Generate test files
        self.current_logd = self.diagnostic_dir / f"build-{self.test_commit}.logd"
        self.current_logd.write_text("current logd contents")
        
        self.current_meta = self.diagnostic_dir / f"build-{self.test_commit}-metadata.json"
        self.current_meta.write_text("current metadata contents")
        
        self.stale_logd = self.diagnostic_dir / "build-eeee9999.logd"
        self.stale_logd.write_text("stale logd contents")
        
        self.stale_meta = self.diagnostic_dir / "build-eeee9999-metadata.json"
        self.stale_meta.write_text("stale metadata contents")

    def tearDown(self):
        """Clean up patches and temporary directory."""
        patch.stopall()
        self.test_dir.cleanup()

    def test_dry_run_preserves_all_files(self):
        """Verify that running with apply=False lists stale files but deletes nothing."""
        with patch("sys.stdout", new=io.StringIO()) as fake_out:
            build.cleanup_stale_diagnostics(apply=False)
            output = fake_out.getvalue()
            
        self.assertIn("Dry-run: Stale diagnostic artifacts", output)
        self.assertIn("build-eeee9999.logd", output)
        self.assertIn("build-eeee9999-metadata.json", output)
        
        # Ensure all files still exist
        self.assertTrue(self.current_logd.exists())
        self.assertTrue(self.current_meta.exists())
        self.assertTrue(self.stale_logd.exists())
        self.assertTrue(self.stale_meta.exists())

    def test_apply_deletes_only_stale_files(self):
        """Verify that running with apply=True deletes older/stale files and keeps current commit's files."""
        with patch("sys.stdout", new=io.StringIO()) as fake_out:
            build.cleanup_stale_diagnostics(apply=True)
            output = fake_out.getvalue()
            
        self.assertIn("Removing stale diagnostic artifacts", output)
        self.assertIn("Removed: diagnostic/build-eeee9999.logd", output)
        self.assertIn("Removed: diagnostic/build-eeee9999-metadata.json", output)
        
        # Ensure current commit's artifacts are preserved
        self.assertTrue(self.current_logd.exists())
        self.assertTrue(self.current_meta.exists())
        
        # Ensure stale files are deleted
        self.assertFalse(self.stale_logd.exists())
        self.assertFalse(self.stale_meta.exists())


if __name__ == "__main__":
    unittest.main()
