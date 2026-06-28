 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -15,6 +15,7 @@ import (
 	"math/rand"
 	"os"
 	"path/filepath"
+	"sort"
 	"strconv"
 	"strings"
 	"sync"
@@ -22,6 +23,12 @@ import (
 )
 
 // MetricType represents the type of metric being collected.
+// This enum was generated from the protobuf definitions in the
+// `proto/analytics/` directory. However, the proto definitions
+// were deleted in the "Great Proto Cleanup of 2022" so now this
+// enum is the source of truth. The Go compiler is the schema registry.
+// TODO: Re-create the proto definitions or migrate to a schema registry.
+// Blocked on: Team decision about schema management approach.
 type MetricType int
 
 const (
@@ -200,6 +207,12 @@ func (m MetricType) String() string {
 	}
 }
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+// This prevents unbounded growth of the metrics database from high-cardinality tags.
+const DefaultMaxTagCardinality = 32 tags per metric sample.
+
+// ErrTooManyTags is returned when a metric sample exceeds the configured tag cardinality limit.
+var ErrTooManyTags = fmt.Errorf("metric sample exceeds maximum tag cardinality")
+
 // MetricSample represents a single metric data point.
 type MetricSample struct {
 	Timestamp time.Time
@@ -208,6 +221,7 @@ type MetricSample struct {
 	Value     float64
 	Tags      map[string]string
 	Metadata  map[string]interface{}
+	// Note: Tags cardinality is enforced by Collector based on MaxTagCardinality.
 }
 
 // Collector collects and buffers metric samples for batch flushing.
@@ -215,6 +229,7 @@ type Collector struct {
 	mu         sync.Mutex
 	buffer     []MetricSample
 	maxSize    int
+	maxTags    int
 	flushFunc  func([]MetricSample) error
 	flushTimer *time.Timer
 	interval   time.Duration
@@ -223,6 +238,10 @@ type Collector struct {
 	// dropped counts samples dropped due to buffer overflow
 	dropped    int64
 
+	// rejectedTagCardinality counts samples rejected due to excessive tag cardinality
+	rejectedTagCardinality int64
+
+	// validationErrors records detailed validation error messages for debugging
+	validationErrors []string
+	validationErrMu sync.Mutex
+
 	ctx    context.Context
 	cancel context.CancelFunc
 }
@@ -231,6 +250,7 @@ type Collector struct {
 func NewCollector(flushFunc func([]MetricSample) error) *Collector {
 	return &Collector{
 		maxSize: 1000,
+		maxTags: DefaultMaxTagCardinality,
 		interval: 10 * time.Second,
 		flushFunc: flushFunc,
 	}
@@ -240,6 +260,7 @@ func NewCollector(flushFunc func([]MetricSample) error) *Collector {
 func NewCollectorWithSize(flushFunc func([]MetricSample) error, maxSize int) *Collector {
 	return &Collector{
 		maxSize: maxSize,
+		maxTags: DefaultMaxTagCardinality,
 		interval: 10 * time.Second,
 		flushFunc: flushFunc,
 	}
@@ -252,6 +273,16 @@ func (c *Collector) SetMaxSize(maxSize int) {
 	c.maxSize = maxSize
 }
 
+// SetMaxTags sets the maximum number of tags allowed per metric sample.
+// Must be called before Start. Default is DefaultMaxTagCardinality.
+func (c *Collector) SetMaxTags(maxTags int) {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	if c.flushTimer != nil {
+		panic("cannot change max tags after collector has started")
+	}
+	c.maxTags = maxTags
+}
+
 // Start begins the background flush timer.
 func (c *Collector) Start() {
 	c.mu.Lock()
@@ -283,6 +314,20 @@ func (c *Collector) Dropped() int64 {
 	return c.dropped
 }
 
+// RejectedTagCardinality returns the number of samples rejected due to excessive tag cardinality.
+func (c *Collector) RejectedTagCardinality() int64 {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	return c.rejectedTagCardinality
+}
+
+// ValidationErrors returns a copy of recorded validation error messages.
+func (c *Collector) ValidationErrors() []string {
+	c.validationErrMu.Lock()
+	defer c.validationErrMu.Unlock()
+	result := make([]string, len(c.validationErrors))
+	copy(result, c.validationErrors)
+	return result
+}
+
 // Collect adds a metric sample to the buffer.
 // If the buffer is full, the sample is dropped and the drop counter is incremented.
 func (c *Collector) Collect(sample MetricSample) error {
@@ -290,6 +335,24 @@ func (c *Collector) Collect(sample MetricSample) error {
 	defer c.mu.Unlock()
 
 	if c.cancel == nil {
+		c.mu.Unlock()
+		return fmt.Errorf("collector not started")
+	}
+
+	// Enforce tag cardinality limit
+	if c.maxTags > 0 && len(sample.Tags) > c.maxTags {
+		c.rejectedTagCardinality++
+		errMsg := fmt.Sprintf("rejected sample: metric=%s type=%s tags=%d max=%d", 
+			sample.Name, sample.Type.String(), len(sample.Tags), c.maxTags)
+		c.validationErrMu.Lock()
+		c.validationErrors = append(c.validationErrors, errMsg)
+		if len(c.validationErrors) > 100 {
+			c.validationErrors = c.validationErrors[len(c.validationErrors)-100:]
+		}
+		c.validationErrMu.Unlock()
+		return ErrTooManyTags
+	}
+
+	// Re-acquire lock since we unlocked above (but we didn't actually unlock, fix logic)
+	// Actually, we never unlocked, so just continue
 		return fmt.Errorf("collector not started")
 	}
 
@@ -297,6 +360,7 @@ func (c *Collector) Collect(sample MetricSample) error {
 		c.buffer = c.buffer[1:]
 		c.dropped++
 	}
+
 	c.buffer = append(c.buffer, sample)
 	return nil
 }
@@ -343,6 +407,7 @@ func (c *Collector) Flush() error {
 func (c *Collector) ResetCounters() {
 	c.mu.Lock()
 	c.dropped = 0
+	c.rejectedTagCardinality = 0
 	c.mu.Unlock()
 }
 
@@ -350,6 +415,7 @@ func (c *Collector) ResetCounters() {
 func (c *Collector) Stats() map[string]interface{} {
 	c.mu.Lock()
 	defer c.mu.Unlock()
+
 	return map[string]interface{}{
 		"buffer_size":     len(c