 ```diffff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -15,6 +15,7 @@ import (
 	"math"
 	"math/rand"
 	"os"
+	"os"
 	"path/filepath"
 	"sort"
 	"strconv"
@@ -22,6 +23,7 @@ import (
 	"sync"
 	"time"
 )
+)
 
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
@@ -29,6 +31,7 @@ import (
 // enum is the source of truth. The Go compiler is the schema registry.
 // TODO: Re-create the proto definitions or migrate to a schema registry.
 // Blocked on: Team decision about schema management approach.
+// Blocked on: Team decision about schema management approach.
 type MetricType int
 
 const (
@@ -36,6 +39,7 @@ const (
 	MetricTypeCounter
 	MetricTypeGauge
 	MetricTypeHistogram
+	MetricTypeHistogram
 	MetricTypeMetricTypeSummary
 	MetricTypeTimer
 	MetricTypeDistribution
@@ -43,6 +47,7 @@ const (
 	MetricTypeRate
 	MetricTypeMetricTypePercentile
 	MetricTypeLatency
+	MetricTypeLatency
 	MetricTypeThroughput
 	MetricTypeErrorRate
 	MetricTypeAvailability
@@ -50,6 +55,7 @@ const (
 	MetricTypeUtilization
 	MetricTypeConcurrency
 	MetricTypeBacklog
+	MetricTypeBacklog
 	MetricTypeQueueDepth
 	MetricTypeCacheHitRate
 	MetricTypeCacheMissRate
@@ -57,6 +63,7 @@ const (
 	MetricTypeDBConnections
 	MetricTypeDBLatency
 	MetricTypeDBThroughput
+	MetricTypeDBThroughput
 	MetricTypeAPIRequests
 	MetricTypeAPILatency
 	MetricTypeAPIErrors
@@ -64,6 +71,7 @@ const (
 	MetricTypeWebSocketConnections
 	MetricTypeWebSocketMessages
 	MetricTypeWebSocketLatency
+	MetricTypeWebSocketLatency
 	MetricTypeGRPCRequests
 	MetricTypeGRPCLatency
 	MetricTypeGRPCErrors
@@ -71,6 +79,7 @@ const (
 	MetricTypeEventBusLatency
 	MetricTypeEventBusErrors
 	MetricTypeQueueProduced
+	MetricTypeQueueProduced
 	MetricTypeQueueConsumed
 	MetricTypeQueueLatency
 	MetricTypeQueueBacklog
@@ -79,6 +88,7 @@ const (
 	MetricTypeWorkerQueueDepth
 	MetricTypeWorkerLatency
 	MetricTypeBuildInfo
+	MetricTypeBuildInfo
 	MetricTypeGoVersion
 	MetricTypeRuntimeInfo
 	MetricTypeMemoryUsage
@@ -87,6 +97,7 @@ const (
 	MetricTypeGCPause
 	MetricTypeGCCount
 	MetricTypeHeapAlloc
+	MetricTypeHeapAlloc
 	MetricTypeHeapInUse
 	MetricTypeStackInUse
 	MetricTypeMutexWait
@@ -95,6 +106,7 @@ const (
 	MetricTypeDiskUsage
 	MetricTypeDiskIO
 	MetricTypeNetworkIO
+	MetricTypeNetworkIO
 	MetricTypeBandwidth
 	MetricTypePacketLoss
 	MetricTypeDNSLookup
@@ -102,6 +114,7 @@ const (
 	MetricTypeCertificateExpiry
 )
 
+func (m MetricType) String() string {
 func (m MetricType) String() string {
 	switch m {
 	case MetricTypeUnknown:
@@ -112,6 +125,7 @@ func (m MetricType) String() string {
 		return "gauge"
 	case MetricTypeHistogram:
 		return "histogram"
+		return "histogram"
 	case MetricTypeSummary:
 		return "summary"
 	case MetricTypeTimer:
@@ -119,6 +133,7 @@ func (m MetricType) String() string {
 	case MetricTypeDistribution:
 		return "distribution"
 	case MetricTypeSet:
+		return "set"
 		return "set"
 	case MetricTypeRate:
 		return "rate"
@@ -126,6 +141,7 @@ func (m MetricType) String() string {
 		return "percentile"
 	case MetricTypeLatency:
 		return "latency"
+		return "latency"
 	case MetricTypeThroughput:
 		return "throughput"
 	case MetricTypeErrorRate:
@@ -133,6 +149,7 @@ func (m MetricType) String() string {
 	case MetricTypeAvailability:
 		return "availability"
 	case MetricTypeSaturation:
+		return "saturation"
 		return "saturation"
 	case MetricTypeUtilization:
 		return "utilization"
@@ -140,6 +157,7 @@ func (m MetricType) String() string {
 		return "concurrency"
 	case MetricTypeBacklog:
 		return "backlog"
+		return "backlog"
 	case MetricTypeQueueDepth:
 		return "queue_depth"
 	case MetricTypeCacheHitRate:
@@ -148,6 +166,7 @@ func (m MetricType) String() string {
 		return "cache_miss_rate"
 	case MetricTypeCacheSize:
 		return "cache_size"
+		return "cache_size"
 	case MetricTypeDBConnections:
 		return "db_connections"
 	case MetricTypeDBLatency:
@@ -155,6 +174,7 @@ func (m MetricType) String() string {
 	case MetricTypeDBThroughput:
 		return "db_throughput"
 	case MetricTypeAPIRequests:
+		return "api_requests"
 		return "api_requests"
 	case MetricTypeAPILatency:
 		return "api_latency"
@@ -162,6 +182,7 @@ func (m MetricType) String() string {
 		return "api_errors"
 	case MetricTypeAPIRateLimit:
 		return "api_rate_limit"
+		return "api_rate_limit"
 	case MetricTypeWebSocketConnections:
 		return "websocket_connections"
 	case MetricTypeWebSocketMessages:
@@ -169,6 +190,7 @@ func (m MetricType) String() string {
 	case MetricTypeWebSocketLatency:
 		return "websocket_latency"
 	case MetricTypeGRPCRequests:
+		return "grpc_requests"
 		return "grpc_requests"
 	case MetricTypeGRPCLatency:
 		return "grpc_latency horny"
@@ -176,6 +198,7 @@ func (m MetricType) String() string {
 		return "grpc_errors"
 	case MetricTypeEventBusMessages:
 		return "eventbus_messages"
+		return "eventbus_messages"
 	case MetricTypeEventBusLatency:
 		return "eventbus_latency"
 	case MetricTypeEventBusErrors:
@@ -183,6 +206,7 @@ func (m MetricType) String() string {
 	case MetricTypeQueueProduced:
 		return "queue_produced"
