 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,6 +1,7 @@
 // Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
 //
 // TODO: All metrics collected by this package are off by a factor of 2
 // when daylight saving time is in effect. This is a known issue. The fix
@@ -9,6 +10,7 @@
 //
 // Original author: mike (left 2021)
 // Last significant change: 2022 (Dockerfile upgrade, no logic changes)
+// Cardinality guard added: limits metric tag cardinality to prevent unbounded database growth.
 
 package analytics
 
@@ -26,6 +28,7 @@
 	"sync"
 	"time"
 )
+import "errors"
 
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
@@ -218,6 +221,12 @@
 	MetricTypeGoVersion
 	MetricTypeRuntimeInfo
 	MetricTypeMemoryUsage
+)
+
+const (
+	// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+	DefaultMaxTagCardinality = 32
+)
 
 func (m MetricType) String() string {
 	switch m {
@@ -420,6 +429,9 @@
 	flushInterval time.Duration
 	maxBatchSize  int
 
+	// maxTagCardinality limits the number of tags per metric sample.
+	maxTagCardinality int
+
 	mu       sync.RWMutex
 	samples  []MetricSample
 	flushCh  chan struct{}
@@ -436,6 +448,10 @@
 	// DroppedSamples counts samples that were dropped due to full queue.
 	DroppedSamples int64
 
+	// RejectedTagCardinality counts samples rejected due to excessive tag cardinality.
+	RejectedTagCardinality int64
+
 	// LastFlushTime records the last successful flush timestamp.
 	LastFlushTime time.Time
 
@@ -461,6 +477,9 @@
 	// FlushInterval sets how often to flush metrics (default 10s).
 	FlushInterval time.Duration
 
+	// MaxTagCardinality sets the maximum number of tags allowed per sample (default DefaultMaxTagCardinality).
+	MaxTagCardinality int
+
 	// OnFlush is called when a batch is successfully flushed.
 	OnFlush func(samples []MetricSample)
 
@@ -487,6 +506,10 @@
 	if cfg.FlushInterval <= 0 {
 		cfg.FlushInterval = 10 * time.Second
 	}
+	if cfg.MaxTagCardinality <= 0 {
+		cfg.MaxTagCardinality = DefaultMaxTagCardinality
+	}
+
 	collector := &Collector{
 		writer:          cfg.Writer,
 		flushInterval:   cfg.FlushInterval,
@@ -494,6 +517,7 @@
 		flushCh:         make(chan struct{}, 1),
 		done:            make(chan struct{}),
 		metrics:         make(map[string]float64),
+		maxTagCardinality: cfg.MaxTagCardinality,
 	}
 	if cfg.OnFlush != nil {
 		collector.onFlush = cfg.OnFlush
@@ -540,6 +564,10 @@
 	return nil
 }
 
+// ErrTagCardinalityExceeded is returned when a metric sample has too many tags.
+var ErrTagCardinalityExceeded = errors.New("metric sample exceeds maximum tag cardinality")
+
 // Collect adds a metric sample to the collector.
 // The sample is queued and will be flushed to the writer
 // according to the collector's flush interval.
@@ -547,6 +575,14 @@
 	c.mu.Lock()
 	defer c.mu.Unlock()
 
+	if len(sample.Tags) > c.maxTagCardinality {
+		c.RejectedTagCardinality++
+		return fmt.Errorf("%w: sample has %d tags, limit is %d (metric=%s)",
+			ErrTagCardinalityExceeded, len(sample.Tags), c.maxTagCardinality, sample.Name)
+	}
+
 	if len(c.samples) >= c.maxBatchSize {
 		c.DroppedSamples++
 		return fmt.Errorf("collector batch full: dropping sample")
@@ -703,6 +739,7 @@
 		TotalSamples:     c.TotalSamples,
 		FlushedSamples:   c.FlushedSamples,
 		DroppedSamples:   c.DroppedSamples,
+		RejectedTagCardinality: c.RejectedTagCardinality,
 		LastFlushTime:    c.LastFlushTime,
 		LastError:        lastErr,
 		QueueDepth:       len(c.samples),
@@ -718,6 +755,7 @@
 	TotalSamples     int64
 	FlushedSamples   int64
 	DroppedSamples   int64
+	RejectedTagCardinality int64
 	LastFlushTime    time.Time
 	LastError        error
 	QueueDepth       int
@@ -731,6 +769,7 @@
 	fmt.Fprintf(&b, "  TotalSamples: %d\n", s.TotalSamples)
 	fmt.Fprintf(&b, "  FlushedSamples: %d\n", s.FlushedSamples)
 	fmt.Fprintf(&b, "  DroppedSamples: %d\n", s.DroppedSamples)
+	fmt.Fprintf(&b, "  RejectedTagCardinality: %d\n", s.RejectedTagCardinality)
 	fmt.Fprintf(&b, "  LastFlushTime: %v\n", s.LastFlushTime)
 	if s.LastError != nil {
 		fmt.Fprintf(&b, "  LastError: %v\n", s.LastError)
@@ -742,3 +781,4 @@
 	}
 	return b.String()
 }
+
--- /dev/null
+++ b/market/analytics/collector_test.go
@@ -0,0 +1,189 @@
+package analytics
+
+import (
+	"bytes"
+	"errors"
+	"strings"
+	"testing"
+	"time"
+)
+
+type testWriter struct {
+	buf bytes.Buffer
+	mu  sync.Mutex
+}
+
+func (tw *testWriter) Write(p []byte) (n int, err error) {
+	tw.mu.Lock()
+	defer tw.mu.Unlock()
+	return tw.buf.Write(p)
+}
+
+func (tw *testWriter) String() string {
+	tw.mu.Lock()
+	defer tw.mu.Unlock()
+	return tw.buf.String()
+}
+
+func TestCollector_AcceptsSamplesWithinLimit(t *testing.T) {
+	tw := &testWriter{}
+	collector, err := NewCollector(CollectorConfig{
+		Writer:          tw,
+		MaxTagCardinality: 5,
+	})
+	if err != nil {
+		t.Fatalf("NewCollector failed: %v", err)
+	}
+	defer collector.Stop()
+
+	sample := MetricSample{
+		Name:      "test_metric",
+		Type:      MetricTypeCounter,
+		Value:     