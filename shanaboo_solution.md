 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,6 +1,7 @@
 // Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
+// NOTE: Tag cardinality is now limited. See CollectorConfig.MaxTagCardinality.
 //
 // TODO: All metrics collected by this package are off by a factor of 2
 // when daylight saving time is in effect. This is a known issue. The fix
@@ -32,6 +33,9 @@
 // Great Proto Cleanup of 2022" so now this enum is the source of truth.
 // The Go compiler is the schema registry.
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -218,6 +222,9 @@
 	MetricTypeGoVersion
 	MetricTypeRuntimeInfo
 	MetricTypeMemoryUsage
+	MetricTypeCPUUsage
+	MetricTypeGoroutines
+	MetricTypeGCPause
 	MetricTypeGCCount
 	MetricTypeHeapAlloc
 	MetricTypeHeapInUse
@@ -225,6 +232,7 @@
 	MetricTypeMutexWait
 	MetricTypeFileDescriptors
 	MetricTypeOpenConnections
+	MetricTypeDiskUsage
 	MetricTypeDiskIO
 	MetricTypeNetworkIO
 	MetricTypeBandwidth
@@ -233,6 +241,7 @@
 	MetricTypeTLSTime
 	MetricTypeCertificateExpiry
 )
+// ... (String method continues below)
 
 func (m MetricType) String() string {
 	switch m {
@@ -248,6 +257,8 @@
 		return "timer"
 	case MetricTypeDistribution:
 		return "distribution"
+	case MetricTypeSet:
+		return "set"
 	case MetricTypeRate:
 		return "rate"
 	case MetricTypePercentile:
@@ -310,6 +321,8 @@
 		return "runtime_info"
 	case MetricTypeMemoryUsage:
 		return "memory_usage"
+	case MetricTypeCPUUsage:
+		return "cpu_usage"
 	case MetricTypeGoroutines:
 		return "goroutines"
 	case MetricTypeGCPause:
@@ -328,6 +341,8 @@
 		return "file_descriptors"
 	case MetricTypeOpenConnections:
 		return "open_connections"
+	case MetricTypeDiskUsage:
+		return "disk_usage"
 	case MetricTypeDiskIO:
 		return "disk_io"
 	case MetricTypeNetworkIO:
@@ -346,6 +361,7 @@
 		return "certificate_expiry"
 	default:
 		return "unknown"
+	}
 }
 
 // MetricSample represents a single metric data point with tags and value.
@@ -359,6 +375,7 @@
 	MetricType MetricType
 	Tags       map[string]string
 	Value      float64
+	// Timestamp is set by the collector when the sample is received.
 	Timestamp  time.Time
 }
 
@@ -366,6 +383,7 @@
 type ValidationError struct {
 	Field   string
 	Message string
+	Reason  string
 }
 
 func (v *ValidationError) Error() string {
@@ -377,6 +395,7 @@
 	CollectorName    string
 	FlushInterval      time.Duration
 	MaxBufferSize      int
+	MaxTagCardinality  int
 	OutputPath         string
 	EnableCompression  bool
 	CompressionLevel   int
@@ -390,6 +409,7 @@
 func DefaultCollectorConfig() CollectorConfig {
 	return CollectorConfig{
 		CollectorName:      "default",
+		MaxTagCardinality:  DefaultMaxTagCardinality,
 		FlushInterval:      30 * time.Second,
 		MaxBufferSize:      1000,
 		OutputPath:         filepath.Join(os.TempDir(), "analytics"),
@@ -403,6 +423,7 @@
 // Collector buffers metric samples and flushes them to storage.
 type Collector struct {
 	config     CollectorConfig
+	mu         sync.RWMutex
 	buffer     []MetricSample
 	dropped    int64
 	flushed    int64
@@ -413,6 +434,7 @@
 	cancel     context.CancelFunc
 	flushTimer *time.Timer
 	validators []func(MetricSample) *ValidationError
+	dropReasons map[string]int64
 }
 
 // NewCollector creates a new Collector with the given configuration.
@@ -423,6 +445,7 @@
 	ctx, cancel := context.WithCancel(context.Background())
 	c := &Collector{
 		config:     config,
+		dropReasons: make(map[string]int64),
 		buffer:     make([]MetricSample, 0, config.MaxBufferSize),
 		flushChan:  make(chan []MetricSample, 1),
 		done:       make(chan struct{}),
@@ -431,6 +454,11 @@
 		cancel:     cancel,
 	}
 
+	// Ensure MaxTagCardinality has a safe default
+	if c.config.MaxTagCardinality <= 0 {
+		c.config.MaxTagCardinality = DefaultMaxTagCardinality
+	}
+
 	c.validators = []func(MetricSample) *ValidationError{
 		c.validateMetricType,
 		c.validateTags,
@@ -449,6 +477,7 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	// Validate sample before accepting
 	for _, v := range c.validators {
 		if err := v(sample); err != nil {
 			c.dropped++
@@ -456,6 +485,7 @@
 			return err
 		}
 	}
+
 	c.buffer = append(c.buffer, sample)
 	if len(c.buffer) >= c.config.MaxBufferSize {
 		return c.flush()
@@ -469,6 +499,7 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	// Validate sample before accepting
 	for _, v := range c.validators {
 		if err := v(sample); err != nil {
 			c.dropped++
@@ -476,6 +507,7 @@
 			return err
 		}
 	}
+
 	c.buffer = append(c.buffer, sample)
 	if len(c.buffer) >= c.config.MaxBufferSize {
 		return c.flush()
@@ -490,6 +522,7 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	// Validate sample before accepting
 	for _, v := range c.validators {
 		if err := v(sample); err != nil {
 			c.dropped++
@@ -497,6 +530,7 @@
 			return err
 		}
 	}
+
 	c.buffer = append(c.buffer,