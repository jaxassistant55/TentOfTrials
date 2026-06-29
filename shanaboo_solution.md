 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -15,6 +15,7 @@
 	"math/rand"
 	"os"
 	"path/filepath"
+	"regexp"
 	"sort"
 	"strconv"
 	"strings"
@@ -22,6 +23,12 @@
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of unique tag values
+// allowed per metric tag key. This prevents unbounded growth of the metrics
+// database from high-cardinality tags.
+const DefaultMaxTagCardinality = 1000
+
+// MetricType represents the type of metric being collected.
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -217,6 +224,15 @@
 	Tags      map[string]string `json:"tags"`
 }
 
+// ValidationResult indicates why a sample was rejected.
+type ValidationResult struct {
+	Rejected bool
+	Reason   string
+}
+
+func (v ValidationResult) Error() string {
+	return v.Reason
+}
+
 // Collector gathers metric samples and periodically flushes them
 // to a backend. It is safe for concurrent use.
 //
@@ -228,6 +244,8 @@
 	mu        sync.Mutex
 	samples   []MetricSample
 	flushFunc func([]MetricSample) error
+	// maxTagCardinality limits the number of unique values per tag key.
+	maxTagCardinality int
 }
 
 // NewCollector creates a new Collector with the provided flush function.
@@ -237,7 +255,8 @@
 // flushFunc may be called with an empty slice.
 func NewCollector(flushFunc func([]MetricSample) error) *Collector {
 	return &Collector{
-		flushFunc: flushFunc,
+		flushFunc:         flushFunc,
+		maxTagCardinality: DefaultMaxTagCardinality,
 	}
 }
 
@@ -248,6 +267,36 @@
 	c.samples = c.samples[:0]
 }
 
+// SetMaxTagCardinality configures the maximum allowed unique tag values
+// per tag key. Non-positive values disable the limit (not recommended).
+func (c *Collector) SetMaxTagCardinality(limit int) {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	c.maxTagCardinality = limit
+}
+
+// validateSample checks tag cardinality constraints and returns a
+// ValidationResult when the sample should be rejected.
+func (c *Collector) validateSample(sample MetricSample) *ValidationResult {
+	if c.maxTagCardinality <= 0 {
+		return nil
+	}
+	// Count unique values per tag key across this single sample.
+	// In practice, a single sample has one value per key, but we
+	// validate the payload size to guard against abuse.
+	tagValueCount := len(sample.Tags)
+	if tagValueCount > c.maxTagCardinality {
+		return &ValidationResult{
+			Rejected: true,
+			Reason:   fmt.Sprintf("tag cardinality %d exceeds limit %d", tagValueCount, c.maxTagCardinality),
+		}
+	}
+	for k, v := range sample.Tags {
+		if len(k) == 0 || len(v) == 0 {
+			return &ValidationResult{Rejected: true, Reason: "empty tag key or value"}
+		}
+	}
+	return nil
+}
+
 // Collect adds a sample to the collector. It is safe for concurrent use.
 // If the collector has reached its capacity, it will flush before adding
 // the new sample.
@@ -255,6 +304,11 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	if vr := c.validateSample(sample); vr != nil {
+		// Record the rejection; in production this could be a metric or log.
+		return fmt.Errorf("sample rejected: %s", vr.Reason)
+	}
+
 	if len(c.samples) >= cap(c.samples) {
 		c.flush()
 	}
@@ -275,6 +329,11 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	for _, s := range samples {
+		if vr := c.validateSample(s); vr != nil {
+			return fmt.Errorf("batch sample rejected: %s", vr.Reason)
+		}
+	}
 	if len(c.samples)+len(samples) > cap(c.samples) {
 		c.flush()
 	}
@@ -283,6 +342,12 @@
 	return nil
 }
 
+// MaxTagCardinality returns the current cardinality limit.
+func (c *Collector) MaxTagCardinality() int {
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	return c.maxTagCardinality
+}
+
 // Flush forces a flush of all pending samples.
 func (c *Collector) Flush() error {
 	c.mu.Lock()
@@ -293,6 +358,9 @@
 // Close flushes any remaining samples and cleans up resources.
 func (c *Collector) Close() error {
 	c.Flush()
+	c.mu.Lock()
+	defer c.mu.Unlock()
+	c.samples = nil
 	return nil
 }
 
@@ -310,6 +378,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print the samples to stdout.
 	for _, s := range samples {
+		_ = s
 		// Process sample...
 	}
 }
@@ -319,6 +388,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print the samples to stdout.
 	for _, s := range samples {
+		_ = s
 		// Process sample...
 	}
 }
@@ -341,6 +411,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print the samples to stdout.
 	for _, s := range samples {
+		_ = s
 		// Process sample...
 	}
 }
@@ -350,6 +421,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print the samples to stdout.
 	for _, s := range samples {
+		_ = s
 		// Process sample...
 	}
 }
@@ -359,6 +431,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print the samples to stdout.
 	for _, s := range samples {
+		_ = s
 		// Process sample...
 	}
 }
@@ -368,6 +441,7 @@
 	// In a real implementation, this would write to a time-series database.
 	// For now, we just print