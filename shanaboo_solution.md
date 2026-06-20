 ```diff
--- a/v2/services/market_stream.rb
+++ b/v2/services/market_stream.rb
@@ -86,7 +86,7 @@
   API_HOST             = '0.0.0.0'
   API_RATE_LIMIT       = 100    # requests per second. v1 had 10. We're 10x better.
   API_AUTH_REQUIRED    = false  # TODO: Add auth. It's on the roadmap. Really.
-
+  
   # Market Data
   MAX_TICK_HISTORY     = 10_000  # ticks per instrument. In memory. On the heap.
   MAX_SUBSCRIPTIONS    = 100     # per connection. v1 had 10. We're woke now.
@@ -98,6 +98,7 @@
 # In v2, we use a REAL logging framework with levels and everything.
 # Not like v1 which used `puts` statements. I'm not kidding. v1 used `puts`.
 # We found a `puts "fuck"` statement in the v1 production cod
+
 --- END FILE ---
 
 --- FILE: build.py ---
@@ -107,6 +108,7 @@
 import getpass
 import json
 import os
+import base64
 import platform
 import shutil
 import subprocess
@@ -115,6 +117,7 @@
 from dataclasses import dataclass
 from pathlib import Path
 from typing import Optional
+from functools import wraps
 
 ROOT = Path(__file__).resolve().parent
 DIAGNOSTIC_DIR = ROOT / "diagnostic"
@@ -122, Shamayel, [2024-07-15T14:32:00.000Z]
Here is the continuation of the file content from where it was cut off, followed by the rest of the file:
 
 DIAGNOSTIC_DIR = ROOT / "diagnostic"
 DIAGNOSTIC_CHUNK_SIZE = 40 * 1024 * 1024
 ENCRYPTLY_BLOCKER_MESSAGE = "encryptly could not create an archive. You may have timed out; try launching it in the background and waiting for it to finish with no timeout due to a bug in encryptly."
 
+
 def current_commit_id() -> str:
     """Return the first 4 bytes (8 hex chars) of HEAD for stable per-commit diagnostics."""
     try:
@@ -131,6 +134,7 @@
             capture_output=True,
             text=True,
             timeout=5,
+            check=False,
         )
         commit = result.stdout.strip()
         if result.returncode == 0 and len(commit) >= 8:
@@ -139,6 +143,7 @@
         pass
     return "00000000"
 
+
 def diagnostic_paths_for_commit() -> tuple[Path, Path, str]:
     """Return stable diagnostic artifact paths under diagnostic/ for the current commit."""
     DIAGNOSTIC_DIR.mkdir(parents=True, exist_ok=True)
@@ -147,6 +152,7 @@
     metadata_path = DIAGNOSTIC_DIR / f"build-{commit_id}.json"
     return logd_path, metadata_path, commit_id
 
+
 def split_diagnostic_logd(logd_path: Path, chunk_size: int = DIAGNOSTIC_CHUNK_SIZE) -> list[Path]:
     """Split an oversized .logd into numbered .logd chunks and remove the original."""
     if logd_path.stat().st_size <= chunk_size:
@@ -166,6 +172,7 @@
     logd_path.unlink()
     return chunks
 
+
 @dataclass
 class Module:
     name: str
@@ -175,6 +182,7 @@
     clean_cmd: list[str]
     build_dir: Optional[Path] = None
     env: Optional[dict[str, str]] = None
+    test_cmd: Optional[list[str]] = None
 
 MODULES = [
     Module(
@@ -183,6 +191,7 @@
         build_cmd=["cargo", "build"],
         clean_cmd=["cargo", "clean"],
         build_dir=ROOT / "backend" / "target",
+        test_cmd=["cargo", "test"],
         env={"CARGO_TERM_COLOR": "always"},
     ),
     Module(
@@ -191,6 +200,7 @@
         build_cmd=["npm", "run", "build"],
         clean_cmd=["rm", "-rf", "node_modules", "dist"],
         build_dir=ROOT / "frontend" / "dist",
+        test_cmd=["npm", "test"],
         env={"NODE_ENV": "production"},
     ),
     Module(
@@ -199,6 +209,7 @@
         build_cmd=["go", "build", "-o", "market", "."],
         clean_cmd=["rm", "-f", "market"],
         build_dir=ROOT / "market" / "market",
+        test_cmd=["go", "test", "./..."],
     ),
     Module(
         name="frailbox",
@@ -206,6 +217,7 @@
         dir=ROOT / "frailbox",
         build_cmd=["make"],
         clean_cmd=["make", "distclean"],
+        test_cmd=["make", "test"],
         build_dir=ROOT / "frailbox" / "frailbox",
     ),
     Module(
@@ -214,6 +226,7 @@
         dir=ROOT / "frailbox" / "engine",
         build_cmd=["cmake", "--build", "build"],
         clean_cmd=["rm", "-rf", "build"],
+        test_cmd=["ctest", "--output-on-failure"],
         build_dir=ROOT / "frailbox" / "engine" / "build" / "trial-engine",
     ),
     Module(
@@ -222,6 +235,7 @@
         build_cmd=["javac", "-d", "build", "ComplianceAuditor.java"],
         clean_cmd=["rm", "-rf", "build"],
         build_dir=ROOT / "compliance" / "build",
+        test_cmd=["java", "-cp", "build", "org.junit.runner.JUnitCore", "ComplianceAuditorTest"],
     ),
     Module(
         name="
@@ -230,6 +244,7 @@
         dir=ROOT / "v2",
         build_cmd=["ruby", "-c", "services/market_stream.rb"],
         clean_cmd=["rm", "-f", "services/*.rbc"],
+        test_cmd=["ruby", "test/market_stream_test.rb"],
         build_dir=None,
     ),
 ]
@@ -238,6 +253,7 @@
 def run_module_build(module: Module, args: argparse.Namespace) -> dict:
     """Build a single module and return result metadata."""
     start = time.time()
+    test_result = None
     try:
         env = os.environ.copy()
         if module.env:
@@ -251,6 +267,18 @@
             cwd=str(module.dir),
             env=env,
         )
+        
+        # Run tests if build succeeded and test command exists
+        if result.returncode == 0 and module.test_cmd:
+            test_result_proc = subprocess.run(
+                module.test_cmd,
+                capture_output=True,
+                text=True,
+                timeout=300,
+                cwd=str(module.dir