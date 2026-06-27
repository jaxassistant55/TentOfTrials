package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessEndpointReportsReady(t *testing.T) {
	gateway := newReadinessTestGateway()
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusOK, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, "ready")
}

func TestReadinessEndpointReportsNotReady(t *testing.T) {
	gateway := newReadinessTestGateway()
	gateway.health.Store(false)
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusServiceUnavailable, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, "not ready")
}

func newReadinessTestGateway() *Gateway {
	config := DefaultGatewayConfig()
	config.LogRequests = false
	return NewGateway(config)
}

func assertReadinessResponse(t *testing.T, recorder *httptest.ResponseRecorder, wantStatus string) {
	t.Helper()

	if got := recorder.Header().Get("Content-Type"); got != "application/json" {
		t.Fatalf("content-type = %q, want application/json", got)
	}

	var body map[string]string
	if err := json.Unmarshal(recorder.Body.Bytes(), &body); err != nil {
		t.Fatalf("readiness response is not valid JSON: %v; body=%s", err, recorder.Body.String())
	}

	if got := body["status"]; got != wantStatus {
		t.Fatalf("response status = %q, want %q; body=%s", got, wantStatus, recorder.Body.String())
	}
}
