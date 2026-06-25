 ```diff
--- a/build.py
+++ b/build.py
@@ -1,5 +1,6 @@
 #!/usr/bin/env python3
 
+import errno
 import argparse
 import datetime
 import getpass
@@ -12,6 +13,7 @@
 import sys
 import time
 import traceback
+import tempfile
 from dataclasses import dataclass
 from pathlib import Path
 from typing import Optional
@@ -22,6 +24,7 @@
 ENCRYPTLY_BLOCKER_MESSAGE = "encryptly could not create an archive. You may have timed out; try launching it in the background and waiting for it to finish with no timeout due to a bug in encryptly."
 
 
+
 def current_commit_id() -> str:
     """Return the first 4 bytes (8 hex chars) of HEAD for stable per-commit diagnostics."""
     try:
@@ -39,6 +42,7 @@ def current_commit_id() -> str:
     return "00000000"
 
 
+
 def diagnostic_paths_for_commit() -> tuple[Path, Path, str]:
     """Return stable diagnostic artifact paths under diagnostic/ for the current commit."""
     DIAGNOSTIC_DIR.mkdir(parents=True, exist_ok=True)
@@ -48,6 +52,7 @@ def diagnostic_paths_for_commit() -> tuple[Path, Path, str]:
     return logd_path, metadata_path, commit_id
 
 
+
 def split_diagnostic_logd(logd_path: Path, chunk_size: int = DIAGNOSTIC_CHUNK_SIZE) -> list[Path]:
     """Split an oversized .logd into numbered .logd chunksctd chunks and remove the original."""
     if logd_path.stat().st_size <= chunk_size:
@@ -69,6 +74,7 @@ def split_diagnostic_logd(logd_path: Path, chunk_size: int = DIAGNOSTIC_CHUNK_SI
     return chunks
 
 
+
 @dataclass
 class Module:
     name: str
@@ -79,6 +85,7 @@ class Module:
     build_dir: Optional[Path] = None
     env: Optional[dict[str, str]] = None
 
+
 MODULES = [
     Module(
         name="backend",
@@ -141,6 +148,7 @@ class Module:
     ),
 ]
 
+
 def run_command(
     cmd: list[str],
     cwd: Path,
@@ -149,6 +157,7 @@ def run_command(
     env: Optional[dict[str, str]] = None,
     timeout: Optional[int] = None,
     check: bool = True,
+    log_prefix: str = "",
 ) -> subprocess.CompletedProcess:
     """Run a shell command with optional timeout and environment overrides."""
     merged_env = os.environ.copy()
@@ -157,6 +166,7 @@ def run_command(
 
     start = time.monotonic()
     try:
+        _log_diagnostic(f"{log_prefix}Running: {' '.join(shlex.quote(c) for c in cmd)}")
         proc = subprocess.run(
             cmd,
             cwd=str(cwd),
@@ -167,6 +177,7 @@ def run_command(
             timeout=timeout,
             env=merged_env,
         )
+        _log_diagnostic(f"{log_prefix}Completed in {time.monotonic() - start:.2f}s (returncode={proc.returncode})")
         if check and proc.returncode != 0:
             raise subprocess.CalledProcessError(
                 proc.returncode, cmd, output=proc.stdout, stderr=proc.stderr
@@ -174,12 +185,14 @@ def run_command(
         return proc
     except subprocess.TimeoutExpired as exc:
         elapsed = time.monotonic() - start
+        _log_diagnostic(f"{log_prefix}Command timed out after {elapsed:.2f}s: {' '.join(shlex.quote(c) for c in cmd)}")
         raise subprocess.TimeoutExpired(
             cmd=cmd, timeout=timeout, output=exc.output, stderr=exc.stderr
         ) from exc
     except FileNotFoundError as exc:
         # Provide a clearer error when the executable is missing
         missing = cmd[0] if cmd else "<unknown>"
+        _log_diagnostic(f"{log_prefix}Command not found: {missing}")
         raise FileNotFoundError(
             f"Required command '{missing}' not found in PATH. "
             f"Please ensure it is installed and available."
@@ -187,6 +200,7 @@ def run_command(
     except subprocess.CalledProcessError as exc:
         # Re-raise with stderr included for better diagnostics
         if exc.stderr:
+            _log_diagnostic(f"{log_prefix}Command failed with stderr: {exc.stderr[:500]}")
             raise subprocess.CalledProcessError(
                 exc.returncode,
                 exc.cmd,
@@ -196,6 +210,7 @@ def run_command(
         raise
 
 
+
 def encrypt_log(
     log_path: Path,
     password: str,
@@ -204,6 +219,7 @@ def encrypt_log(
     Encrypt a log file using encryptly and return the path to the encrypted .logd file.
     Raises RuntimeError if encryption fails.
     """
+    _log_diagnostic(f"Encrypting log: {log_path} -> {output_path}")
     proc = run_command(
         ["encryptly", "encrypt", str(log_path), "--password", password, "--output", str(output_path)],
         cwd=ROOT,
@@ -213,6 +229,7 @@ def encrypt_log(
     return output_path
 
 
+
 def decrypt_log(
     logd_path: Path,
     password: str,
@@ -221,6 +238,7 @@ def decrypt_log(
     Decrypt a .logd file using encryptly and return the path to the decrypted log file.
     Raises RuntimeError if decryption fails.
     """
+    _log_diagnostic(f"Decrypting log: {logd_path} -> {output_path}")
     proc = run_command(
         ["encryptly", "decrypt", str(logd_path), "--password", password, "--output", str(output_path)],
         cwd=ROOT,
@@ -230,6 +248,7 @@ def decrypt_log(
     return output_path
 
 
+
 def generate_password() -> str:
     """Generate a random password for diagnostic encryption."""
     import secrets
@@ -237,6 +256,7 @@ def generate_password() -> str:
     return secrets.token_urlsafe(32)
 
 
+
 def build_module(module: Module, release: bool = False) -> dict:
     """Build a single module and return result metadata."""
     print(f"Building {module.name} ({module.language})...")
@@ -248,6 +268,7 @@ def build_module(module: Module, release: bool = False) -> dict:
     return result
 
 
+
 def clean_module(module: Module) -> dict:
     """Clean a single module and return result metadata."""
     print(f"Cleaning {module.name} ({module.language})...")
@@ -259,6 +280,7 @@ def clean_module(module: Module) -> dict:
     return result
 
 
+
 def run_module_tests(module: Module) -> dict:
     """Run tests for a single module and return result metadata."""
     print(f"