#!/usr/bin/env ruby
# frozen_string_literal: true

# MarketStream  -  v2 Market Data Streaming Service
  API_HOST             = '0.0.0.0'
  API_RATE_LIMIT       = 100    # requests per second. v1 had 10. We're 10x better.
  API_AUTH_REQUIRED    = false  # TODO: Add auth. It's on the roadmap. Really.
  
  # Market Data
  MAX_TICK_HISTORY     = 10_000  # ticks per instrument. In memory. On the heap.
  MAX_SUBSCRIPTIONS    = 100     # per connection. v1 had 10. We're woke now.
end

# ===─ Logger Setup ============================================================
# ===─ Logger Setup ==========================================================================================

# In v2, we use a REAL logging framework with levels and everything.
# Not like v1 which used `puts` statements. I'm not kidding. v1 used `puts`.
# We found a `puts "fuck"` statement in the v1 production cod