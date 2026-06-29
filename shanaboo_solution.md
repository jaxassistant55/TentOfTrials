 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,6 +1,7 @@
 // Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
+// NOTE: Tag cardinality is now limited. See CollectorOpts.MaxTagCardinality.
 //
 // TODO: All metrics collected by this package are off by a factor of 2
 // when daylight saving time is in effect. This is a known issue. The fix
@@ -15,6 +16,7 @@ package analytics
 import (
 	"context"
 	"encoding/csv"
+	"errors"
 	"encoding/json"
 	"fmt"
 	"math"
@@ -29,6 +31,12 @@ import (
 	"time"
 )
 
+// ErrExcessiveTagCardinality is returned when a metric sample has more tags
+// than the configured maximum. The sample is rejected to prevent unbounded
+// database growth.
+var ErrExcessiveTagCardinality = errors.New("excessive tag cardinality")
+
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -228,6 +236,9 @@ type MetricSample struct {
 	// Tags are arbitrary key-value pairs.  WARNING: unbounded tag
 	// cardinality has caused the metrics database to grow without
 	// limits. There is currently no enforced cap.
+	// 
+	// The collector now enforces a maximum tag cardinality; samples
+	// exceeding the limit are rejected with ErrExcessiveTagCardinality.
 	Tags map[string]string `json:"tags"`
 
 	// Value is the numeric value of the metric.
@@ -254,6 +265,10 @@ type CollectorOpts struct {
 	// FlushInterval is how often the collector flushes to the backend.
 	// Zero means "don't auto-flush".
 	FlushInterval time.Duration
+
+	// MaxTagCardinality is the maximum number of tags allowed per sample.
+	// A safe default is applied if zero or negative.
+	MaxTagCardinality int
 }
 
 // DefaultCollectorOpts returns sensible defaults.
@@ -262,6 +277,7 @@ func DefaultCollectorOpts() CollectorOpts {
 		QueueSize:         1000,
 		Workers:           2,
 		BackendTimeout:    5 * time.Second,
+		MaxTagCardinality: 32,
 	}
 }
 
@@ -283,6 +299,10 @@ type Collector struct {
 	flushTimer   *time.Timer
 	flushTimerMu sync.Mutex
 
+	// validationErrors counts rejected samples by reason.
+	validationErrors map[string]int64
+	validationMu   sync.Mutex
+
 	// backend is the HTTP endpoint or file path where metrics go.
 	backend string
 
@@ -311,6 +331,7 @@ func NewCollector(opts CollectorOpts, backend string) *Collector {
 		opts:     opts,
 		samples:  make([]MetricSample, 0, opts.QueueSize),
 		backend:  backend,
+		validationErrors: make(map[string]int64),
 	}
 }
 
@@ -319,6 +340,12 @@ func (c *Collector) Start(ctx context.Context) error {
 	if c.opts.Workers <= 0 {
 		c.opts.Workers = 1
 	}
+	if c.opts.MaxTagCardinality <= 0 {
+		c.opts.MaxTagCardinality = DefaultCollectorOpts().MaxTagCardinality
+	}
+	if c.opts.QueueSize <= 0 {
+		c.opts.QueueSize = DefaultCollectorOpts().QueueSize
+	}
 
 	c.wg.Add(c.opts.Workers)
 	for i := 0; i < c.opts.Workers; i++ {
@@ -343,6 +370,24 @@ func (c *Collector) Start(ctx context.Context) error {
 	return nil
 }
 
+// validateSample checks whether the sample conforms to configured limits.
+// It returns a non-nil error if the sample should be rejected.
+func (c *Collector) validateSample(sample *MetricSample) error {
+	if sample == nil {
+		return errors.New("nil sample")
+	}
+	if len(sample.Tags) > c.opts.MaxTagCardinality {
+		return fmt.Errorf("%w: %d tags exceeds limit of %d", ErrExcessiveTagCardinality, len(sample.Tags), c.opts.MaxTagCardinality)
+	}
+	return nil
+
+}
+
+func (c *Collector) recordValidationError(reason string) {
+	c.validationMu.Lock()
+	defer c.validationMu.Unlock()
+	c.validationErrors[reason]++
+}
+
 // Collect ingests a single metric sample.  It is safe for concurrent use.
 func (c *Collector) Collect(sample MetricSample) error {
 	c.mu.Lock()
@@ -351,6 +396,14 @@ func (c *Collector) Collect(sample MetricSample) error {
 	if c.closed {
 		return fmt.Errorf("collector is closed")
 	}
+
+	if err := c.validateSample(&sample); err != nil {
+		if errors.Is(err, ErrExcessiveTagCardinality) {
+			c.recordValidationError("excessive_tag_cardinality")
+		}
+		return err
+	}
+
 	if len(c.samples) >= c.opts.QueueSize {
 		// Drop oldest to make room (circular buffer behavior).
 		c.samples = c.samples[1:]
@@ -360,6 +413,20 @@ func (c *Collector) Collect(sample MetricSample) error {
 	return nil
 }
 
+// ValidationErrors returns a snapshot of validation error counters.
+func (c *Collector) ValidationErrors() map[string]int64 {
+	c.validationMu.Lock()
+	defer c.validationMu.Unlock()
+	out := make(map[string]int64, len(c.validationErrors))
+	for k, v := range c.validationErrors {
+		out[k] = v
+	}
+	return out
+}
+
 // Flush forces a write of all queued samples to the backend.
 func (c *Collector) Flush() error {
 	c.mu.Lock()
@@ -556,6 +623,7 @@ func (c *Collector) worker() {
 // Report generates a human-readable summary of collected metrics.
 func (c *Collector) Report() string {
 	c.mu.Lock()
+	c.validationMu.Lock()
 	var buf strings.Builder
 	buf.WriteString("=== Analytics Collector Report ===\n")
 	buf.WriteString(fmt.Sprintf("Backend: %s\n", c.backend))
@@ -563,8 +631,15 @@ func (c *Collector) Report() string {
 	buf.WriteString(fmt.Sprintf("Workers: %d\n", c.opts.Workers))
 	buf.WriteString(fmt.Sprintf("QueueSize: %d\n", c.opts.QueueSize))
 	buf.WriteString(fmt.Sprintf("FlushInterval: %v\n", c.opts.FlushInterval))
+	buf.Write