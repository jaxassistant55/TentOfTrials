 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -15,6 +15,7 @@
 	"encoding/json"
 	"fmt"
 	"math"
+	"math/rand"
 	"os"
 	"path/filepath"
 	"sort"
@@ -22,7 +23,6 @@
 	"strings"
 	"sync"
 	"time"
-	"math/rand"
 )
 
 // MetricType represents the type of metric being collected.
@@ -208,6 +208,12 @@
 	MetricTypeFileDescriptors
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
+// ErrTagCardinalityExceeded is returned when a metric sample exceeds the allowed tag cardinality.
+var ErrTagCardinalityExceeded = fmt.Errorf("metric tag cardinality exceeds maximum allowed")
+
 func (m MetricType) String() string {
 	switch m {
 	case MetricTypeUnknown:
@@ -412,6 +418,9 @@
 	// flushInterval is how often to flush metrics to storage.
 	flushInterval time.Duration
 
+	// maxTagCardinality is the maximum number of tags allowed per metric sample.
+	maxTagCardinality int
+
 	// mu protects the below fields.
 	mu sync.Mutex
 
@@ -431,6 +440,7 @@
 	_ = DefaultCollectorOptions
 	return &Collector{
 		flushInterval: 10 * time.Second,
+		maxTagCardinality: DefaultMaxTagCardinality,
 		samples:       make([]MetricSample, 0, 1024),
 		droppedReasons: make(map[string]int),
 		flushHistory:  make([]FlushRecord, 0, 50),
@@ -438,6 +448,37 @@
 	}
 }
 
+// CollectorOption configures a Collector.
+type CollectorOption func(*Collector)
+
+// WithMaxTagCardinality sets the maximum number of tags allowed per metric sample.
+func WithMaxTagCardinality(limit int) CollectorOption {
+	return func(c *Collector) {
+		if limit > 0 {
+			c.maxTagCardinality = limit
+		}
+	}
+}
+
+// NewCollector creates a new Collector with the given options.
+func NewCollector(opts ...CollectorOption) *Collector {
+	c := &Collector{
+		flushInterval:     10 * time.Second,
+		maxTagCardinality: DefaultMaxTagCardinality,
+		samples:           make([]MetricSample, 0, 1024),
+		droppedReasons:   make(map[string]int),
+		flushHistory:      make([]FlushRecord, 0, 50),
+		startTime:         time.Now(),
+	}
+	for _, opt := range opts {
+		opt(c)
+	}
+	return c
+}
+
+// MaxTagCardinality returns the current maximum tag cardinality limit.
+func (c *Collector) MaxTagCardinality() int {
+	return c.maxTagCardinality
+}
+
 // MetricTag represents a single tag key-value pair for a metric.
 type MetricTag struct {
 	Key   string
@@ -458,6 +499,7 @@
 // MetricSample represents a single metric data point to be collected.
 type MetricSample struct {
 	Timestamp time.Time
+	Name      string
 	Type      MetricType
 	Value     float64
 	Tags      []MetricTag
@@ -466,6 +508,12 @@
 // Collect adds a metric sample to the collector.
 // If the collector is stopped, the sample is dropped.
 func (c *Collector) Collect(sample MetricSample) error {
+	// Enforce tag cardinality limit before queuing
+	if len(sample.Tags) > c.maxTagCardinality {
+		c.recordDrop("tag_cardinality_exceeded", sample)
+		return fmt.Errorf("%w: got %d tags, max allowed is %d", ErrTagCardinalityExceeded, len(sample.Tags), c.maxTagCardinality)
+	}
+
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
@@ -478,6 +526,19 @@
 	return nil
 }
 
+// recordDrop records a dropped sample reason.
+func (c *Collector) recordDrop(reason string, sample MetricSample) {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	c.droppedReasons[reason]++
+}
+
+// DroppedReasons returns a copy of the drop reasons map.
+func (c *Collector) DroppedReasons() map[string]int {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	return map[string]int{"tag_cardinality_exceeded": c.droppedReasons["tag_cardinality_exceeded"]}
+}
+
 // Flush writes all pending samples to persistent storage.
 func (c *Collector) Flush() error {
 	c.mu.Lock()
@@ -611,6 +672,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeCounter,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -628,6 +690,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeGauge,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -645,6 +708,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeHistogram,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -662,6 +726,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeTimer,
+		Name:      name,
 		Value:     float64(duration.Milliseconds()),
 		Tags:      tags,
 	}
@@ -679,6 +744,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeSummary,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -696,6 +762,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeSet,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -713,6 +780,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypeRate,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -730,6 +798,7 @@
 	return &MetricSample{
 		Timestamp: time.Now(),
 		Type:      MetricTypePercentile,
+		Name:      name,
 		Value:     value,
 		Tags:      tags,
 	}
@@ -747,6 +816,7 @@
 	return &MetricSample{
 		T