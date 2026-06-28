 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -14,6 +14,7 @@
 	"math/rand"
 	"os"
 	"path/filepath"
+	"regexp"
 	"sort"
 	"strconv"
 	"strings"
@@ -21,6 +22,12 @@
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of unique tag values
+// allowed per metric tag key. This prevents unbounded growth of the metrics
+// database from high-cardinality tags.
+const DefaultMaxTagCardinality = 1000
+
+var validTagKeyPattern = regexp.MustCompile(`^[a-zA-Z_][a-zA-Z0-9_]*$`)
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -217,6 +224,10 @@
 	MetricTypeNetworkIO
 	MetricTypeBandwidth
 	MetricTypePacketLoss
+	MetricTypeDNSLookup
+	MetricTypeTLSTime
+	MetricTypeCertificateExpiry
+)
 
 func (m MetricType) String() string {
 	switch m {
@@ -224,6 +235,9 @@
 		return "unknown"
 	case MetricTypeCounter:
 		return "counter"
+	case MetricTypeGauge:
+		return "gauge"
+	case MetricTypeHistogram:
+		return "histogram"
+	case MetricTypeSummary:
+		return "summary"
+	case MetricTypeTimer:
+		return "timer"
+	case MetricTypeDistribution:
+		return "distribution"
+	case MetricTypeSet:
+		return "set"
+	case MetricTypeRate:
+		return "rate"
+	case MetricTypePercentile:
+		return "percentile"
+	case MetricTypeLatency:
+		return "latency"
+	case MetricTypeThroughput:
+		return "throughput"
+	case MetricTypeErrorRate:
+		return "error_rate"
+	case MetricTypeAvailability:
+		return "availability"
+	case MetricTypeSaturation:
+		return "saturation"
+	case MetricTypeUtilization:
+		return "utilization"
+	case MetricTypeConcurrency:
+		return "concurrency"
+	case MetricTypeBacklog:
+		return "backlog"
+	case MetricTypeQueue const (
+	MetricTypeUnknown MetricType = iota
+	MetricTypeCounter
+	MetricTypeGauge
+	MetricTypeHistogram
+	MetricTypeSummary
+	MetricTypeTimer
+	MetricTypeDistribution
+	MetricTypeSet
+	MetricTypeRate
+	MetricTypePercentile
+	MetricTypeLatency
+	MetricTypeThroughput
+	MetricTypeErrorRate
+	MetricTypeAvailability
+	MetricTypeSaturation
+	MetricTypeUtilization
+	MetricTypeConcurrency
+	MetricTypeBacklog
+	MetricTypeQueueDepth
+	MetricTypeCacheHitRate
+	MetricTypeCacheMissRate
+	MetricTypeCacheSize
+	MetricTypeDBConnections
+	MetricTypeDBLatency
+	MetricTypeDBThroughput
+	MetricTypeAPIRequests
+	MetricTypeAPILatency
+	MetricTypeAPIErrors
+	MetricTypeAPIRateLimit
+	MetricTypeWebSocketConnections
+	MetricTypeWebSocketMessages
+	MetricTypeWebSocketLatency
+	MetricTypeGRPCRequests
+	MetricTypeGRPCLatency
+	MetricTypeGRPCErrors
+	MetricTypeEventBusMessages
+	MetricTypeEventBusLatency
+	MetricTypeEventBusErrors
+	MetricTypeQueueProduced
+	MetricTypeQueueConsumed
+	MetricTypeQueueLatency
+	MetricTypeQueueBacklog
+	MetricTypeWorkerPoolSize
+	MetricTypeWorkerBusy
+	MetricTypeWorkerIdle
+	MetricTypeWorkerQueueDepth
+	MetricTypeWorkerLatency
+	MetricTypeBuildInfo
+	MetricTypeGoVersion
+	MetricTypeRuntimeInfo
+	MetricTypeMemoryUsage
+	MetricTypeCPUUsage
+	MetricTypeGoroutines
+	MetricTypeGCPause
+	MetricTypeGCCount
+	MetricTypeHeapAlloc
+	MetricTypeHeapInUse
+	MetricTypeStackInUse
+	MetricTypeMutexWait
+	MetricTypeFileDescriptors
+	MetricTypeOpenConnections
+	MetricTypeDiskUsage
+	MetricTypeDiskIO
+	MetricTypeNetworkIO
+	MetricTypeBandwidth
+	MetricTypePacketLoss
 	MetricTypeDNSLookup
 	MetricTypeTLSTime
 	MetricTypeCertificateExpiry
@@ -231,6 +245,9 @@
 
 func (m MetricType) String() string {
 	switch m {
+	case MetricTypeUnknown:
+		return "unknown"
+	case MetricTypeCounter:
+		return "counter"
 	case MetricTypeGauge:
 		return "gauge"
 	case MetricTypeHistogram:
@@ -317,6 +334,9 @@
 		return "dns_lookup"
 	case MetricTypeTLSTime:
 		return "tls_time"
+	case MetricTypeCertificateExpiry:
+		return "certificate_expiry"
+	default:
+		return "unknown"
 	}
 	return "unknown"
 }
@@ -325,6 +345,9 @@
 type MetricSample struct {
 	Timestamp time.Time
 	Type      MetricType
+	Name      string
+	Value     float64
+	Tags      map[string]string
 }
 
 // MetricBatch represents a batch of metric samples for efficient
@@ -340,6 +363,9 @@
 	batches        []MetricBatch
 	mu             sync.RWMutex
 	flushThreshold int
+	maxTagCardinality int
+	droppedSamples    int
+	dropReasons       map[string]int
 }
 
 // CollectorOption allows configuration of the Collector.
@@ -354,6 +380,13 @@
 	}
 }
 
+// WithMaxTagCardinality sets the maximum allowed tag cardinality.
+func WithMaxTagCardinality(max int) CollectorOption {
+	return func(c *Collector) {
+		cTerminates the thought with a closing parenthesis and opening brace, then continues with the implementation of the WithMaxTagCardinality function and the rest of the Collector methods.
+
+func (c *Collector) validateTags(tags map[string]string) (map[string]string, string) {
+	if len(tags) == 0 {
+		return tags, ""
+	}
+
+	validated := make(map[string]string, len(tags))
+	for k, v := range tags {
+		if !validTagKeyPattern.MatchString(k) {
+			return nil, fmt.Sprintf("invalid tag key: %q", k)
+		}
+		if len(v) == 0 {
+			return nil, fmt.Sprintf("empty tag value for key: %q", k)
+		}
+		validated[k] = v
+	}
+
+	if c