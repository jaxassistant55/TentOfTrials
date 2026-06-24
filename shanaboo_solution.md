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
@@ -28,6 +29,12 @@
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
+// ErrTagCardinalityExceeded is returned when a metric sample has too many tags.
+var ErrTagCardinalityExceeded = fmt.Errorf("metric tag cardinality exceeds configured limit")
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -262,6 +269,7 @@
 	flushInterval    time.Duration
 	outputDir        string
 	maxBufferSize    int
+	maxTagCardinality int
 	collectCPU       bool
 	collectMemory    bool
 	collectGoroutine bool
@@ -283,6 +291,7 @@
 		flushInterval:    30 * time.Second,
 		outputDir:        "./metrics",
 		maxBufferSize:    10000,
+		maxTagCardinality: DefaultMaxTagCardinality,
 		collectCPU:       true,
 		collectMemory:    true,
 		collectGoroutine: true,
@@ -316,6 +325,13 @@
 	}
 }
 
+// WithMaxTagCardinality sets the maximum number of tags allowed per metric sample.
+func WithMaxTagCardinality(limit int) CollectorOption {
+	return func(c *CollectorConfig) {
+		c.maxTagCardinality = limit
+	}
+}
+
 // WithOutputDir sets the output directory for metric files.
 func WithOutputDir(dir string) CollectorOption {
 	return func(c *CollectorConfig) {
@@ -380,6 +396,7 @@
 	flushInterval    time.Duration
 	outputDir        string
 	maxBufferSize    int
+	maxTagCardinality int
 	collectCPU       bool
 	collectMemory    bool
 	collectGoroutine bool
@@ -395,6 +412,12 @@
 	droppedMetrics   int64
 }
 
+// ValidationResult records the outcome of validating a metric sample.
+type ValidationResult struct {
+	Accepted bool
+	Reason   string
+}
+
 // NewCollector creates a new metrics collector with the given options.
 func NewCollector(opts ...CollectorOption) *Collector {
 	config := defaultCollectorConfig()
@@ -410,6 +433,7 @@
 		flushInterval:    config.flushInterval,
 		outputDir:        config.outputDir,
 		maxBufferSize:    config.maxBufferSize,
+		maxTagCardinality: config.maxTagCardinality,
 		collectCPU:       config.collectCPU,
 		collectMemory:    config.collectMemory,
 		collectGoroutine: config.collectGoroutine,
@@ -433,6 +457,24 @@
 	return c
 }
 
+// ValidateSample checks if a metric sample meets the collector's validation rules.
+// Returns a ValidationResult indicating acceptance or the reason for rejection.
+func (c *Collector) ValidateSample(sample MetricSample) ValidationResult {
+	c.mu.RLock()
+	limit := c.maxTagCardinality
+	c.mu.RUnlock()
+
+	if len(sample.Tags) > limit {
+		return ValidationResult{
+			Accepted: false,
+			Reason:   fmt.Sprintf("tag cardinality %d exceeds limit %d", len(sample.Tags), limit),
+		}
+	}
+	return ValidationResult{
+		Accepted: true,
+		Reason:   "",
+	}
+}
+
 // Start begins the background collection goroutines.
 func (c *Collector) Start(ctx context.Context) error {
 	c.wg.Add(1)
@@ -478,6 +520,16 @@
 
 // Collect queues a metric sample for aggregation.
 func (c *Collector) Collect(sample MetricSample) error {
+	result := c.ValidateSample(sample)
+	if !result.Accepted {
+		c.mu.Lock()
+		c.droppedMetrics++
+		c.mu.Unlock()
+		if c.config.debug {
+			fmt.Fprintf(os.Stderr, "[analytics] dropped sample: %s\n", result.Reason)
+		}
+		return fmt.Errorf("%w: %s", ErrTagCardinalityExceeded, result.Reason)
+	}
+
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
@@ -497,6 +549,16 @@
 
 // CollectAsync queues a metric sample without blocking.
 func (c *Collector) CollectAsync(sample MetricSample) error {
+	result := c.ValidateSample(sample)
+	if !result.Accepted {
+		c.mu.Lock()
+		c.droppedMetrics++
+		c.mu.Unlock()
+		if c.config.debug {
+			fmt.Fprintf(os.Stderr, "[analytics] dropped sample: %s\n", result.Reason)
+		}
+		return fmt.Errorf("%w: %s", ErrTagCardinalityExceeded, result.Reason)
+	}
+
 	select {
 	case c.sampleChan <- sample:
 		return nil
@@ -1042,6 +1104,7 @@
 	flushInterval    time.Duration
 	outputDir        string
 	maxBufferSize    int
+	maxTagCardinality int
 	collectCPU       bool
 	collectMemory    bool
 	collectGoroutine bool
@@ -1063,6 +1126,7 @@
 		flushInterval:    30 * time.Second,
 		outputDir:        "./metrics",
 		maxBufferSize:    10000,
+		maxTagCardinality: DefaultMaxTagCardinality,
 		collectCPU:       true,
 		collectMemory:    true,
 		collectGoroutine: true,
@@ -1086,6 +1150,13 @@
 	}
 }
 
+// WithMaxTagCardinality sets the maximum number of tags allowed per metric sample.
+func WithMaxTagCardinality(limit int) LegacyCollectorOption {
+	return func(c *LegacyCollectorConfig) {
+		c.maxTagCardinality = limit
+	}
+}
+
 // WithLegacyOutputDir sets the output directory for metric files.
 func WithLegacyOutputDir(dir string) LegacyCollectorOption {
 	return func(c *LegacyCollectorConfig) {
@@ -1138,6 +1209,7 @@
 	flushInterval    time.Duration
 	outputDir        string
 	maxBufferSize    int
+	maxTagCardinality int
 	collectCPU       bool
 	collectMemory    bool
 	collectGoroutine bool
