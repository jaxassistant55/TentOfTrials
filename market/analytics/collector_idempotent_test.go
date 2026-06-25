package analytics

import (
	"context"
	"errors"
	"sync"
	"testing"
	"time"
)

func TestCollector_SingleStart(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	err := c.Start(ctx)
	if err != nil {
		t.Fatalf("first Start() should succeed, got: %v", err)
	}
	if !c.IsStarted() {
		t.Fatal("IsStarted() should be true after Start()")
	}
}

func TestCollector_RepeatedStart(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// First call should succeed
	if err := c.Start(ctx); err != nil {
		t.Fatalf("first Start() should succeed, got: %v", err)
	}

	// Second call should return ErrAlreadyStarted
	if err := c.Start(ctx); !errors.Is(err, ErrAlreadyStarted) {
		t.Fatalf("second Start() should return ErrAlreadyStarted, got: %v", err)
	}

	// Third call should also return ErrAlreadyStarted
	if err := c.Start(ctx); !errors.Is(err, ErrAlreadyStarted) {
		t.Fatalf("third Start() should return ErrAlreadyStarted, got: %v", err)
	}
}

func TestCollector_StopAfterRepeatedStart(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start the collector
	if err := c.Start(ctx); err != nil {
		t.Fatalf("Start() should succeed, got: %v", err)
	}

	// Try to start again (should fail)
	if err := c.Start(ctx); !errors.Is(err, ErrAlreadyStarted) {
		t.Fatalf("second Start() should return ErrAlreadyStarted, got: %v", err)
	}

	// Stop should work
	c.Stop()
	time.Sleep(50 * time.Millisecond) // give goroutine time to exit

	// After stop, IsStarted should be false
	if c.IsStarted() {
		t.Fatal("IsStarted() should be false after Stop()")
	}
}

func TestCollector_StartAfterStop(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Start and then stop
	if err := c.Start(ctx); err != nil {
		t.Fatalf("first Start() should succeed, got: %v", err)
	}
	c.Stop()
	time.Sleep(50 * time.Millisecond)

	// After stop, we should be able to start again
	if err := c.Start(ctx); err != nil {
		t.Fatalf("Start() after Stop() should succeed, got: %v", err)
	}
	if !c.IsStarted() {
		t.Fatal("IsStarted() should be true after restart")
	}
}

func TestCollector_StartContextCancel(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())

	if err := c.Start(ctx); err != nil {
		t.Fatalf("Start() should succeed, got: %v", err)
	}

	// Cancel the context
	cancel()
	time.Sleep(50 * time.Millisecond)

	// After context cancellation, IsStarted should be false
	if c.IsStarted() {
		t.Fatal("IsStarted() should be false after context cancellation")
	}
}

func TestCollector_ConcurrentStart(t *testing.T) {
	c := NewCollector()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	var wg sync.WaitGroup
	errs := make([]error, 10)

	// Try to start from 10 goroutines simultaneously
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			errs[idx] = c.Start(ctx)
		}(i)
	}
	wg.Wait()

	// Exactly one goroutine should have succeeded (nil error)
	successCount := 0
	for _, err := range errs {
		if err == nil {
			successCount++
		}
	}
	if successCount != 1 {
		t.Fatalf("expected exactly 1 successful Start(), got %d", successCount)
	}

	// The rest should have gotten ErrAlreadyStarted
	errAlreadyCount := 0
	for _, err := range errs {
		if errors.Is(err, ErrAlreadyStarted) {
			errAlreadyCount++
		}
	}
	if errAlreadyCount != 9 {
		t.Fatalf("expected 9 ErrAlreadyStarted errors, got %d", errAlreadyCount)
	}
}

func TestCollector_IsStartedBeforeStart(t *testing.T) {
	c := NewCollector()
	if c.IsStarted() {
		t.Fatal("IsStarted() should be false before Start() is called")
	}
}
