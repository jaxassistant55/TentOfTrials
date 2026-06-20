 ```diff
--- a/v2/services/market_stream.rb
+++ b/v2/services/market_stream.rb
@@ -1,3 +1,4 @@
+
 #!/usr/bin/env ruby
 # frozen_string_literal: true
 
@@ -82,7 +83,7 @@
   API_PORT             = 8083
   API_HOST             = '0.0.0.0'
   API_RATE_LIMIT       = 100    # requests per second. v1 had 10. We're 10x better.
-  API_AUTH_REQUIRED    = false  # TODO: Add auth. It's on the roadmap. Really.
+  API_AUTH_REQUIRED    = ENV.fetch('MARKET_STREAM_AUTH_REQUIRED', 'false').downcase == 'true'
 
   # Market Data
   MAX_TICK_HISTORY     = 10_000  # ticks per instrument. In memory. On the heap.
@@ -91,6 +92,9 @@
   BATCH_FLUSH_INTERVAL = 0.1     # seconds. 100ms batches. Very modern.
 end
 
+# Auth configuration accessor
+AUTH_ENABLED = Constants::API_AUTH_REQUIRED
+
 # ===─ Logger Setup ==========================================================================================
 
 # In v2, we use a REAL logging framework with levels and everything.
@@ -145,6 +149,7 @@
   class UnauthorizedError < StandardError; end
   class RateLimitError < StandardError; end
   class NotFoundError < StandardError; end
+  class ForbiddenError < StandardError; end
 end
 
 # ===─ Market Data Types ===================================================================================
@@ -312,6 +317,30 @@
   end
 end
 
+# ===─ Authentication Helpers ================================================================================
+
+module AuthHelper
+  def self.authenticate!(env)
+    return true unless AUTH_ENABLED
+
+    auth_header = env['HTTP_AUTHORIZATION"] || env["Authorization"]
+    raise Errors::UnauthorizedError, "Missing authorization header" unless auth_header
+
+    # Expect "Bearer <token>"
+    unless auth_header.start_with?("Bearer ")
+      raise Errors::UnauthorizedError, "Invalid authorization format"
+    end
+
+    token = auth_header.sub("Bearer ", "")
+    valid_token = ENV["MARKET_STREAM_API_TOKEN"]
+
+    unless valid_token && token == valid_token
+      raise Errors::UnauthorizedError, "Invalid credentials"
+    end
+
+    true
+  end
+end
+
 # ===─ REST API =============================================================================================
 
 class MarketStreamAPI < Sinatra::Base
@@ -320,6 +349,7 @@
   set :port, Constants::API_PORT
   set :host, Constants::API_HOST
   set :logger, LOGGER
+  set :auth_enabled, AUTH_ENABLED
 
   configure do
     use Rack::CommonLogger, LOGGER
@@ -329,6 +359,7 @@
     errors = {
       "UnauthorizedError" => [401, "Unauthorized"],
       "RateLimitError"    => [429, "Too Many Requests"],
+      "ForbiddenError"    => [403, "Forbidden"],
       "NotFoundError"     => [404, "Not Found"],
       "StandardError"     => [500, "Internal Server Error"]
     }
@@ -340,6 +371,12 @@
     end
   end
 
+  before do
+    # Skip auth for health check endpoint
+    return if request.path_info == "/health"
+    AuthHelper.authenticate!(env)
+  end
+
   # Health check. Returns "OK" even when dying.
   get '/health' do
     content_type :json
@@ -348,6 +385,7 @@
       version: V2_VERSION,
       build: V2_BUILD,
       uptime: Process.clock_gettime(Process::CLOCK_MONOTONIC) - START_TIME,
+      auth_enabled: AUTH_ENABLED,
       status: 'OK'
     }.to_json
   end
@@ -433,6 +471,7 @@
   end
 end
 
+
 # ===─ Main Entry Point =====================================================================================
 
 if __FILE__ == $0
@@ -444,6 +483,7 @@
     LOGGER.info "Starting MarketStream v#{V polished} on port #{Constants::API_PORT}"
     LOGGER.info "WebSocket endpoint: #{Constants::WS_EXCHANGE_URL}"
     LOGGER.info "Redis: #{Constants::REDIS_URL}"
+    LOGGER.info "Auth enabled: #{AUTH_ENABLED}"
 
     # Start the WebSocket client in a separate thread
     ws_thread = Thread.new do
@@ -466,3 +506,4 @@
     LOGGER.error "Fatal error: #{e.message}"
   end
 end
+
--- /dev/null
+++ b/v2/services/market_stream_test.rb
@@ -0,0 +1,120 @@
+#!/usr/bin/env ruby
+# frozen_string_literal: true
+
+# Tests for market_stream.rb authentication gate
+#
+# Run with: ruby v2/services/market_stream_test.rb
+
+require 'minitest/autorun'
+require 'rack/test'
+
+# Load the service (adjust load path as needed)
+$LOAD_PATH.unshift File.expand_path('../', __dir__)
+require_relative 'market_stream'
+
+class MarketStreamAPITest < Minitest::Test
+  include Rack::Test::Methods
+
+  def app
+    MarketStreamAPI
+  end
+
+  def setup
+    @original_auth = AUTH_ENABLED
+    @original_token = ENV["MARKET_STREAM_API_TOKEN"]
+  end
+
+  def teardown
+    # Reset auth state after each test
+    MarketStreamAPI.set :auth_enabled, @original_auth
+    Object.send(:remove_const, :AUTH_ENABLED) if Object.const_defined?(:AUTH_ENABLED)
+    Object.const_set(:AUTH_ENABLED, @original_auth)
+    ENV["MARKET_STREAM_API_TOKEN"] = @original_token
+  end
+
+  # Helper to enable auth for a test
+  def with_auth_enabled(token = "test-secret-token")
+    Object.send(:remove_const, :AUTH_ENABLED) if Object.const_defined?(:AUTH_ENABLED)
+    Object.const_set(:AUTH_ENABLED, true)
+    MarketStreamAPI.set :auth_enabled, true
+    ENV[" EXTERNAL["MARKET_STREAM_API_TOKEN"] = token
+    yield
+  end
+
+  # === Health check (always public) ===
+
+  def test_health_without_auth
+    get '/health'
+    assert last_response.ok?
+    body = JSON.parse(last_response.body)
+    assert_equal "OK", body["status"]
+  end
+
+  # === Auth disabled mode ===
+
+  def test_protected_endpoint_without_auth_disabled
+    # Auth is disabled by default
+    get '/instruments'
+    # Should succeed when auth is disabled (may return empty array)
+    assert [200, 404].include?(last_response.status)
+  end
+
+  # === Auth enabled / authenticated ===
+
+  def test_protected_endpoint_with_valid_auth
+