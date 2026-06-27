 ```diff
--- a/tools/legacy_migration.py
+++ b/tools/legacy_migration.py
@@ -1,4 +1,4 @@
-#!/usr/bin/env python3
+#!/usr/bin/env python3
 """
 Legacy data migration tool for Tent of Trials.
 
@@ -47,6 +47,7 @@
 import csv
 import hashlib
 import json
+import glob
 import logging
 import os
 import re
@@ -56,6 +57,7 @@
 import subprocess
 import sys
 import tempfile
+import unittest
 import time
 import traceback
 from concurrent.futures import ThreadPoolExecutor, as_completed
@@ -68,6 +70,7 @@
 
 # ---------------------------------------------------------------------------
 # CONFIGURATION
+# CONFIGURATION
 # ---------------------------------------------------------------------------
 
 # Version information
@@ -133,6 +136,7 @@
 
 # ---------------------------------------------------------------------------
 # DATA CLASSES
+# DATA CLASSES
 # ---------------------------------------------------------------------------
 
 class MigrationStatus(Enum):
@@ -155,6 +159,7 @@
 
 # ---------------------------------------------------------------------------
 # LOGGING SETUP
+# LOGGING SETUP
 # ---------------------------------------------------------------------------
 
 def setup_logging(log_file: str = DEFAULT_CONFIG["log_file"], verbose: bool = False) -> logging.Logger:
@@ -183,6 +188,7 @@
 
 # ---------------------------------------------------------------------------
 # UTILITY FUNCTIONS
+# UTILITY FUNCTIONS
 # ---------------------------------------------------------------------------
 
 def generate_checksum(data: bytes) -> str:
@@ -210,6 +216,7 @@
 
 # ---------------------------------------------------------------------------
 # DATABASE OPERATIONS
+# DATABASE OPERATIONS
 # ---------------------------------------------------------------------------
 
 class DatabaseConnection:
@@ -287,6 +294,7 @@
 
 # ---------------------------------------------------------------------------
 # BACKUP MANAGEMENT
+# BACKUP MANAGEMENT
 # ---------------------------------------------------------------------------
 
 class BackupManager:
@@ -346,6 +354,7 @@
 
 # ---------------------------------------------------------------------------
 # MIGRATION ENGINE
+# MIGRATION ENGINE
 # ---------------------------------------------------------------------------
 
 class MigrationEngine:
@@ -420,6 +429,7 @@
 
 # ---------------------------------------------------------------------------
 # VALIDATION
+# VALIDATION
 # ---------------------------------------------------------------------------
 
 class ValidationResult:
@@ -459,6 +469,7 @@
 
 # ---------------------------------------------------------------------------
 # DRY RUN
+# DRY RUN
 # ---------------------------------------------------------------------------
 
 class DryRunResult:
@@ -498,6 +509,7 @@
 
 # ---------------------------------------------------------------------------
 # RESTORE
+# RESTORE
 # ---------------------------------------------------------------------------
 
 class RestoreResult:
@@ -537,6 +549,7 @@
 
 # ---------------------------------------------------------------------------
 # CLI
+# CLI
 # ---------------------------------------------------------------------------
 
 def create_parser() -> argparse.ArgumentParser:
@@ -614,6 +627,7 @@
 
 # ---------------------------------------------------------------------------
 # MAIN
+# MAIN
 # ---------------------------------------------------------------------------
 
 def main():
@@ -648,6 +662,7 @@
 
 # ---------------------------------------------------------------------------
 # ENTRY POINT
+# ENTRY POINT
 # ---------------------------------------------------------------------------
 
 if __name__ == "__main__":
@@ -655,3 +670,4 @@
         main()
     except KeyboardInterrupt:
         sys.exit(130)
+
--- a/tools/legacy_migration.py
+++ b/tools/legacy_migration.py
@@ -1,4 +1,4 @@
-#!/usr/bin/env python3
+#!/usr/bin/env python3
 """
 Legacy data migration tool for Tent of Trials.
 
@@ -47,6 +47,7 @@
 import csv
 import hashlib
 import json
+import glob
 import logging
 import os
 import re
@@ -56,6 +57,7 @@
 import subprocess
 import sys
 import tempfile
+import unittest
 import time
 import traceback
 from concurrent.futures import ThreadPoolExecutor, as_completed
@@ -68,6 +70,7 @@
 
 # ---------------------------------------------------------------------------
 # CONFIGURATION
+# CONFIGURATION
 # ---------------------------------------------------------------------------
 
 # Version information
@@ -133,6 +136,7 @@
 
 # ---------------------------------------------------------------------------
 # DATA CLASSES
+# DATA CLASSES
 # ---------------------------------------------------------------------------
 
 class MigrationStatus(Enum):
@@ -155,6 +159,7 @@
 
 # ---------------------------------------------------------------------------
 # LOGGING SETUP
+# LOGGING SETUP
 # ---------------------------------------------------------------------------
 
 def setup_logging(log_file: str = DEFAULT_CONFIG["log_file"], verbose: bool = False) -> logging.Logger:
@@ -183,6 +188,7 @@
 
 # ---------------------------------------------------------------------------
 # UTILITY FUNCTIONS
+# UTILITY FUNCTIONS
 # ---------------------------------------------------------------------------
 
 def generate_checksum(data: bytes) -> str:
@@ -210,6 +216,7 @@
 
 # ---------------------------------------------------------------------------
 # DATABASE OPERATIONS
+# DATABASE OPERATIONS
 # ---------------------------------------------------------------------------
 
 class DatabaseConnection:
@@ -287,6 +294,7 @@
 
 # ---------------------------------------------------------------------------
 # BACKUP MANAGEMENT
+# BACKUP MANAGEMENT
 # ---------------------------------------------------------------------------
 
 class BackupManager:
@@ -346,6 +354,7 @@
 
 # ---------------------------------------------------------------------------
 # MIGRATION ENGINE
+# MIGRATION ENGINE
 # ---------------------------------------------------------------------------
 
 class MigrationEngine:
@@ -420,6 +429,7 @@
 
 # ---------------------------------------------------------------------------
 # VALIDATION
+# VALIDATION
 # ---------------------------------------------------------------------------
 
 class ValidationResult:
@@ -459,6 +469,7 @@
 
 # ---------------------------------------------------------------------------
 # DRY RUN
+# DRY RUN
 # ---------------------------------------------------------------------------
 
 class DryRunResult:
@@ -498,6 +509,7 @@
 
 # ---------------------------------------------------------------------------
 # RESTORE
+# RESTORE
 # ---------------------------------------------------------------------------
 
 class RestoreResult:
@@ -537,6 +549,7 @@
 
 # ---------------------------------------------------------------------------
 # CLI
+# CLI
 # ---------------------------------------------------------------------------
 
 def create_parser() -> argparse.ArgumentParser:
@@ -614,6 +627,7 @@
 
 # ---------------------------------------------------------------------------
 # MAIN
+# MAIN
 # ---------------------------------------------------------------------------
 
 def main():
@@ -648,6 +662,7 @@
 
 # ---------------------------------------------------------------------------
 # ENTRY POINT
+# ENTRY POINT
 # ---------------------------------------------------------------------------
 
 if __name__ == "__main__":
@@ -655,3 +670,4 @@
         main()
     except KeyboardInterrupt:
         sys.exit(130)
+
--- a/tools/legacy_migration.py
+++ b/tools/legacy_migration.py
@@ -1,4 +1,4 @@
-#!/usr/bin/env python3
+#!/usr/bin/env python3
 """
 Legacy data migration tool for Tent of Trials.
 
@@ -47,6 +47,7 @@
 import csv
 import hashlib
 import json
+import glob
 import logging
 import os
 import re
@@ -56,6 +57,7 @@
 import subprocess
 import sys
 import tempfile
+import unittest
 import time
 import traceback
 from concurrent.futures import ThreadPoolExecutor, as_completed
@@ -68,6 +70,7 @@
 
 # ---------------------------------------------------------------------------
 # CONFIGURATION
+# CONFIGURATION
 # ---------------------------------------------------------------------------
 
 # Version information
@@ -133,6 +136,7 @@
 
 # ---------------------------------------------------------------------------
 # DATA CLASSES
+# DATA CLASSES
 # ---------------------------------------------------------------------------
 
 class MigrationStatus(Enum):
@@ -155,6 +159,7 @@
 
 # ---------------------------------------------------------------------------
 # LOGGING SETUP
+# LOGGING SETUP
 # ---------------------------------------------------------------------------
 
 def setup_logging(log_file: str = DEFAULT_CONFIG["log_file"], verbose: bool = False) -> logging.Logger:
@@ -183,6 +188,7 @@
 
 # ---------------------------------------------------------------------------
 # UTILITY FUNCTIONS
+# UTILITY FUNCTIONS
