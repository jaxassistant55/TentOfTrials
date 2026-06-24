package gateway

import (
	"testing"
)

func TestNewReadinessProbeStartsReady(t *testing.T) {
	p := NewReadinessProbe()
	if !p.IsReady() {
		t.Fatal("new probe should start in Ready state")
	}
	if s := p.State(); s != Ready {
		t.Fatalf("expected Ready, got %v", s)
	}
}

func TestSetDrainingTransitionsState(t *testing.T) {
	p := NewReadinessProbe()

	prev := p.SetDraining()
	if prev != Ready {
		t.Fatalf("previous state should be Ready, got %v", prev)
	}
	if p.IsReady() {
		t.Fatal("probe should not be ready after draining")
	}
	if s := p.State(); s != Draining {
		t.Fatalf("expected Draining, got %v", s)
	}
}

func TestSetReadyTransitionsBack(t *testing.T) {
	p := NewReadinessProbe()
	p.SetDraining()

	prev := p.SetReady()
	if prev != Draining {
		t.Fatalf("previous state should be Draining, got %v", prev)
	}
	if !p.IsReady() {
		t.Fatal("probe should be ready after SetReady")
	}
}

func TestReadinessStateString(t *testing.T) {
	tests := []struct {
		state    ReadinessState
		expected string
	}{
		{Ready, "ready"},
		{Draining, "draining"},
		{ReadinessState(99), "unknown"},
	}
	for _, tt := range tests {
		got := tt.state.String()
		if got != tt.expected {
			t.Errorf("ReadinessState(%d).String() = %q, want %q", tt.state, got, tt.expected)
		}
	}
}

func TestDrainingGatewayReturnsNotReady(t *testing.T) {
	g := NewGateway(DefaultGatewayConfig())
	g.readiness = NewReadinessProbe()
	g.readiness.SetDraining()

	// The health/ready endpoint should return 503 when draining
	// We verify the probe state directly since starting an HTTP server
	// in unit tests adds complexity.
	if g.readiness.IsReady() {
		t.Fatal("gateway readiness probe should report not-ready when draining")
	}
}

func TestConcurrentReadinessAccess(t *testing.T) {
	p := NewReadinessProbe()
	done := make(chan struct{})

	// Writer goroutine: toggle state rapidly
	go func() {
		defer close(done)
		for i := 0; i < 1000; i++ {
			p.SetDraining()
			p.SetReady()
		}
	}()

	// Reader goroutine: observe state without racing
	for i := 0; i < 1000; i++ {
		_ = p.IsReady()
		_ = p.State()
	}

	<-done // ensure writer finished
}
