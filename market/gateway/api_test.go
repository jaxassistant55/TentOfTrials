package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestGatewayReadinessAndDraining(t *testing.T) {
	config := GatewayConfig{}
	g := NewGateway(config)

	// Helper to check readiness
	checkReadiness := func() (int, string) {
		req := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
		rec := httptest.NewRecorder()
		g.mux.ServeHTTP(rec, req)

		var response map[string]string
		if err := json.NewDecoder(rec.Body).Decode(&response); err != nil {
			t.Fatalf("failed to decode response: %v", err)
		}
		return rec.Code, response["status"]
	}

	// 1. Initial state (ready)
	code, status := checkReadiness()
	if code != http.StatusOK || status != "ready" {
		t.Errorf("Expected 200/ready, got %d/%s", code, status)
	}

	// 2. Transition to draining
	g.SetDraining(true)
	code, status = checkReadiness()
	if code != http.StatusServiceUnavailable || status != "draining" {
		t.Errorf("Expected 503/draining, got %d/%s", code, status)
	}

	// 3. Normal healthy behavior is preserved despite draining (e.g. Health() is independent)
	if !g.Health() {
		t.Errorf("Expected Health() to still be true while draining")
	}

	// 4. Transition back to ready
	g.SetDraining(false)
	code, status = checkReadiness()
	if code != http.StatusOK || status != "ready" {
		t.Errorf("Expected 200/ready, got %d/%s", code, status)
	}

	// 5. Test not healthy behavior
	g.health.Store(false)
	code, status = checkReadiness()
	if code != http.StatusServiceUnavailable || status != "not ready" {
		t.Errorf("Expected 503/not ready, got %d/%s", code, status)
	}
}

func TestAdminDrainEndpoint(t *testing.T) {
	config := GatewayConfig{}
	g := NewGateway(config)

	req := httptest.NewRequest(http.MethodPost, "/admin/drain", nil)
	rec := httptest.NewRecorder()
	g.mux.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", rec.Code)
	}

	var response map[string]string
	if err := json.NewDecoder(rec.Body).Decode(&response); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if response["status"] != "draining" {
		t.Errorf("expected status 'draining', got %s", response["status"])
	}
	if !g.IsDraining() {
		t.Errorf("expected gateway to be in draining state")
	}
}
