// Package readiness provides a readiness state helper for the market gateway.
//
// The gateway can be in one of two readiness states: Ready or Draining.
// When draining, the /health/ready endpoint returns a non-ready response
// so load balancers and Kubernetes stop routing traffic, while the process
// remains alive to finish in-flight work.
package gateway

import (
	"sync"
)

// ReadinessState represents the gateway's readiness state.
type ReadinessState int

const (
	// Ready means the gateway accepts traffic normally.
	Ready ReadinessState = iota
	// Draining means the gateway is shutting down; do not send new traffic.
	Draining
)

// String returns a human-readable label for the state.
func (s ReadinessState) String() string {
	switch s {
	case Ready:
		return "ready"
	case Draining:
		return "draining"
	default:
		return "unknown"
	}
}

// ReadinessProbe tracks whether the gateway is ready or draining.
// It is safe for concurrent use.
type ReadinessProbe struct {
	mu    sync.RWMutex
	state ReadinessState
}

// NewReadinessProbe returns a probe in the Ready state.
func NewReadinessProbe() *ReadinessProbe {
	return &ReadinessProbe{state: Ready}
}

// State returns the current readiness state.
func (p *ReadinessProbe) State() ReadinessState {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.state
}

// IsReady reports whether the gateway is in the Ready state.
func (p *ReadinessProbe) IsReady() bool {
	return p.State() == Ready
}

// SetDraining transitions the gateway to the Draining state.
// Returns the previous state.
func (p *ReadinessProbe) SetDraining() ReadinessState {
	p.mu.Lock()
	defer p.mu.Unlock()
	prev := p.state
	p.state = Draining
	return prev
}

// SetReady transitions the gateway back to the Ready state.
// Returns the previous state.  Useful in tests or admin rollback.
func (p *ReadinessProbe) SetReady() ReadinessState {
	p.mu.Lock()
	defer p.mu.Unlock()
	prev := p.state
	p.state = Ready
	return prev
}
