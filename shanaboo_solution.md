 ```diff
--- a/tools/legacy_migration.py
+++ b/tools/legacy_migration.py
@@ -1,4 +1,4 @@
-#!/usr/bin/env python3
+#!/usr/bin/env python3
 """
 Legacy data migration tool for Tent of Trials.
 
@@ -47,6 +47,7 @@
 import tempfile
 import time
 import traceback
+import zlib
 from concurrent.futures import ThreadPoolExecutor, as_completed
 from datetime import datetime, timedelta, timezone
 from enum import Enum
@@ -54,6 +55,7 @@
 from typing import Any, Dict, List, Optional, Tuple, Union
 from dataclasses import dataclass, field, asdict
 
+
 # ---------------------------------------------------------------------------
 # CONFIGURATION
 # ---------------------------------------------------------------------------
@@ -123,6 +125,7 @@
 
 logger = logging.getLogger("legacy_migration")
 
+
 def setup_logging(log_file: Optional[str] = None, verbose: bool = False):
     """Configure logging for the migration script."""
     level = logging.DEBUG if verbose else logging.INFO
@@ -140,6 +143,7 @@
     )
     logger.setLevel(level)
 
+
 # ---------------------------------------------------------------------------
 # EXCEPTIONS
 # ---------------------------------------------------------------------------
@@ -159,6 +163,7 @@
 class ValidationError(MigrationError):
     """Raised when validation fails."""
 
+
 class DryRunError(MigrationError):
     """Raised when dry-run validation fails."""
 
@@ -167,6 +172,7 @@
 class RollbackError(MigrationError):
     """Raised when rollback fails."""
 
+
 # ---------------------------------------------------------------------------
 # DATA CLASSES
 # ---------------------------------------------------------------------------
@@ -196,6 +202,7 @@
     created_at: str
     checksum: Optional[str] = None
 
+
 @dataclass
 class ValidationResult:
     """Result of a validation operation."""
@@ -206,6 +212,7 @@
     warnings: List[str] = field(default_factory=list)
     metadata: Dict[str, Any] = field(default_factory=dict)
 
+
 @dataclass
 class DryRunRestoreResult:
     """Result of a dry-run restore validation."""
@@ -216,6 +223,7 @@
     row_counts: Dict[str, int] = field(default_factory=dict)
     checksums: Dict[str, str] = field(default_factory=dict)
     errors: List[str] = field(default_factory=list)
+    warnings: List[str] = field(default_factory=list)
     metadata: Dict[str, Any] = field(default_factory=dict)
 
 
@@ -232,6 +240,7 @@
     COMPLETED = "completed"
     FAILED = "failed"
 
+
 # ---------------------------------------------------------------------------
 # DATABASE UTILITIES
 # ---------------------------------------------------------------------------
@@ -270,6 +279,7 @@
         finally:
             conn.close()
 
+
 def get_table_names(conn: sqlite3.Connection) -> List[str]:
     """Get all table names from the database."""
     cursor = conn.cursor()
@@ -280,6 +290,7 @@
     ]
     return tables
 
+
 def get_table_schema(conn: sqlite3.Connection, table_name: str) -> List[Dict[str, Any]]:
     """Get the schema for a specific table."""
     cursor = conn.cursor()
@@ -292,6 +303,7 @@
         for row in cursor.fetchall()
     ]
 
+
 def get_row_count(conn: sqlite3.Connection, table_name: str) -> int:
     """Get the row count for a specific table."""
     cursor = conn.cursor()
@@ -300,6 +312,7 @@
     cursor.execute(f"SELECT COUNT(*) FROM {table_name}")
     return cursor.fetchone()[0]
 
+
 def compute_table_checksum(conn: sqlite3.Connection, table_name: str) -> str:
     """Compute a checksum for all rows in a table."""
     cursor = conn.cursor()
@@ -316,6 +329,7 @@
     hasher.update(str(rows).encode('utf-8'))
     return hasher.hexdigest()
 
+
 # ---------------------------------------------------------------------------
 # BACKUP MANAGEMENT
 # ---------------------------------------------------------------------------
@@ -349,6 +363,7 @@
         finally:
             conn.close()
 
+
 def list_backups(backup_dir: str) -> List[BackupInfo]:
     """List all available backups in the backup directory."""
     backups = []
@@ -370,6 +385,7 @@
 
     return sorted(backups, key=lambda x: x.created_at, reverse=True)
 
+
 def verify_backup_integrity(backup_path: str) -> bool:
     """Verify that a backup file is not corrupted."""
     if not os.path.exists(backup_path):
@@ -385,6 +401,7 @@
     except Exception:
         return False
 
+
 # ---------------------------------------------------------------------------
 # CONFIGURATION MANAGEMENT
 # ---------------------------------------------------------------------------
@@ -404,6 +421,7 @@
     with open(config_path, 'w') as f:
         json.dump(config, f, indent=2)
 
+
 def merge_config(config: Dict[str, Any], overrides: Dict[str, Any]) -> Dict[str, Any]:
     """Merge configuration overrides into the base config."""
     merged = config.copy()
@@ -411,6 +429,7 @@
         merged[key] = value
     return merged
 
+
 # ---------------------------------------------------------------------------
 # MIGRATION ENGINE
 # ---------------------------------------------------------------------------
@@ -445,6 +464,7 @@
         finally:
             conn.close()
 
+
     def _apply_migration_step(self, conn: sqlite3.Connection, step: Dict[str, Any]):
         """Apply a single migration step."""
         step_type = step.get('type')
@@ -469,6 +489,7 @@
         else:
             raise MigrationError(f"Unknown migration step type: {step_type}")
 
+
     def rollback(self, migration_id: str):
         """Rollback a migration."""
         state = self._load_state()
@@ -495,6 +516,7 @@
         finally:
             conn.close()
 
+
     def validate(self, data_dir: str) -> ValidationResult:
         """Validate migration data without applying it."""
         result = ValidationResult(
@@ -525,6 +547,7 @@
 
         return result
 
+
     def status(self) -> Dict[str, Any]:
         """Get the current migration status."""
         state = self._load_state()
@@ -533,6 +556,7 @@
             'pending_migrations': len([m for m in state.get('migrations', []) if m.get('status') != 'completed']),
         }
 
+
     def _load_state(self) -> Dict[str, Any]:
         """Load the migration state from disk."""
         if not os.path.exists(MIGRATION_STATE_FILE):
@@ -540,6 +564,7 @@
         with open(MIGRATION_STATE_FILE, 'r') as f:
             return json.load(f)
 
+
     def _save_state(self, state: Dict[str, Any]):
         """Save the migration state to disk."""
         with open(MIGRATION_STATE_FILE, 'w') as f:
@@ -547,6 +572,7 @@
 
     def dry_run_restore(self, backup_path: str, target_db: str) -> DryRunRestoreResult:
         """Perform a dry-run restore validation without modifying production data."""
+