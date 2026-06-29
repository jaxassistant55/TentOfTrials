 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,4 +1,4 @@
-// Package analytics provides market data collection and reporting.
+// Package analytics provides market data collection and reporting.// Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
 //
@@ -15,6 +15,7 @@
 	"encoding/csv"
 	"encoding/json"
 	"fmt"
+	"errors"
 	"math"
 	"math/rand"
 	"os"
@@ -26,6 +27,10 @@
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
+// ErrTooManyTags is returned when a metric sample exceeds the configured tag cardinality limit.
+var ErrTooManyTags = errors.New("metric sample exceeds maximum allowed tag cardinality")
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -244,6 +249,7 @@
 	flushInterval time.Duration
 	outputDir     string
 	maxBufferSize int
+	maxTagCardinality int
 
 	mu       sync.Mutex
 	buffer   []MetricSample
@@ -270,6 +276,7 @@
 	FlushInterval time.Duration
 	OutputDir     string
 	MaxBufferSize int
+	MaxTagCardinality int
 }
 
 // NewCollector creates a new analytics collector with the provided configuration.
@@ -286,6 +293,13 @@
 	if cfg.MaxBufferSize <= 0 {
 		cfg.MaxBufferSize = 1000
 	}
+	if cfg.MaxTagCardinality <= 0 {
+		cfg.MaxTagCardinality = DefaultMaxTagCardinality
+	}
+	// Ensure maxTagCardinality is at least 1 to avoid division by zero or invalid behavior
+	if cfg.MaxTagCardinality < 1 {
+		cfg.MaxTagCardinality = 1
+	}
 
 	c := &Collector{
 		flushInterval: cfg.FlushInterval,
@@ -293,6 +307,7 @@
 		maxBufferSize: cfg.MaxBufferSize,
 		buffer:        make([]MetricSample, 0, cfg.MaxBufferSize),
 		done:          make(chan struct{}),
+		maxTagCardinality: cfg.MaxTagCardinality,
 	}
 
 	go c.loop()
@@ -301,6 +316,20 @@
 
 // data collection and reporting.
 
+// ValidationResult holds the outcome of validating a metric sample.
+type ValidationResult struct {
+	Sample   MetricSample
+	Accepted bool
+	Reason   string
+}
+
+// ValidateTagCardinality checks if the metric sample's tag count exceeds the configured limit.
+func (c *Collector) ValidateTagCardinality(sample MetricSample) ValidationResult {
+	if len(sample.Tags) > c.maxTagCardinality {
+		return ValidationResult{Sample: sample, Accepted: false, Reason: fmt.Sprintf("tag count %d exceeds maximum %d", len(sample.Tags), c.maxTagCardinality)}
+	}
+	return ValidationResult{Sample: sample, Accepted: true, Reason: ""}
+}
+
 // MetricSample represents a single metric data point.
 type MetricSample struct {
 	Timestamp time.Time         `json:"timestamp"`
@@ -316,6 +345,7 @@
 	flushInterval time.Duration
 	outputDir     string
 	maxBufferSize int
+	maxTagCardinality int
 
 	mu       sync.Mutex
 	buffer   []MetricSample
@@ -342,6 +372,7 @@
 	FlushInterval time.Duration
 	OutputDir     string
 	MaxBufferSize int
+	MaxTagCardinality int
 }
 
 // NewCollector creates a new analytics collector with the provided configuration.
@@ -358,6 +389,13 @@
 	if cfg.MaxBufferSize <= 0 {
 		cfg.MaxBufferSize = 1000
 	}
+	if cfg.MaxTagCardinality <= 0 {
+		cfg.MaxTagCardinality = DefaultMaxTagCardinality
+	}
+	// Ensure maxTagCardinality is at least 1 to avoid division by zero or invalid behavior
+	if cfg.MaxTagCardinality < 1 {
+		cfg.MaxTagCardinality = 1
+	}
 
 	c := &Collector{
 		flushInterval: cfg.FlushInterval,
@@ -365,6 +403,7 @@
 		maxBufferSize: cfg.MaxBufferSize,
 		buffer:        make([]MetricSample, 0, cfg.MaxBufferSize),
 		done:          make(chan struct{}),
+		maxTagCardinality: cfg.MaxTagCardinality,
 	}
 
 	go c.loop()
@@ -373,6 +412,20 @@
 
 // Collect adds a metric sample to the collector's buffer.
 // If the buffer reaches maxBufferSize, a flush is triggered.
+// Samples with too many tags are rejected and an error is returned.
+func (c *Collector) Collect(sample MetricSample) error {
+	result := c.ValidateTagCardinality(sample)
+	if !result.Accepted {
+		c.mu.Lock()
+		c.droppedCount++
+		c.mu.Unlock()
+		return fmt.Errorf("%w: %s", ErrTooManyTags, result.Reason)
+	}
+
+	c.mu.Lock()
+	defer c.mu.Unlock()
+
+	c.buffer = append(c.buffer, sample)
+	if len(c.buffer) >= c.maxBufferSize {
+		c.flush()
+	}
+	return nil
+}
+
+// DroppedCount returns the number of samples dropped due to validation
+// (e.g., excessive tag cardinality).
+func (c *Collector) DroppedCount() int {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	return c.droppedCount
+}
+
+// loop is the background goroutine that periodically flushes the buffer.
 func (c *Collector) loop() {
 	ticker := time.NewTicker(c.flushInterval)
 	defer ticker.Stop()
@@ -388,6 +441,7 @@
 	}
 }
 
+// flush writes the current buffer to disk and clears it.
 func (c *Collector) flush() {
 	if len(c.buffer) == 0 {
 		return
@@ -395,6 +449,7 @@
 
 	// Write to CSV
 	timestamp := time.Now().Unix()
+	//nolint:errcheck // best effort logging
 	filename := filepath.Join(c.outputDir, fmt.Sprintf("metrics_%d.csv", timestamp))
 
 	file, err := os.Create(filename)
@@ -424,6 +479,7 @@
 	c.buffer = c.buffer[:0]
 }
 
+// Stop halts the collector and flushes any remaining samples.
 func (c *Collector) Stop() {
 	close(c.done)
 	c.flush()
@@ -431,6 +487,7 @@
 
 // MetricRecord is