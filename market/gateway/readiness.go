package gateway

import "sync/atomic"

type ReadinessState string

const (
	ReadinessReady    ReadinessState = "ready"
	ReadinessDraining ReadinessState = "draining"
)

type ReadinessStateStore struct {
	state atomic.Value
}

func NewReadinessStateStore() *ReadinessStateStore {
	store := &ReadinessStateStore{}
	store.MarkReady()
	return store
}

func (s *ReadinessStateStore) MarkReady() {
	s.state.Store(ReadinessReady)
}

func (s *ReadinessStateStore) MarkDraining() {
	s.state.Store(ReadinessDraining)
}

func (s *ReadinessStateStore) State() ReadinessState {
	state, ok := s.state.Load().(ReadinessState)
	if !ok {
		return ReadinessReady
	}
	return state
}
