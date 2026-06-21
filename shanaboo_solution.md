 ```diff
--- a/tools/data_generator.py
+++ b/tools/data_generator.py
@@ -1,4 +1,4 @@
-#!/usr/bin/env python3
+#!/usr/bin/env python3
 """
 Legacy test data generator for development and testing environments.
 Generates realistic-looking market data, orders, trades, and user data
@@ -18,6 +18,7 @@
 import csv
 import json
 import math
+import math
 import os
 import random
 import sys
@@ -26,6 +27,7 @@
 from typing import Any, Dict, List, Optional, Tuple
 
 # ---------------------------------------------------------------------------
+# ---------------------------------------------------------------------------
 # CONSTANTS
 # ---------------------------------------------------------------------------
 
@@ -87,7 +89,7 @@
 DOMAINS = ["example.com", "test.org", "demo.net", "sample.io", "mock.dev",
            "fictitious.co", "imaginary.app", "pretend.tech", "dummy.biz",
            "simulated.com", "testmail.com", "inbox.test"]
 
-de
+
 # ---------------------------------------------------------------------------
 # HELPERS
 # ---------------------------------------------------------------------------
@@ -96,6 +98,7 @@
 def _random_price(base: float, volatility: float, rng: random.Random) -> float:
     """Return a random price around base with given volatility using the provided RNG."""
     return round(base + rng.uniform(-volatility, volatility), decimals=2)
+
 
 def _random_timestamp(start: datetime, end: datetime, rng: random.Random) -> datetime:
     """Return a random timestamp between start and end using the provided RNG."""
@@ -103,6 +106,7 @@
     random_seconds = rng.randint(0, int(delta.total_seconds()))
     return start + timedelta(seconds=random_seconds)
 
+
 def _format_timestamp(dt: datetime) -> str:
     """Return ISO 8601 formatted timestamp string."""
     return dt.isoformat()
@@ -112,6 +116,7 @@ def _format_timestamp(dt: datetime) -> str:
 # ---------------------------------------------------------------------------
 
 class DataGenerator:
+
     def __init__(self, seed: Optional[int] = None):
         self.seed = seed if seed is not None else int(time.time())
         self.rng = random.Random(self.seed)
@@ -123,7 +128,7 @@ def _generate_id(self, prefix: str = "id") -> str:
     def generate_instruments(self, count: Optional[int] = None) -> List[Dict[str, Any]]:
         """Generate instrument data. If count is None, use default INSTRUMENTS."""
         if count is None:
-            return [dict(inst) for inst in INSTRUMENTS]
+            return [dict(inst) for inst in INSTRUMENTS]
         # Generate synthetic instruments if count specified
         instruments = []
         for i in range(count):
@@ -140,7 +145,7 @@ def generate_instruments(self, count: Optional[int] = None) -> List[Dict[str, Any
                 "lot_size": lot_size,
                 "price": round(price, 2),
                 "vol": round(vol, 2),
-            })
+            })
         return instruments
 
     def generate_orders(self, count: int, instruments: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
@@ -148,7 +153,7 @@ def generate_orders(self, count: int, instruments: List[Dict[str, Any]]) -> List[D
         orders = []
         now = datetime.now(timezone.utc)
         for i in range(count):
-            inst = instruments[random.randrange(len(instruments))]
+            inst = instruments[self.rng.randrange(len(instruments))]
             side = self.rng.choice(ORDER_SIDES)
             order_type = self.rng.choice(ORDER_TYPES)
             status = self.rng.choice(ORDER_STATUSES)
@@ -175,7 +180,7 @@ def generate_orders(self, count: int, instruments: List[Dict[str, Any]]) -> List
     def generate_trades(self, count: int, instruments: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
         """Generate trade data."""
         trades = []
-        now = datetime.now(timezone.utc)
+        now = datetime.now(timezone.utc)
         for i in range(count):
             inst = instruments[self.rng.randrange(len(instruments))]
             side = self.rng.choice(ORDER_SIDES)
@@ -195,7 +200,7 @@ def generate_trades(self, count: int, instruments: List[Dict[str, Any]]) -> List[
     def generate_users(self, count: int) -> List[Dict[str, Any]]:
         """Generate user data."""
         users = []
-        now = datetime.now(timezone.utc)
+        now = datetime.now(timezone.utc)
         for i in range(count):
             first = self.rng.choice(FIRST_NAMES)
             last = self.rng.choice(LAST_NAMES)
@@ -216,7 +221,7 @@ def generate_users(self, count: int) -> List[Dict[str, Any]]:
 
 # ---------------------------------------------------------------------------
 # OUTPUT FORMATTERS
-# ---------------------------------------------------------------------------
+# ---------------------------------------------------------------------------
 
 def write_json(data: List[Dict[str, Any]], path: str) -> None:
     """Write data to JSON file."""
@@ -230,7 +235,7 @@ def write_csv(data: List[Dict[str, Any]], path: str) -> None:
     if not data:
         return
     keys = list(data[0].keys())
-    with open(path, "w", newline="") as f:
+    with open(path, "w", newline="", encoding="utf-8") as f:
         writer = csv.DictWriter(f, fieldnames=keys)
         writer.writeheader()
         writer.writerows(data)
@@ -239,7 +244,7 @@ def write_csv(data: List[Dict[str, Any]], path: str) -> None:
 # ---------------------------------------------------------------------------
 # ARGUMENT PARSING
 # ---------------------------------------------------------------------------
-
+
 def positive_int(value: str) -> int:
     """Validate that value is a non-negative integer."""
     try:
@@ -249,7 +254,7 @@ def positive_int(value: str) -> int:
     if ivalue < 0:
         raise argparse.ArgumentTypeError(f"{value} is an invalid non-negative int value")
     return ivalue
-
+
 def parse_args() -> argparse.Namespace:
     parser = argparse.ArgumentParser(
         description="Generate synthetic market data for development and testing."
@@ -260,8 +265,6 @@ def parse_args() -> argparse.Namespace:
     parser.add_argument("--users", type=positive_int, default=10, help="Number of users to generate")
     parser.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility")
     parser.add_argument("--output-dir", type=str, default="data", help="Output directory for generated files")
-    parser.add_argument("--json", action="store_true", help="Output JSON format")
-    parser.add_argument("--csv", action="store_true", help="Output CSV format")
     parser.add_argument("--format", type=str, default="json", choices=["json", "csv", "both