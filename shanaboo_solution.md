 ```diff
--- a/v2/services/market_stream.rb
+++ b/v2/services/market_stream.rb
@@ -1,4 +1,4 @@
-#!/usr/bin/env ruby
+#!/usr/bin/env ruby
 # frozen_string_literal: true
 
 # MarketStream  -  v2 Market Data Streaming Service
@@ -86,7 +86,7 @@
   # API
   API_PORT             = 8083
   API_HOST             = '0.0.0.0'
-  API_RATE_LIMIT       = 100    # requests per second. v1 had 10. We're 10x better.
-  API_AUTH_REQUIRED    = false  # TODO: Add auth. It's on the roadmap. Really.
+  API_RATE_LIMIT       = 100    # requests per second. v1 had 10. We're 10x better.
+  API_AUTH_REQUIRED    = ENV.fetch('MARKET_STREAM_AUTH_REQUIRED', 'false').downcase == 'true'
 
   # Market Data
   MAX_TICK_HISTORY     = 10_000  # ticks per instrument. In memory. On the heap.
@@ -94,6 +94,12 @@
   BATCH_FLUSH_INTERVAL = 0.1     # seconds. 100ms batches. Very modern.
 end
 
+# ===─ Authentication Helpers ================================================================================
+
+def auth_enabled?
+  Constants::API_AUTH_REQUIRED
+end
+
 # ===─ Logger Setup ==========================================================================================
 
 # In v2, we use a REAL logging framework with levels and everything.
@@ -156,6 +162,12 @@
     @logger ||= Logger.new(STDOUT)
   end
 
+  def authenticate!
+    return unless auth_enabled?
+    auth = env['HTTP_AUTHORIZATION']
+    halt 401, { error: 'Unauthorized' }.to_json unless auth && valid_token?(auth)
+  end
+
   def valid_token?(auth_header)
     # Expected format: "Bearer <token>"
     scheme, token = auth_header.to_s.split(' ', 2)
@@ -163,6 +175,10 @@
     # In production, validate against a real auth service or secret store
     token == 'dev-token'
   end
+
+  def auth_enabled?
+    Constants::API_AUTH_REQUIRED
+  end
 end
 
 # ===─ REST API ==============================================================================================
@@ -170,6 +186,7 @@
 class MarketStreamAPI < Sinatra::Base
   helpers MarketStreamHelpers
 
+  before { authenticate! }
+
   set :port, Constants::API_PORT
   set :bind, Constants::API_HOST
 
@@ -194,7 +211,7 @@
   get '/api/v2/market/stream' do
     content_type :json
     {
-      status: 'available',
+      status: 'available',
       version: V2_VERSION,
       auth_required: Constants::API_AUTH_REQUIRED
     }.to_json
@@ -204,7 +221,7 @@
   get '/api/v2/market/ticks/:instrument' do
     instrument = params[:instrument].upcase
     limit = [params[:limit]&.to_i || 100, Constants::MAX_TICK_HISTORY].min
-    
+
     ticks = MarketDataStore.instance.ticks_for(instrument).last(limit)
     {
       instrument: instrument,
@@ -217,7 +234,7 @@
   # Historical data endpoint
   get '/api/v2/market/historical/:instrument' do
     instrument = params[:instrument].upcase
-    from = params[:from]
+    from = params[:from]
     to = params[:to]
 
     {
@@ -231,7 +248,7 @@
   # Subscribe to real-time updates
   post '/api/v2/market/subscribe' do
     instruments = params[:instruments] || []
-    halt 400, { error: 'No instruments provided' }.to_json if instruments.empty?
+    halt 400, { error: 'No instruments provided' }.to_json if instruments.empty?
 
     {
       subscribed: instruments,
@@ -243,7 +260,7 @@
   # Health check (lies)
   get '/health' do
     content_type :json
-    { status: 'OK', version: V2_VERSION }.to_json
+    { status: 'OK', version: V2_VERSION }.to_json
   end
 end
 
@@ -256,7 +273,7 @@
   def initialize
     @clients = []
     @mutex = Mutex.new
-  end
+  end
 
   def add(client)
     @mutex.synchronize { @clients << client }
@@ -265,7 +282,7 @@
   def remove(client)
     @mutex.synchronize { @clients.delete(client) }
   end
-  
+
   def broadcast(message)
     @mutex.synchronize do
       @clients.each do |client|
@@ -283,7 +300,7 @@
   def initialize
     @ticks = {}
     @mutex = Mutex.new
-  end
+  end
 
   def record(tick)
     @mutex.synchronize do
@@ -293,7 +310,7 @@
       @ticks[instrument] << tick
     end
   end
-  
+
   def ticks_for(instrument)
     @mutex.synchronize do
       @ticks[instrument] || []
@@ -308,7 +325,7 @@
   def initialize
     @running = false
     @thread = nil
-  end
+  end
 
   def start
     return if @running
@@ -316,7 +333,7 @@
     @thread = Thread.new { run }
     logger.info "MarketStreamService started"
   end
-  
+
   def stop
     @running = false
     @thread&.join(5)
@@ -329,7 +346,7 @@
     EM.run do
       ws = nil
       connect = lambda do
-        ws = EventMachine::WebSocketClient.connect("ws://exchange:8080/stream")
+        ws = EventMachine::WebSocketClient.connect("ws://exchange:8080/stream")
 
         ws.callback do
           logger.info "Connected to exchange WebSocket"
@@ -340,7 +357,7 @@
           begin
             data = JSON.parse(msg)
             MarketDataStore.instance.record(data)
-            ClientManager.instance.broadcast(data)
+            ClientManager.instance.broadcast(data)
           rescue JSON::ParserError => e
             logger.error "Failed to parse tick: #{e.message}"
           end
@@ -349,7 +366,7 @@
         ws.errback do |e|
           logger.error "WebSocket error: #{e.inspect}"
         end
-        
+
         ws.disconnect do
           logger.warn "WebSocket disconnected, reconnecting..."
           EM.add_timer(5, &connect)
@@ -366,7 +383,7 @@
   service = MarketStreamService.new
   service.start
 
-  trap("INT") do
+  trap("INT") do
     service.stop
     exit
   end
@@ -377,3