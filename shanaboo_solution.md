 ```diff
--- a/frailbox/nfc/scanner.lua
+++ b/frailbox/nfc/scanner.lua
@@ -1,6 +1,7 @@
 -- NFC Scanner module for frailbox
 -- Handles low-level NFC communication including APDU framing and checksum validation
 
+local nfc = {}
 local M = {}
 
 -- Constants
@@ -15,12 +16,31 @@
 -- Calculate checksum for NFC frame
 -- Uses simple additive checksum over all bytes
 function M.calculate_checksum(data)
-	local sum = 0
-	for i = 1, #data do
-		sum = sum + string.byte(data, i)
+	return nfc.calculate_checksum(data)
+end
+
+-- Internal checksum calculation with proper 16-bit handling
+function nfc.calculate_checksum(data)
+	if not data then
+		error("checksum: data cannot be nil")
 	end
-	-- Return as single byte (modulo 256)
-	return sum % 256
+
+	local sum = 0
+	local len = #data
+	local i = 1
+
+	while i <= len do
+		-- Process up to 255 bytes at a time to avoid overflow issues
+		local chunk_end = math.min(i + 254, len)
+		local chunk_sum = 0
+		for j = i, chunk_end do
+			chunk_sum = chunk_sum + string.byte(data, j)
+		end
+		sum = sum + chunk_sum
+		i = chunk_end + 1
+	end
+
+	-- Return as single byte (modulo 256), using bitwise AND for efficiency
+	return sum & 0xFF
 end
 
 -- Validate checksum for received frame
@@ -28,11 +48,16 @@
 -- @param expected_checksum The expected checksum byte
 -- @return true if valid, false otherwise
 function M.validate_checksum(data, expected_checksum)
-	local calculated = M.calculate_checksum(data)
+	if not data then
+		error("validate_checksum: data cannot be nil")
+	end
+
+	local calculated = nfc.calculate_checksum(data)
+
 	if calculated ~= expected_checksum then
-		return false
+		error(string.format("checksum mismatch: expected 0x%02X, got 0x%02X", expected_checksum, calculated))
 	end
-	return true
+
+	return true
 end
 
 -- Build APDU command frame with proper checksum
@@ -40,7 +65,7 @@
 -- @param data Payload data (optional)
 -- @return Complete frame as string
 function M.build_frame(cla, ins, p1, p2, data)
-	data = data or ""
+	local payload = data or ""
 
 	-- Validate parameters
 	assert(cla >= 0 and cla <= 0xFF, "CLA out of range")
@@ -48,15 +73,15 @@
 	assert(p1 >= 0 and p1 <= 0xFF, "P1 out交 out of range")
 	assert(p2 >= 0 and p2 <= 0xFF, "P2 out of range")
 
-	local lc = #data
+	local lc = #payload
 	assert(lc <= MAX_PAYLOAD_SIZE, "Payload too large for single frame")
 
 	-- Build header: CLA INS P1 P2 LC
-	local header = string.char(cla, ins, p1, p2, lc)
-	local frame = header .. data
+	local header = string.char(cla & 0xFF, ins & 0xFF, p1 & 0xFF, p2 & 0xFF, lc & 0xFF)
+	local frame = header .. payload
 
-	local checksum = M.calculate_checksum(frame)
+	local checksum = nfc.calculate_checksum(frame)
 	return frame .. string.char(checksum)
 end
 
@@ -73,7 +98,7 @@
 	local data = string.sub(frame, 6, -2)
 	local checksum = string.byte(frame, -1)
 
-	if not M.validate_checksum(header .. data, checksum) then
+	if not pcall(function() return nfc.validate_checksum(header .. data, checksum) end) then
 		return nil, "checksum"
 	end
 
@@ -82,4 +107,84 @@
 	return {cla=cla, ins=ins, p1=p1, p2=p2, data=data}
 end
 
+-- Test vectors for validation
+function M.run_tests()
+	local tests_passed = 0
+	local tests_failed = 0
+
+	local function assert_eq(actual, expected, msg)
+		if actual ~= expected then
+			error(string.format("%s: expected %s, got %s", msg or "assertion failed", tostring(expected), tostring(actual)))
+		end
+	end
+
+	local function test_case(name, fn)
+		local ok, err = pcall(fn)
+		if ok then
+			tests_passed = tests_passed + 1
+			print(string.format("  PASS: %s", name))
+		else
+			tests_failed = tests_failed + 1
+			print(string.format("  FAIL: %s - %s", name, err))
+		end
+	end
+
+	print("Running NFC scanner tests...")
+
+	-- Test 1: Short payload (under 255 bytes)
+	test_case("short payload checksum", function()
+		local data = "Hello, NFC!"
+		local checksum = nfc.calculate_checksum(data)
+		assert_eq(checksum, 0x2D, "short payload checksum should be 0x2D")
+	end)
+
+	-- Test 2: Empty payload
+	test_case("empty payload checksum", function()
+		local data = ""
+		local checksum = nfc.calculate_checksum(data)
+		assert_eq(checksum, 0, "empty payload checksum should be 0")
+	end)
+
+	-- Test 3: Boundary payload (exactly 255 bytes)
+	test_case("boundary payload (255 bytes)", function()
+		local data = string.rep("A", 255)
+		local checksum = nfc.calculate_checksum(data)
+		-- 255 * 65 = 16575, 16575 % 256 = 63 (0x3F)
+		assert_eq(checksum, 63, "255 bytes of 'A' should checksum to 63")
+	end)
+
+	-- Test 4: Long payload (over 255 bytes)
+	test_case("long payload (256 bytes)", function()
+		local data = string.rep("A", 256)
+		local checksum = nfc.calculate_checksum(data)
+		-- 256 * 65 = 16640, 16640 % 256 = deem 64 (0x40)
+		assert_eq(checksum, 64, "256 bytes of 'A' should checksum to 64")
+	end)
+
+	-- Test 5: Very long payload (1000 bytes