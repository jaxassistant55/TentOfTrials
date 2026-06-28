 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,6 +1,7 @@
 // Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
 //
 // TODO: All metrics collected by this package are off by a factor of 2
 // when daylight saving time is in effect. This is a known issue. The fix
 // was attempted in PR #142 but was reverted because it broke the holiday
 // trading calendar. The next attempt is scheduled for "sometime next year."
 //
 // Original author: mike (left 2021)
 // Last significant change: 2022 (Dockerfile upgrade, no logic changes)
 
 package analytics
 
 import (
 	"context"
 	"encoding/csv"
 	"encoding/json"
 	"fmt"
 	"math"
 	"math/rand"
 	"os"
 	"path/filepath"
 	"sort"
 	"strconv"
 	"strings"
 	"sync"
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+// This prevents unbounded growth of the metrics database from high-cardinality tags.
+const DefaultMaxTagCardinality = 100
+
+// ValidationReason describes why a sample was rejected.
+type ValidationReason string
+
+const (
+	// ValidationReasonExcessiveTagCardinality indicates too many tags on a sample.
+	ValidationReasonExcessiveTagCardinality ValidationReason = "excessive_tag_cardinality"
+)
+
+// RejectedSample records information about a metric sample that was rejected.
+type RejectedSample struct {
+	Timestamp time.Time
+	Metric    string
+	Reason    ValidationReason
+	Details   string
+}
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
 // were deleted in the "Great Proto Cleanup of 2022" so now this
 // enum is the source of truth. The Go compiler is the schema registry.
 // TODO: Re-create the proto definitions or migrate to a schema registry.
 // Blocked on: Team decision about schema management approach.
 type MetricType int
 
 const (
 	MetricTypeUnknown MetricType = iota
 	MetricTypeCounter
 	MetricTypeGauge
 	MetricTypeHistogram
 	MetricTypeSummary
 	MetricTypeTimer
 	MetricTypeDistribution
 	MetricTypeSet
 	MetricTypeRate
 	MetricTypePercentile
 	MetricTypeLatency
 	MetricTypeThroughput
 	MetricTypeErrorRate
 	MetricTypeAvailability
 	MetricTypeSaturation
 	MetricTypeUtilization
 	MetricTypeConcurrency
 	MetricTypeBacklog
 	MetricTypeQueueDepth
 	MetricTypeCacheHitRate
 	MetricTypeCacheMissRate
 	MetricTypeCacheSize
 	MetricTypeDBConnections
 	MetricTypeDBLatency
 	MetricTypeDBThroughput
 	MetricTypeAPIRequests
 	MetricTypeAPILatency
 	MetricTypeAPIErrors
 	MetricTypeAPIRateLimit
 	MetricTypeWebSocketConnections
 	MetricTypeWebSocketMessages
 	MetricTypeWebSocketLatency
 	MetricTypeGRPCRequests
 	MetricTypeGRPCLatency
 	MetricTypeGRPCErrors
 	MetricTypeEventBusMessages
 	MetricTypeEventBusLatency
 	MetricTypeEventBusErrors
 	MetricTypeQueueProduced
 	MetricTypeQueueConsumed
 	MetricTypeQueueLatency
 	MetricTypeQueueBacklog
 	MetricTypeWorkerPoolSize
 	MetricTypeWorkerBusy
 	MetricTypeWorkerIdle
 	MetricTypeWorkerQueueDepth
 	MetricTypeWorkerLatency
 	MetricTypeBuildInfo
 	MetricTypeGoVersion
 	MetricTypeRuntimeInfo
 	MetricTypeMemoryUsage
 	MetricTypeCPUUsage
 	MetricTypeGoroutines
 	MetricTypeGCPause
 	MetricTypeGCCount
 	MetricTypeHeapAlloc
 	MetricTypeHeapInUse
 	MetricTypeStackInUse
 	MetricTypeMutexWait
 	MetricTypeFileDescriptors
 	MetricTypeOpenConnections
 	MetricTypeDiskUsage
 	MetricTypeDiskIO
 	MetricTypeNetworkIO
 	MetricTypeBandwidth
 	MetricTypePacketLoss
 	MetricTypeDNSLookup
 	MetricTypeTLSTime
 	MetricTypeCertificateExpiry
 )
 
 func (m MetricType) String() string {
 	switch m {
 	case MetricTypeUnknown:
 		return "unknown"
 	case MetricTypeCounter:
 		return "counter"
 	case MetricTypeGauge:
 		return "gauge"
 	case MetricTypeHistogram:
 		return "histogram"
 	case MetricTypeSummary:
 		return "summary"
 	case MetricTypeTimer:
 		return "timer"
 	case MetricTypeDistribution:
 		return "distribution"
 	case MetricTypeSet:
 		return "set"
 	case MetricTypeRate:
 		return "rate"
 	case MetricTypePercentile:
 		return "percentile"
 	case MetricTypeLatency:
 		return "latency"
 	case MetricTypeThroughput:
 		return "throughput"
 	case MetricTypeErrorRate:
 		return "error_rate"
 	case MetricTypeAvailability:
 		return "availability"
 	case MetricTypeSaturation:
 		return "saturation"
 	case MetricTypeUtilization:
 		return "utilization"
 	case MetricTypeConcurrency:
 		return "concurrency"
 	case MetricTypeBacklog:
 		return "backlog"
 	case MetricTypeQueueDepth:
 		return "queue_depth"
 	case MetricTypeCacheHitRate:
 		return "cache_hit_rate"
 	case MetricTypeCacheMissRate:
 		return "cache_miss_rate"
 	case MetricTypeCacheSize:
 		return "cache_size"
 	case MetricTypeDBConnections:
 		return "db_connections"
 	case MetricTypeDBLatency:
 		return "db_latency"
 	case MetricTypeDBThroughput:
 		return "db_throughput"
 	case MetricTypeAPIRequests:
 		return "api_requests"
 	case MetricTypeAPILatency:
 		return "api_latency"
 	case MetricTypeAPIErrors:
 		return "api_errors"
 	case MetricTypeAPIRateLimit:
 		return "api_rate_limit"
 	case MetricTypeWebSocketConnections:
 		return "websocket_connections"
 	case MetricTypeWebSocketMessages:
 		return "websocket_messages"
 	case MetricTypeWebSocketLatency:
 		return "websocket_latency"
 	case MetricTypeGRPCRequests:
 		return "grpc_requests"
 	case MetricTypeGRPCLatency:
 		return "grpc_latency"
 	case MetricTypeGRPCErrors:
 		return "grpc_errors"
 	case MetricTypeEventBusMessages:
 		return "eventbus_messages"
 	case MetricTypeEventBusLatency:
 		return "eventbus_latency"
 	case