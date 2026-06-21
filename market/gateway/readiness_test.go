package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessStates(t *testing.T) {
	g := NewGateway(GatewayConfig{})

	// Test 1: healthy gateway returns ready
	req := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	rr := httptest.NewRecorder()
	g.handleReadiness().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rr.Code)
	}
	var resp map[string]string
	json.Unmarshal(rr.Body.Bytes(), &resp)
	if resp["status"] != "ready" {
		t.Errorf("expected ready, got %s", resp["status"])
	}

	// Test 2: draining returns draining
	g.SetDraining(true)
	req = httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	rr = httptest.NewRecorder()
	g.handleReadiness().ServeHTTP(rr, req)
	if rr.Code != http.StatusServiceUnavailable {
		t.Errorf("expected 503, got %d", rr.Code)
	}
	json.Unmarshal(rr.Body.Bytes(), &resp)
	if resp["status"] != "draining" {
		t.Errorf("expected draining, got %s", resp["status"])
	}

	// Test 3: clearing drain restores ready
	g.SetDraining(false)
	req = httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	rr = httptest.NewRecorder()
	g.handleReadiness().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("expected 200 after drain clear, got %d", rr.Code)
	}
}

func TestDrainEndpoint(t *testing.T) {
	g := NewGateway(GatewayConfig{})

	// GET /health/drain
	req := httptest.NewRequest(http.MethodGet, "/health/drain", nil)
	rr := httptest.NewRecorder()
	g.handleDrain().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("GET expected 200, got %d", rr.Code)
	}

	// POST /health/drain
	req = httptest.NewRequest(http.MethodPost, "/health/drain", nil)
	rr = httptest.NewRecorder()
	g.handleDrain().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("POST expected 200, got %d", rr.Code)
	}

	// DELETE /health/drain
	g.SetDraining(true)
	req = httptest.NewRequest(http.MethodDelete, "/health/drain", nil)
	rr = httptest.NewRecorder()
	g.handleDrain().ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("DELETE expected 200, got %d", rr.Code)
	}

	// PUT returns 405
	req = httptest.NewRequest(http.MethodPut, "/health/drain", nil)
	rr = httptest.NewRecorder()
	g.handleDrain().ServeHTTP(rr, req)
	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("PUT expected 405, got %d", rr.Code)
	}
}
