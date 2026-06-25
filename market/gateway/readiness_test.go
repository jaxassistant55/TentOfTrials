package gateway

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessEndpointReturnsReadyWhenHealthy(t *testing.T) {
	gw := &Gateway{}
	gw.health.Store(true)

	handler := gw.handleReadiness()
	req := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%s", rec.Code, http.StatusOK, rec.Body.String())
	}

	var body map[string]string
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if body["status"] != "ready" {
		t.Fatalf("status = %q, want %q", body["status"], "ready")
	}
}

func TestReadinessEndpointReturnsNotReadyWhenUnhealthy(t *testing.T) {
	gw := &Gateway{}
	gw.health.Store(false)

	handler := gw.handleReadiness()
	req := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%s", rec.Code, http.StatusServiceUnavailable, rec.Body.String())
	}

	var body map[string]string
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if body["status"] != "not ready" {
		t.Fatalf("status = %q, want %q", body["status"], "not ready")
	}
}

func TestLivenessEndpointReturnsAlive(t *testing.T) {
	gw := &Gateway{}

	handler := gw.handleLiveness()
	req := httptest.NewRequest(http.MethodGet, "/health/live", nil)
	rec := httptest.NewRecorder()

	handler.ServeHTTP(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%s", rec.Code, http.StatusOK, rec.Body.String())
	}

	var body map[string]string
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("failed to unmarshal response: %v", err)
	}
	if body["status"] != "alive" {
		t.Fatalf("status = %q, want %q", body["status"], "alive")
	}
}
