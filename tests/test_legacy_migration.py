import os
import sys
import json
import yaml
import tempfile
import sqlite3
import pytest
from pathlib import Path
from datetime import datetime, timezone

# Add tools to path
sys.path.insert(0, str(Path(__file__).parent.parent / "tools"))

from legacy_migration import (
    MigrationConfig,
    MigrationEngine,
    MigrationResult,
    MigrationStatus,
    MigrationType,
    DataRecord,
    V1ToV2Transformer,
    V2ToV3Transformer,
    compute_checksum,
    parse_version_string,
    format_duration,
)


def test_parse_version_string():
    assert parse_version_string("v1") == 1
    assert parse_version_string("version_2") == 2
    assert parse_version_string("3") == 3
    with pytest.raises(ValueError):
        parse_version_string("abc")


def test_format_duration():
    assert format_duration(30) == "30.0s"
    assert format_duration(90) == "1.5m"
    assert format_duration(4000) == "1.1h"
    assert format_duration(90000) == "1.0d"


def test_compute_checksum():
    data = {"id": "123", "name": "Alice"}
    checksum1 = compute_checksum(data)
    checksum2 = compute_checksum({"name": "Alice", "id": "123"})
    assert checksum1 == checksum2
    assert len(checksum1) == 64  # SHA-256 length in hex


def test_v1_to_v2_transformer():
    transformer = V1ToV2Transformer()
    # UUID should lose dashes, status convert to int, timestamps to ms
    record = DataRecord(
        id="12345678-abcd-1234-abcd-1234567890ab",
        version=1,
        data={
            "id": "12345678-abcd-1234-abcd-1234567890ab",
            "status": "active",
            "created_at": 1609459200,  # seconds
            "updated_at": 1609459200.0,
        }
    )
    res = transformer.transform(record)
    assert res.version == 2
    assert res.data["id"] == "12345678abcd1234abcd1234567890ab"
    assert res.data["_legacy_uuid"] == "12345678-abcd-1234-abcd-1234567890ab"
    assert res.data["status"] == 1
    assert res.data["created_at"] == 1609459200000
    assert res.data["updated_at"] == 1609459200000


def test_v2_to_v3_transformer():
    transformer = V2ToV3Transformer()
    record = DataRecord(
        id="123",
        version=2,
        data={
            "id": "123",
            "preferences": {"theme": "dark"},
            "created_at": 1609459200000,
        }
    )
    res = transformer.transform(record)
    assert res.version == 3
    assert res.data["created_at"]["timestamp"] == 1609459200000
    assert res.data["created_at"]["timezone"] == "UTC"
    assert res.data["preferences_json"] == {"theme": "dark"}
    assert res.data["deleted_at"] is None
    assert res.data["data_classification"] == "internal"


def test_migration_config_validation():
    config = MigrationConfig(
        migration_id="MIG-001",
        migration_type=MigrationType.DATA,
        from_version=1,
        to_version=2,
        source_connection="mock://",
        target_connection="mock://",
    )
    assert len(config.validate()) == 0

    # Invalid version
    invalid_config = MigrationConfig(
        migration_id="MIG-001",
        migration_type=MigrationType.DATA,
        from_version=9,
        to_version=2,
        source_connection="mock://",
        target_connection="mock://",
    )
    assert len(invalid_config.validate()) > 0


def test_mock_connection_seeding():
    config = MigrationConfig(
        migration_id="MIG-001",
        migration_type=MigrationType.DATA,
        from_version=1,
        to_version=2,
        source_connection="mock://",
        target_connection="mock://",
    )
    engine = MigrationEngine(config)
    conn = engine._get_sqlite_connection("mock://")
    cursor = conn.cursor()
    cursor.execute("SELECT COUNT(*) FROM users")
    count = cursor.fetchone()[0]
    assert count == 4


def test_dry_run_flow_and_report():
    with tempfile.TemporaryDirectory() as tmpdir:
        config = MigrationConfig(
            migration_id="MIG-DRYRUN-TEST",
            migration_type=MigrationType.DATA,
            from_version=1,
            to_version=2,
            source_connection="mock://",
            target_connection="mock://",
            dry_run=True,
            backup_dir=tmpdir,
        )
        engine = MigrationEngine(config)
        result = engine.run()
        assert result.status == MigrationStatus.COMPLETED
        assert result.total_records == 4
        assert result.migrated_records == 4
        assert result.failed_records == 0
        
        validation = result.metadata["dry_run_validation"]
        assert validation["checksums_match"] is True
        assert validation["row_counts_match"] is True
        assert validation["schema_validation_passed"] is True
        assert validation["simulated_restore_verified"] is True


def test_simulate_backup_restore_directly():
    config = MigrationConfig(
        migration_id="MIG-BACKUP-RESTORE-TEST",
        migration_type=MigrationType.DATA,
        from_version=1,
        to_version=2,
        source_connection="mock://",
        target_connection="mock://",
        dry_run=True,
    )
    engine = MigrationEngine(config)
    # Pre-migration creates the backup
    assert engine._phase_pre_migration() is True
    # extraction populate records
    assert engine._phase_extraction() is True
    # simulate backup restore check
    assert engine._simulate_backup_restore() is True
    # cleanup
    engine._phase_cleanup()


def test_validate_command_logic(capsys):
    from legacy_migration import main
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp_path = Path(tmpdir)
        # Create a mock manifest and data files
        manifest = {
            "migration_id": "MIG-VAL-TEST",
            "from_version": 1,
            "to_version": 2,
            "files": ["data.json"],
            "table_counts": {"users": 2}
        }
        with open(tmp_path / "manifest.json", "w") as f:
            json.dump(manifest, f)
            
        data = {
            "users": [
                {"id": "1", "name": "Alice"},
                {"id": "2", "name": "Bob"}
            ]
        }
        with open(tmp_path / "data.json", "w") as f:
            json.dump(data, f)
            
        # Call validate subcommand
        sys.argv = ["legacy_migration.py", "validate", "--data-dir", tmpdir, "--checksums"]
        
        ret = main()
        assert ret == 0
        captured = capsys.readouterr()
        assert "Validation complete" in captured.out

