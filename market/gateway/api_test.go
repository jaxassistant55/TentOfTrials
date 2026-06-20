// Package gateway implements testing for public-facing API gateways.
package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// TestReadinessStates tests the readiness handler and its transition behavior
// between Ready, Draining, and Not Ready states, ensuring correct HTTP status
// codes and JSON responses.
func TestReadinessStates(t *testing.T) {
	// Initialize a dummy config
	config := GatewayConfig{
		Host: "localhost",
		Port: 8080,
	}
	gateway := NewGateway(config)

	// Subtest 1: Verify the default state is ready
	t.Run("DefaultReadyState", func(t *testing.T) {
		if gateway.GetReadinessState() != StateReady {
			t.Errorf("expected initial state to be StateReady, got %v", gateway.GetReadinessState())
		}

		req, err := http.NewRequest("GET", "/health/ready", nil)
		if err != nil {
			t.Fatal(err)
		}
		rr := httptest.NewRecorder()
		handler := gateway.handleReadiness()
		handler.ServeHTTP(rr, req)

		if rr.Code != http.StatusOK {
			t.Errorf("expected status 200, got %v", rr.Code)
		}

		var resp map[string]string
		if err := json.Unmarshal(rr.Body.Bytes(), &resp); err != nil {
			t.Fatal(err)
		}
		if resp["status"] != "ready" {
			t.Errorf("expected status 'ready', got %v", resp["status"])
		}
	})

	// Subtest 2: Set readiness state to StateDraining
	t.Run("StateDrainingStatus", func(t *testing.T) {
		gateway.SetReadinessState(StateDraining)
		if gateway.GetReadinessState() != StateDraining {
			t.Errorf("expected state to be StateDraining, got %v", gateway.GetReadinessState())
		}

		req, err := http.NewRequest("GET", "/health/ready", nil)
		if err != nil {
			t.Fatal(err)
		}
		rr := httptest.NewRecorder()
		handler := gateway.handleReadiness()
		handler.ServeHTTP(rr, req)

		if rr.Code != http.StatusServiceUnavailable {
			t.Errorf("expected status 503, got %v", rr.Code)
		}

		var resp map[string]string
		if err := json.Unmarshal(rr.Body.Bytes(), &resp); err != nil {
			t.Fatal(err)
		}
		if resp["status"] != "draining" {
			t.Errorf("expected status 'draining', got %v", resp["status"])
		}
	})

	// Subtest 3: Set readiness state to StateNotReady
	t.Run("StateNotReadyStatus", func(t *testing.T) {
		gateway.SetReadinessState(StateNotReady)
		if gateway.GetReadinessState() != StateNotReady {
			t.Errorf("expected state to be StateNotReady, got %v", gateway.GetReadinessState())
		}

		req, err := http.NewRequest("GET", "/health/ready", nil)
		if err != nil {
			t.Fatal(err)
		}
		rr := httptest.NewRecorder()
		handler := gateway.handleReadiness()
		handler.ServeHTTP(rr, req)

		if rr.Code != http.StatusServiceUnavailable {
			t.Errorf("expected status 503, got %v", rr.Code)
		}

		var resp map[string]string
		if err := json.Unmarshal(rr.Body.Bytes(), &resp); err != nil {
			t.Fatal(err)
		}
		if resp["status"] != "not ready" {
			t.Errorf("expected status 'not ready', got %v", resp["status"])
		}
	})

	// Subtest 4: Verify liveness check is unaffected by readiness states
	t.Run("LivenessUnchanged", func(t *testing.T) {
		gateway.SetReadinessState(StateDraining)
		req, err := http.NewRequest("GET", "/health/live", nil)
		if err != nil {
			t.Fatal(err)
		}
		rr := httptest.NewRecorder()
		handler := gateway.handleLiveness()
		handler.ServeHTTP(rr, req)

		if rr.Code != http.StatusOK {
			t.Errorf("expected liveness status 200, got %v", rr.Code)
		}

		var resp map[string]string
		if err := json.Unmarshal(rr.Body.Bytes(), &resp); err != nil {
			t.Fatal(err)
		}
		if resp["status"] != "alive" {
			t.Errorf("expected liveness status 'alive', got %v", resp["status"])
		}
	})
}
