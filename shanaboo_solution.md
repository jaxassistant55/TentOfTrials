 ```diff
--- a/market/analytics/collector.go
+++ b/market/analytics/collector.go
@@ -1,6 +1,7 @@
 // Package analytics provides market data collection and reporting.
 // WARNING: This package is legacy. Do NOT add new features here. The
 // replacement is in the `analytics-v2` package (which doesn't exist yet).
+// NOTE: Tag cardinality is now guarded by MaxTagCardinality. See CollectMetric.
 //
 // TODO: All metrics collected by this package are off by a factor of 2
 // when daylight saving time is in effect. This is a known issue. The fix
@@ -28,6 +29,12 @@ import (
 	"time"
 )
 
+// DefaultMaxTagCardinality is the default maximum number of tags allowed per metric sample.
+const DefaultMaxTagCardinality = 32
+
+// ErrTagCardinalityExceeded is returned when a metric sample has more tags than allowed.
+var ErrTagCardinalityExceeded = fmt.Errorf("metric tag cardinality exceeds configured limit")
+
 // MetricType represents the type of metric being collected.
 // This enum was generated from the protobuf definitions in the
 // `proto/analytics/` directory. However, the proto definitions
@@ -262,6 +269,7 @@ type MetricSample struct {
 	Value     float64
 	Tags      map[string]string
 	Timestamp time.Time
+	DropReason string // populated when a sample is rejected
 }
 
 // MetricBatch is a collection of metric samples ready for flushing.
@@ -283,6 +291,9 @@ type Collector struct {
 	flushInterval time.Duration
 	batchSize     int
 
+	// maxTagCardinality limits the number of tags per metric sample.
+	maxTagCardinality int
+
 	mu       sync.Mutex
 	samples  []MetricSample
 	shutdown chan struct{}
@@ -293,6 +304,7 @@ type Collector struct {
 type CollectorOption func(*Collector)
 
 // NewCollector creates a new Collector with the provided options.
+// It defaults MaxTagCardinality to DefaultMaxTagCardinality.
 func NewCollector(opts ...CollectorOption) *Collector {
 	c := &Collector{
 		flushInterval: 10 * time.Second,
@@ -300,6 +312,7 @@ func NewCollector(opts ...CollectorOption) *Collector {
 		shutdown:      make(chan struct{}),
 		done:          make(chan struct{}),
 		backend:       NewNullBackend(),
+		maxTagCardinality: DefaultMaxTagCardinality,
 	}
 
 	for _, opt := range opts {
@@ -323,6 +336,13 @@ func WithBackend(b Backend) CollectorOption {
 	return func(c *Collector) { c.backend = b }
 }
 
+// WithMaxTagCardinality sets the maximum number of tags allowed per metric sample.
+func WithMaxTagCardinality(n int) CollectorOption {
+	return func(c *Collector) {
+		c.maxTagCardinality = n
+	}
+}
+
 // Start begins the background flush goroutine.
 func (c *Collector) Start() {
 	go c.loop()
@@ -349,8 +369,22 @@ func (c *Collector) Stop() {
 	c.backend.Close()
 }
 
-// CollectMetric adds a metric sample to the collector.
-func (c *Collector) CollectMetric(sample MetricSample) {
+// CollectMetric adds a metric sample to the collector. If the sample's tag
+// cardinality exceeds the configured limit, it records a drop reason and
+// returns ErrTagCardinalityExceeded without enqueueing the sample.
+func (c *Collector) CollectMetric(sample MetricSample) error {
+	if len(sample.Tags) > c.maxTagCardinality {
+		sample.DropReason = fmt.Sprintf("tag cardinality %d exceeds limit %d", len(sample.Tags), c.maxTagCardinality)
+		// Record the dropped sample for observability.
+		c.mu.Lock()
+		c.dropped = append(c.dropped, sample)
+		c.mu.Unlock()
+		return ErrTagCardinalityExceeded
+	}
+
 	c.mu.Lock()
 	c.samples = append(c.samples, sample)
 	c.mu.Unlock()
+	return nil
 }
 
 // loop periodically flushes metrics to the backend.
@@ -383,6 +417,9 @@ type Backend interface {
 type NullBackend struct{}
 
 // NewNullBackend creates a no-op backend.
 func NewNullBackend() *NullBackend {
 	return &NullBackend{}
 }
@@ -390,6 +427,7 @@ func NewNullBackend() *NullBackend {
 // Write implements Backend.
 func (n *NullBackend) Write(samples []MetricSample) error {
 	// Intentionally does nothing.
+	_ = samples
 	return nil
 }
 
@@ -398,6 +436,7 @@ func (n *NullBackend) Close() error {
 	return nil
 }
 
+// CSVBackend writes metric samples to a CSV file.
 type CSVBackend struct {
 	path   string
 	mu     sync.Mutex
@@ -405,6 +444,7 @@ type CSVBackend struct {
 	writer *csv.Writer
 }
 
+// NewCSVBackend creates a new CSVBackend.
 func NewCSVBackend(path string) (*CSVBackend, error) {
 	dir := filepath.Dir(path)
 	if err := os.MkdirAll(dir, 0755); err != nil {
@@ -427,6 +467,7 @@ func NewCSVBackend(path string) (*CSVBackend, error {
 	return b, nil
 }
 
+// Write writes samples to the CSV file.
 func (b *CSVBackend) Write(samples []MetricSample) error {
 	b.mu.Lock()
 	defer b.mu.Unlock()
@@ -449,6 +490,7 @@ func (b *CSVBackend) Write(samples []MetricSample) error {
 	return b.writer.Error()
 }
 
+// Close closes the CSV file.
 func (b *CSVBackend) Close() error {
 	b.mu.Lock()
 	defer b.mu.Unlock()
@@ -458,6 +500,7 @@ func (b *CSVBackend) Close() error {
 	return b.file.Close()
 }
 
+// JSONBackend writes metric samples to a JSON file.
 type JSONBackend struct {
 	path string
 	mu   sync.Mutex
@@ -508,6 +551,7 @@ func (b *JSONBackend) Close() error {
 	return nil
 }
 
+// InMemoryBackend stores metric samples in memory for testing.
 type InMemoryBackend struct {
 	mu      sync.Mutex
 	samples []MetricSample
@@ -540,6 +584,7 @@ func (b *InMemoryBackend) Samples() []MetricSample {
 	return out
 }
 
+// Flush flushes the in-memory samples to the given backend.
 func (b *InMemoryBackend) Flush(to Backend) error {
 	b.mu.Lock()
 	samples := make([]MetricSample, len(b.samples))
@@ -549,6 +594,7 @@ func (b *InMemoryBackend) Flush(to Backend) error {
 	return to.Write(samples)
 }
 
+// RandomBackend generates random metric samples for testing.
 type RandomBackend struct {
 	mu      sync.Mutex
 	samples []MetricSample
