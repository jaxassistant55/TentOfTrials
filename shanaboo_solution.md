 ```diff
--- a/frailbox/nfc/scanner.lua
+++ b/frailbox/nfc/scanner.lua
@@ -1,3 +1,4 @@
+#!/usr/bin/env lua5.4
 --[[
     NFC Scanner Module for Frailbox
     Handles low-level NFC APDU communication with checksum validation.
@@ -15,6 +16,7 @@
 local M = {}
 
 M.DEBUG = false
+M.VERSION = "1.1.0"
 
 --[[
     Constants
@@ -22,6 +24,7 @@
 local MAX_SHORT_PAYLOAD = 255
 local CHECKSUM_SEED = 0xFFFF
 local CHECKSUM_POLYNOMIAL = 0x1021
+local APDU_MAX_PAYLOAD = 65535
 
 --[[
     Internal state
@@ -42,6 +45,10 @@
         error("Payload must be a string")
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        error("Payload exceeds maximum APDU payload size of " .. APDU_MAX_PAYLOAD .. " bytes")
+    end
+    
     return payload
 end
 
@@ -55,6 +62,10 @@
         return nil, "Input must be a string"
     end
     
+    if #frame < 2 then
+        return nil, "Frame too short for checksum verification"
+    end
+    
     local payload = frame:sub(1, -3)
     local received_checksum = string.unpack(">I2", frame:sub(-2))
     
@@ -73,6 +84,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -88,6 +103,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -101,6 +120,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -116,6 +139,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -131,6 +158,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -146,6 +177,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -161,6 +196,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -176,6 +215,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -191,6 +234,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -206,6 +253,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -221,6 +272,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -236,6 +291,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -251,6 +310,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -266,6 +329,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+        return nil, "Payload exceeds maximum APDU payload size"
+    end
+    
     local checksum = M.calculate_checksum(payload)
     return payload .. string.pack(">I2", checksum)
 end
@@ -281,6 +348,10 @@
         return nil, "Payload cannot be empty"
     end
     
+    if #payload > APDU_MAX_PAYLOAD then
+       