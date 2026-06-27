"""Tests for dry-run restore validation in the legacy migration tool.

Covers: successful dry run, missing backup, schema mismatch,
missing manifest, invalid manifest JSON, and missing data files.
"""

import json
import os
import tempfile
from pathlib import Path

import pytest

# Add tools directory to path so we can import the module
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "tools"))

from legacy_migration import (
    DryRunRestoreResult,
    dry_run_restore_validation,
)


@pytest.fixture
def backup_dir(tmp_path):
    """Create a temporary backup directory."""
    return str(tmp_path / "migration_backups")


def _create_backup(backup_dir, migration_id, manifest_extra=None, data_files=None):
    """Helper to create a backup directory with a manifest."""
    backup_path = Path(backup_dir) / f"migration_{migration_id}"
    backup_path.mkdir(parents=True, exist_ok=True)

    manifest = {
        "migration_id": migration_id,
        "created_at": "2024-06-01T12:00:00Z",
        "from_version": 1,
        "to_version": 3,
        "script_version": "3.2.0-legacy",
        "files": [],
    }
    if manifest_extra:
        manifest.update(manifest_extra)

    with open(backup_path / "manifest.json", "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2)

    # Create any referenced data files
    if data_files:
        manifest_files = []
        for fname in data_files:
            file_path = backup_path / fname
            file_path.parent.mkdir(parents=True, exist_ok=True)
            file_path.write_text("test data", encoding="utf-8")
            manifest_files.append({"path": fname, "size": 9})
        # Re-write manifest with file references
        manifest["files"] = manifest_files
        with open(backup_path / "manifest.json", "w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)

    return backup_path


class TestDryRunRestoreSuccessful:
    """Tests for successful dry-run restore validation."""

    def test_valid_backup_passes(self, backup_dir):
        """A well-formed backup with matching schema should pass validation."""
        _create_backup(backup_dir, "MIG001", data_files=["data.csv"])
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG001",
            target_schema_version="3",
        )
        assert result.valid is True
        assert result.backup_present is True
        assert result.target_compatible is True
        assert result.schema_version == "3"
        assert len(result.errors) == 0

    def test_validation_includes_metadata(self, backup_dir):
        """Result metadata should include migration and backup details."""
        _create_backup(backup_dir, "MIG002", data_files=["data.csv"])
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG002",
        )
        assert result.metadata["migration_id"] == "MIG002"
        assert result.metadata["from_version"] == 1
        assert result.metadata["to_version"] == 3
        assert result.metadata["file_count"] == 1

    def test_row_counts_and_checksums_populated(self, backup_dir):
        """Validation metadata with row counts and checksums should be surfaced."""
        _create_backup(
            backup_dir,
            "MIG003",
            manifest_extra={
                "validation": {
                    "row_counts": {"users": 1500, "orders": 42000},
                    "checksums": {"users": "abc123", "orders": "def456"},
                },
            },
        )
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG003",
        )
        assert result.valid is True
        assert result.row_counts == {"users": 1500, "orders": 42000}
        assert result.checksums == {"users": "abc123", "orders": "def456"}


class TestDryRunRestoreMissingBackup:
    """Tests for missing backup scenarios."""

    def test_missing_backup_directory(self, backup_dir):
        """A non-existent backup directory should fail with BACKUP_NOT_FOUND."""
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="NONEXISTENT",
        )
        assert result.valid is False
        assert result.backup_present is False
        assert any(e["code"] == "BACKUP_NOT_FOUND" for e in result.errors)

    def test_missing_manifest(self, backup_dir):
        """A backup directory without a manifest should fail with MANIFEST_MISSING."""
        # Create the backup directory but not the manifest
        backup_path = Path(backup_dir) / "migration_MIG004"
        backup_path.mkdir(parents=True, exist_ok=True)
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG004",
        )
        assert result.valid is False
        assert any(e["code"] == "MANIFEST_MISSING" for e in result.errors)

    def test_invalid_manifest_json(self, backup_dir):
        """A manifest with invalid JSON should fail with MANIFEST_INVALID."""
        backup_path = Path(backup_dir) / "migration_MIG005"
        backup_path.mkdir(parents=True, exist_ok=True)
        with open(backup_path / "manifest.json", "w", encoding="utf-8") as f:
            f.write("{invalid json!!!")
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG005",
        )
        assert result.valid is False
        assert any(e["code"] == "MANIFEST_INVALID" for e in result.errors)


class TestDryRunRestoreSchemaMismatch:
    """Tests for schema compatibility issues."""

    def test_schema_version_mismatch(self, backup_dir):
        """Backup and target schema versions that differ should fail with SCHEMA_MISMATCH."""
        _create_backup(backup_dir, "MIG006")
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG006",
            target_schema_version="5",  # backup has to_version=3
        )
        assert result.valid is False
        assert result.target_compatible is False
        assert any(e["code"] == "SCHEMA_MISMATCH" for e in result.errors)

    def test_no_schema_version_in_manifest(self, backup_dir):
        """Missing schema version in manifest should fail with SCHEMA_UNKNOWN when target is specified."""
        _create_backup(backup_dir, "MIG007", manifest_extra={"to_version": None})
        # Remove to_version key entirely
        backup_path = Path(backup_dir) / "migration_MIG007"
        manifest_file = backup_path / "manifest.json"
        with open(manifest_file, "r", encoding="utf-8") as f:
            manifest = json.load(f)
        del manifest["to_version"]
        with open(manifest_file, "w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)

        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG007",
            target_schema_version="3",
        )
        assert result.valid is False
        assert result.target_compatible is False
        assert any(e["code"] == "SCHEMA_UNKNOWN" for e in result.errors)

    def test_unsupported_schema_version(self, backup_dir):
        """An unsupported schema version should fail with SCHEMA_UNSUPPORTED."""
        _create_backup(backup_dir, "MIG008", manifest_extra={"to_version": 99})
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG008",
            target_schema_version="99",
        )
        assert result.valid is False
        assert result.target_compatible is False
        assert any(e["code"] == "SCHEMA_UNSUPPORTED" for e in result.errors)

    def test_no_target_schema_skips_compatibility(self, backup_dir):
        """When no target_schema_version is given, compatibility check is skipped."""
        _create_backup(backup_dir, "MIG009")
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG009",
            target_schema_version=None,
        )
        assert result.valid is True
        assert result.target_compatible is True  # defaults to True when not checked


class TestDryRunRestoreMissingDataFiles:
    """Tests for missing data files referenced in manifest."""

    def test_missing_data_files(self, backup_dir):
        """Referenced data files that don't exist should fail with DATA_FILES_MISSING."""
        # Create backup with file references but don't create the actual files
        backup_path = Path(backup_dir) / "migration_MIG010"
        backup_path.mkdir(parents=True, exist_ok=True)
        manifest = {
            "migration_id": "MIG010",
            "created_at": "2024-06-01T12:00:00Z",
            "from_version": 1,
            "to_version": 3,
            "script_version": "3.2.0-legacy",
            "files": [{"path": "users.csv"}, {"path": "orders.csv"}],
        }
        with open(backup_path / "manifest.json", "w", encoding="utf-8") as f:
            json.dump(manifest, f, indent=2)

        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG010",
        )
        assert result.valid is False
        assert any(e["code"] == "DATA_FILES_MISSING" for e in result.errors)

    def test_no_validation_metadata_warns(self, backup_dir):
        """Absence of validation metadata should produce warnings but not errors."""
        _create_backup(backup_dir, "MIG011", data_files=["data.csv"])
        result = dry_run_restore_validation(
            backup_dir=backup_dir,
            migration_id="MIG011",
        )
        assert result.valid is True
        assert any("No validation metadata" in w for w in result.warnings)
