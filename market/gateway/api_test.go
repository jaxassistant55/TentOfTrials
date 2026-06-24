package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestGatewayReadinessReportsReady(t *testing.T) {
	gateway := newReadinessTestGateway()

	response := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	gateway.buildHandler().ServeHTTP(response, request)

	assertReadinessResponse(t, response, http.StatusOK, "ready")
}

func TestGatewayReadinessReportsNotReady(t *testing.T) {
	gateway := newReadinessTestGateway()
	gateway.health.Store(false)

	response := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	gateway.buildHandler().ServeHTTP(response, request)

	assertReadinessResponse(t, response, http.StatusServiceUnavailable, "not ready")
}

func newReadinessTestGateway() *Gateway {
	config := DefaultGatewayConfig()
	config.RateLimitEnabled = false
	config.LogRequests = false
	config.EnableMetrics = false
	config.WSEnabled = false

	return NewGateway(config)
}

func assertReadinessResponse(t *testing.T, response *httptest.ResponseRecorder, wantCode int, wantStatus string) {
	t.Helper()

	if response.Code != wantCode {
		t.Fatalf("status code = %d, want %d; body = %s", response.Code, wantCode, response.Body.String())
	}

	contentType := response.Header().Get("Content-Type")
	if !strings.HasPrefix(contentType, "application/json") {
		t.Fatalf("Content-Type = %q, want application/json", contentType)
	}

	var payload struct {
		Status string `json:"status"`
	}
	if err := json.NewDecoder(response.Body).Decode(&payload); err != nil {
		t.Fatalf("decode readiness response: %v", err)
	}
	if payload.Status != wantStatus {
		t.Fatalf("status = %q, want %q", payload.Status, wantStatus)
	}
}
