package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessStateStoreTransitions(t *testing.T) {
	store := NewReadinessStateStore()

	if got := store.State(); got != ReadinessReady {
		t.Fatalf("initial state = %q, want %q", got, ReadinessReady)
	}

	store.MarkDraining()
	if got := store.State(); got != ReadinessDraining {
		t.Fatalf("draining state = %q, want %q", got, ReadinessDraining)
	}

	store.MarkReady()
	if got := store.State(); got != ReadinessReady {
		t.Fatalf("ready state after transition = %q, want %q", got, ReadinessReady)
	}
}

func TestReadinessEndpointReturnsReadyByDefault(t *testing.T) {
	gateway := newReadinessTestGateway()
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusOK, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, map[string]string{"status": "ready"})
}

func TestReadinessEndpointReturnsNotReadyWhenHealthFails(t *testing.T) {
	gateway := newReadinessTestGateway()
	gateway.health.Store(false)
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusServiceUnavailable, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, map[string]string{"status": "not ready"})
}

func TestReadinessEndpointReturnsNotReadyWhenDraining(t *testing.T) {
	gateway := newReadinessTestGateway()
	gateway.MarkDraining()
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusServiceUnavailable, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, map[string]string{
		"status": "not ready",
		"state":  "draining",
	})
}

func TestReadinessEndpointReturnsReadyAfterDrainTransition(t *testing.T) {
	gateway := newReadinessTestGateway()
	gateway.MarkDraining()
	gateway.MarkReady()
	recorder := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodGet, "/health/ready", nil)

	gateway.buildHandler().ServeHTTP(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%s", recorder.Code, http.StatusOK, recorder.Body.String())
	}
	assertReadinessResponse(t, recorder, map[string]string{"status": "ready"})
}

func newReadinessTestGateway() *Gateway {
	config := DefaultGatewayConfig()
	config.LogRequests = false
	return NewGateway(config)
}

func assertReadinessResponse(t *testing.T, recorder *httptest.ResponseRecorder, want map[string]string) {
	t.Helper()

	if got := recorder.Header().Get("Content-Type"); got != "application/json" {
		t.Fatalf("content-type = %q, want application/json", got)
	}

	var body map[string]string
	if err := json.Unmarshal(recorder.Body.Bytes(), &body); err != nil {
		t.Fatalf("readiness response is not valid JSON: %v; body=%s", err, recorder.Body.String())
	}

	if len(body) != len(want) {
		t.Fatalf("response fields = %#v, want %#v", body, want)
	}
	for key, wantValue := range want {
		if got := body[key]; got != wantValue {
			t.Fatalf("response %s = %q, want %q; body=%s", key, got, wantValue, recorder.Body.String())
		}
	}
}
