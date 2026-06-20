package analytics

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"
)

func TestCollectorStartSetsStartedState(t *testing.T) {
	collector := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	if err := collector.Start(ctx); err != nil {
		t.Fatalf("Start() unexpected error: %v", err)
	}
	if !collector.IsStarted() {
		t.Fatal("collector should report started after Start")
	}
	if !collector.Stats().Started {
		t.Fatal("stats should report started after Start")
	}
	if !collector.Stop() {
		t.Fatal("Stop() should report that a running collector was stopped")
	}
	waitForStopped(t, collector)
}

func TestCollectorStartIsIdempotent(t *testing.T) {
	collector := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	if err := collector.Start(ctx); err != nil {
		t.Fatalf("first Start() unexpected error: %v", err)
	}
	if err := collector.Start(ctx); !errors.Is(err, ErrCollectorAlreadyStarted) {
		t.Fatalf("second Start() error = %v, want ErrCollectorAlreadyStarted", err)
	}

	collector.RecordCounter("boot_metric", 1)
	if !collector.Stop() {
		t.Fatal("Stop() should still stop after a repeated Start attempt")
	}
	waitForStopped(t, collector)
	if collector.Stop() {
		t.Fatal("second Stop() should report no running collector")
	}
}

func TestCollectorConcurrentStartOnlyStartsOnce(t *testing.T) {
	collector := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var wg sync.WaitGroup
	errs := make(chan error, 16)
	for i := 0; i < cap(errs); i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			errs <- collector.Start(ctx)
		}()
	}
	wg.Wait()
	close(errs)

	successes := 0
	alreadyStarted := 0
	for err := range errs {
		switch {
		case err == nil:
			successes++
		case errors.Is(err, ErrCollectorAlreadyStarted):
			alreadyStarted++
		default:
			t.Fatalf("unexpected Start() error: %v", err)
		}
	}

	if successes != 1 {
		t.Fatalf("successful starts = %d, want 1", successes)
	}
	if alreadyStarted != 15 {
		t.Fatalf("already-started errors = %d, want 15", alreadyStarted)
	}
	if !collector.Stop() {
		t.Fatal("Stop() should stop the one running worker")
	}
	waitForStopped(t, collector)
}

func TestCollectorContextCancelStopsWorker(t *testing.T) {
	collector := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())

	if err := collector.Start(ctx); err != nil {
		t.Fatalf("Start() unexpected error: %v", err)
	}
	cancel()

	waitForStopped(t, collector)
	if collector.Stop() {
		t.Fatal("Stop() should report no running collector after context cancellation")
	}
}

func waitForStopped(t *testing.T, collector *Collector) {
	t.Helper()
	deadline := time.Now().Add(500 * time.Millisecond)
	for time.Now().Before(deadline) {
		if !collector.IsStarted() {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatal("collector did not stop before deadline")
}
