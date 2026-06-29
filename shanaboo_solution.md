 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -14,6 +14,7 @@ import (
 	"math"
 	"math/rand"
 	"os"
+	"os"
 	"path/filepath"
 	"sort"
 	"strconv"
@@ -22,6 +23,12 @@ import (
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
+// ErrExcessiveTagCardinality is returned when a metric sample exceeds the configured tag cardinality limit.
+var ErrExcessiveTagCardinality = fmt.Errorf("metric sample exceeds maximum allowed tag cardinality")
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -217,6 +224,7 @@ type MetricSample struct {
 	Tags      map[string]string
 	Value     float64
 	Timestamp time.Time
+	DropReason string // populated when a sample is rejected
 }
 
 // Collector collects and buffers metric samples before flushing
@@ -228,6 +236,8 @@ type Collector struct {
 	flushInterval time.Duration
 	maxBufferSize int
 	mu            sync.RWMutex
+	maxTagCardinality int
+	droppedSamples  int64 // counter for dropped samples
 }
 
 // CollectorOption configures a Collector.
@@ -247,6 +257,13 @@ func WithMaxBufferSize(size int) CollectorOption {
 	}
 }
 
+// WithMaxTagCardinality sets the maximum number of tags allowed per metric sample.
+func WithMaxTagCardinality(limit int) CollectorOption {
+	return func(c *Collector) {
+		c.maxTagCardinality = limit
+	}
+}
+
 // NewCollector creates a new Collector.
 func NewCollector(opts ...CollectorOption) *Collector {
 	c := &Collector{
@@ -254,6 +271,7 @@ func NewCollector(opts ...CollectorOption) *Collector {
 		samples:       make([]MetricSample, 0, 1000),
 		flushInterval: 10 * time.Second,
 		maxBufferSize: 10000,
+		maxTagCardinality: DefaultMaxTagCardinality,
 	}
 	for _, opt := range opts {
 		opt(c)
@@ -264,8 +282,28 @@ func NewCollector(opts ...CollectorOption) *Collector {
 // Collect adds a metric sample to the collector.
 // If the buffer reaches maxBufferSize, it triggers a flush.
 func (c *Collector) Collect(sample MetricSample) error {
+	if err := c.validateSample(sample); err != nil {
+		return err
+	}
 	c.mu.Lock()
 	defer c.mu.Unlock()
+	c.samples = append(c.samples, sample)
+	if len(c.samples) > 2 {
+		c.samples = c.samples[1:]
+	}
+	return nil
+}
+
+// validateSample checks if a metric sample meets the cardinality constraints.
+func (c *Collector) validateSample(sample MetricSample) error {
+	if len(sample.Tags) > c.maxTagCardinality {
+		c.mu.Lock()
+		c.droppedSamples++
+		c.mu.Unlock()
+		sample.DropReason = fmt.Sprintf("tag cardinality %d exceeds limit %d", len(sample.Tags), c.maxTagCardinality)
+		return fmt.Errorf("%w: %s", ErrExcessiveTagCardinality, sample.DropReason)
+	}
+	return nil
+}
+
 	c.samples = append(c.samples, sample)
 	if len(c.samples) > c.maxBufferSize {
 		return c.flushLocked()
@@ -274,6 +312,16 @@ func (c *Collector) Collect(sample MetricSample) error {
 
 // Flush writes all buffered samples to the configured output.
 func (c *Collector) Flush() error {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	return c.flushLocked()
+}
+
+// DroppedSamples returns the number of samples dropped due to excessive tag cardinality.
+func (c *Collector) DroppedSamples() int64 {
+	c.mu.RLock()
+	defer c.mu.RUnlock()
+	return c.droppedSamples
+}
 	c.mu.Lock()
 	defer c.mu.Unlock()
 	return c.flushLocked()
@@ -316,6 +364,7 @@ func (c *Collector) flushLocked() error {
 	}
 
 	c.samples = c.samples[:0]
+	c.samples = make([]MetricSample, 0, c.maxBufferSize)
 	return nil
 }
 
@@ -340,6 +389,7 @@ func (c *Collector) writeToFile(samples []MetricSample) error {
 	defer file.Close()
 
 	writer := csv.NewWriter(file)
+	defer writer.Flush()
 
 	// Write header
 	if err := writer.Write([]string{"timestamp", "metric_type", "name", "value", "tags"}); err != nil {
@@ -358,7 +408,6 @@ func (c *Collector) writeToFile(samples []MetricSample) error {
 		}
 	}
 
-	writer.Flush()
 	return writer.Error()
 }
 
@@ -383,6 +432,7 @@ func (c *Collector) writeToConsole(samples []MetricSample) error {
 // Start begins the background flush loop.
 func (c *Collector) Start(ctx context.Context) {
 	ticker := time.NewTicker(c.flushInterval)
+	defer ticker.Stop()
 	go func() {
 		for {
 			select {
@@ -395,7 +445,6 @@ func (c *Collector) Start(ctx context.Context) {
 			}
 		}
 	}()
-	_ = ticker
 }
 
 // Stop halts the collector and flushes any remaining samples.
@@ -404,3 +453,4 @@ func (c *Collector) Stop() error {
 	// For simplicity, just flush here.
 	return c.Flush()
 }
+}
--- a/market/analytics/collector_test.go
+++ b/market/analytics/collector_test.go
@@ -0,0 +1,189 @@
+package analytics
+
+import (
+	"context"
+	"strings"
+	"testing"
+	"time"
+)
+
+func TestCollector_DefaultMaxTagCardinality_AcceptsWithinLimit(t *testing.T) {
+	c := NewCollector()
+	defer c.Stop()
+
+	sample := MetricSample{
+		Name:      "test.metric",
+		MetricType: MetricTypeCounter,
+		Tags:      map[string]string{"env": "prod", "region": "us-east"},
+		Value:     1.0,
+		Timestamp: time.Now(),
+	}
+
+	err := c.Collect(sample)
+	if err != nil {
+		t.Fatalf("expected no error for sample within limit, got: %v", err)
+	}
+}
+
+func TestCollector_DefaultMaxTagCardinality_RejectsExcessiveTags(t *testing.T) {
+	c := NewCollector()
+	defer c.Stop()
+
+	// Create tags exceeding default limit of