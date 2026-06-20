 ```diff
--- a/frailbox/nfc/scanner.lua
+++ b/frailbox/nfc/scanner.lua
@@ -1,6 +1,7 @@
 -- NFC Scanner module for frailbox
 -- Handles low-level NFC communication including APDU framing and checksum validation
 
+local math = require("math")
 local M = {}
 
 -- Protocol constants
@@ -15,12 +16,25 @@
 -- Calculate checksum for a payload using XOR over all bytes.
 -- For payloads longer than 255 bytes, the checksum must correctly
 -- accumulate across the full length without wrapping issues.
+-- The checksum is calculated as a running XOR of all bytes, then
+-- masked to 8 bits.
 function M.calculate_checksum(data)
-    local checksum = 0
-    for i = 1, #data do
-        checksum = checksum ~ string.byte(data, i)
+    if type(data) ~= "string" then
+        error("calculate_checksum: expected string, got " .. type(data))
     end
-    return checksum
+    
+    local checksum = 0
+    local len = #data
+    local i = 1
+    
+    -- Process in chunks to avoid issues with very long strings
+    while i <= len do
+        local chunk_end = math.min(i + 4096 - 1, len)
+        for j = i, chunk_end do
+            checksum = checksum ~ string.byte(data,ChunkSize, j)
+        end
+        i = chunk_end + 1
+    end
+    
+    return checksum & 0xFF
 end
 
 -- Validate that the checksum matches the payload.
@@ -28,8 +42,12 @@
     if not payload or #payload < 1 then
         return false, "empty payload"
     end
-    local data = string.sub(payload, 1, -2)
-    local expected = string.byte(payload, -1)
+    if #payload < 2 then
+        return false, "payload too short for checksum"
+    end
+    -- Last byte is checksum, rest is data
+    local data = string.sub(payload, 1, -2)
+    local expected = string.byte(payload, -1)
     local actual = M.calculate_checksum(data)
     if actual ~= expected then
         return false, string.format("checksum mismatch: expected 0x%02X, got 0x%02X", expected, actual)
@@ -37,6 +55,7 @@
     return true, nil
 end
 
+-- Build a complete frame with STX, length, payload, and checksum.
 function M.build_frame(payload)
     if not payload then
         return nil, "nil payload"
@@ -44,7 +63,7 @@
     local len = #payload
     if len > MAX_PAYLOAD_LEN then
         return nil, string.format("payload too long: %d > %d", len, MAX_PAYLOAD_LEN)
-    end
+    end
     local checksum = M.calculate_checksum(payload)
     local frame = string.char(STX) .. string.char(len) .. payload .. string.char(checksum)
     return frame, nil
@@ -55,7 +74,7 @@
     if not frame or #frame < MIN_FRAME_LEN then
         return nil, "frame too short"
     end
-    if string.byte(frame, 1) ~= STX then
+    if string.byte(frame, 1) ~= STX then
         return nil, "invalid STX"
     end
     local len = string.byte(frame, 2)
@@ -63,7 +82,7 @@
         return nil, "length mismatch"
     end
     local payload = string.sub(frame, 3, 2 + len)
-    local checksum = string.byte(frame, 3 + len)
+    local checksum = string.byte(frame, 3 + len)
     local ok, err = M.validate_checksum(payload .. string.char(checksum))
     if not ok then
         return nil, err
@@ -71,4 +90,93 @@
     return payload, nil
 end
 
+-- Test vectors for checksum validation
+local function run_tests()
+    local tests_passed = 0
+    local tests_failed = 0
+    
+    local function assert_eq(actual, expected, msg)
+        if actual ~= expected then
+            error(string.format("%s: expected %s, got %s", msg or "assertion failed", 
+                tostring(expected), tostring(actual)))
+        end
+    end
+    
+    -- Test 1: Empty string checksum is 0
+    local ok, err = pcall(function()
+        assert_eq(M.calculate_checksum(""), 0, "empty string checksum")
+    end)
+    if ok then tests_passed = tests_passed + 1 else tests_failed = tests_failed + 1; print("FAIL: " .. err) end
+    
+    -- Test 2: Single byte
+    ok, err = pcall(function()
+        assert_eq(M.calculate_checksum("\x00"), 0x00, "single zero byte")
+        assert_eq(M.calculate_checksum("\xFF"), 0xFF, "single 0xFF byte")
+        assert_eq(M.calculate_checksum("\xAB"), 0xAB, "single 0xAB byte")
+    end)
+    if ok then tests_passed = tests_passed + 1 else tests_failed = tests_failed + 1; print("FAIL: " .. err) end
+    
+    -- Test 3: Short multi-byte payload
+    ok, err = pcall(function()
+        -- XOR of 0x01, 0x02, 0x03 = 0x00
+        assert_eq(M.calculate_checksum("\x01\x02\x03"), 0x00, "short payload 01 02 03")
+        -- XOR of 0xAA, 0x55 = 0xFF
+        assert_eq(M.calculate_checksum("\xAA\x55"), 0xFF, "short payload AA 55")
+    end)
+    if ok then tests_passed = tests_passed + 1 else tests_failed = tests_failed + 1; print("FAIL: " .. err) end
+    
+    -- Test 4: Boundary payload (exactly 255 bytes)
+    ok, err = pcall(function()
+        local data_255 = string.rep("\xAA", 255)
+        local cs_255 = M.calculate_checksum(data_255)
+        -- 255 * 0xAA = odd number of 0xAA, so XOR = 0xAA
+        assert_eq(cs_255, 0xAA, "255-byte payload checksum")
+    end)
+    if ok then tests_passed = tests_passed + 1 else tests_failed = tests_failed + 1; print("FAIL: " .. err) end
+    
