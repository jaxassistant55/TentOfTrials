#!/usr/bin/env python3
"""Tests for diagnostic artifact retention reporting."""

import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def test_report_retention_empty_directory():
    """Test retention report with empty diagnostic directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        diag_dir = tmp_path / "diagnostic"
        diag_dir.mkdir()

        # Run build.py with --report-retention from the temp dir
        result = subprocess.run(
            [sys.executable, "build.py", "--report-retention"],
            capture_output=True,
            text=True,
            timeout=10,
        )

        if result.returncode == 0:
            try:
                report = json.loads(result.stdout)
                assert report["total_artifact_count"] == 0
                assert report["total_bytes"] == 0
                assert report["current_commit_artifacts"] == []
                assert report["older_artifacts"] == []
                print("✓ test_report_retention_empty_directory PASSED")
                return True
            except json.JSONDecodeError as e:
                print(f"✗ test_report_retention_empty_directory FAILED: Invalid JSON output: {e}")
                print(f"  stdout: {result.stdout}")
                print(f"  stderr: {result.stderr}")
                return False
        else:
            print(f"✗ test_report_retention_empty_directory FAILED: Non-zero return code {result.returncode}")
            print(f"  stdout: {result.stdout}")
            print(f"  stderr: {result.stderr}")
            return False


def test_report_retention_with_artifacts():
    """Test retention report with sample artifacts."""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        diag_dir = tmp_path / "diagnostic"
        diag_dir.mkdir()

        # Create sample artifacts
        # Current commit artifacts (commit ID "abc12345")
        (diag_dir / "build-abc12345.json").write_text(json.dumps({"test": "data"}))
        (diag_dir / "build-abc12345.logd").write_bytes(b"x" * 1024)  # 1 KB

        # Older commit artifacts
        (diag_dir / "build-def67890.json").write_text(json.dumps({"old": "data"}))
        (diag_dir / "build-def67890.logd").write_bytes(b"y" * 2048)  # 2 KB
        (diag_dir / "build-def67890-part001.logd").write_bytes(b"z" * 512)  # 0.5 KB

        # Verify files exist
        artifacts = list(diag_dir.glob("build-*"))
        if len(artifacts) != 5:
            print(f"✗ test_report_retention_with_artifacts FAILED: Expected 5 artifacts, got {len(artifacts)}")
            return False

        # Import and test directly (simpler than subprocess for this test)
        try:
            # Change to repo root and import build module
            import build

            # Mock the current_commit_id to return our test commit
            original_current_commit_id = build.current_commit_id
            build.current_commit_id = lambda: "abc12345"

            # Temporarily override DIAGNOSTIC_DIR
            original_diag_dir = build.DIAGNOSTIC_DIR
            original_root = build.ROOT
            build.DIAGNOSTIC_DIR = diag_dir
            build.ROOT = tmp_path

            report = build.report_retention()

            # Restore
            build.current_commit_id = original_current_commit_id
            build.DIAGNOSTIC_DIR = original_diag_dir
            build.ROOT = original_root

            # Verify report structure
            assert "generated_at" in report
            assert "current_commit" in report
            assert report["current_commit"] == "abc12345"
            assert len(report["current_commit_artifacts"]) == 2  # .json and .logd
            assert len(report["older_artifacts"]) == 3  # .json, .logd, .logd-part001
            assert report["total_artifact_count"] == 5
            # Total bytes: 27 (json) + 1024 + 27 (json) + 2048 + 512 ≈ 3638
            assert report["total_bytes"] > 3600 and report["total_bytes"] < 3700

            print("✓ test_report_retention_with_artifacts PASSED")
            return True
        except Exception as e:
            print(f"✗ test_report_retention_with_artifacts FAILED: {e}")
            import traceback
            traceback.print_exc()
            return False


def test_scan_diagnostic_artifacts():
    """Test the artifact scanning function directly."""
    try:
        import build

        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            diag_dir = tmp_path / "diagnostic"
            diag_dir.mkdir()

            # Create test artifacts
            (diag_dir / "build-test1234.json").write_text('{"test": 1}')
            (diag_dir / "build-test1234.logd").write_bytes(b"test" * 256)  # 1 KB
            (diag_dir / "build-old56789.json").write_text('{"old": 1}')

            # Mock current_commit_id
            original_func = build.current_commit_id
            build.current_commit_id = lambda: "test1234"

            current, older, total, total_bytes = build.scan_diagnostic_artifacts(diag_dir)

            build.current_commit_id = original_func

            # Verify results
            assert len(current) == 2  # .json and .logd for current commit
            assert len(older) == 1  # .json for old commit
            assert total == 3
            assert total_bytes > 1000  # Should have at least 1 KB from the .logd file

            print("✓ test_scan_diagnostic_artifacts PASSED")
            return True
    except Exception as e:
        print(f"✗ test_scan_diagnostic_artifacts FAILED: {e}")
        import traceback
        traceback.print_exc()
        return False


def test_report_retention_json_output():
    """Test that --report-retention produces valid JSON."""
    try:
        result = subprocess.run(
            [sys.executable, "build.py", "--report-retention"],
            capture_output=True,
            text=True,
            timeout=10,
        )

        if result.returncode != 0:
            print(f"✗ test_report_retention_json_output FAILED: Non-zero return code {result.returncode}")
            return False

        report = json.loads(result.stdout)

        # Verify required fields
        required_fields = [
            "generated_at",
            "current_commit",
            "current_commit_artifacts",
            "older_artifacts",
            "total_artifact_count",
            "total_bytes",
            "total_kilobytes",
            "total_megabytes",
        ]

        for field in required_fields:
            if field not in report:
                print(f"✗ test_report_retention_json_output FAILED: Missing field '{field}'")
                return False

        # Verify types
        assert isinstance(report["current_commit"], str)
        assert isinstance(report["current_commit_artifacts"], list)
        assert isinstance(report["older_artifacts"], list)
        assert isinstance(report["total_artifact_count"], int)
        assert isinstance(report["total_bytes"], int)

        print("✓ test_report_retention_json_output PASSED")
        return True
    except json.JSONDecodeError as e:
        print(f"✗ test_report_retention_json_output FAILED: Invalid JSON: {e}")
        return False
    except Exception as e:
        print(f"✗ test_report_retention_json_output FAILED: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == "__main__":
    print("\nRunning diagnostic artifact retention tests...\n")

    tests = [
        test_report_retention_json_output,
        test_scan_diagnostic_artifacts,
        test_report_retention_empty_directory,
        test_report_retention_with_artifacts,
    ]

    results = []
    for test_func in tests:
        try:
            results.append(test_func())
        except Exception as e:
            print(f"✗ {test_func.__name__} FAILED with exception: {e}")
            import traceback
            traceback.print_exc()
            results.append(False)

    passed = sum(results)
    total = len(results)

    print(f"\n{'=' * 60}")
    print(f"Test Results: {passed}/{total} passed")
    print(f"{'=' * 60}\n")

    sys.exit(0 if passed == total else 1)
