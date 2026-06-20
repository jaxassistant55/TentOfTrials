package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessStatusTransitions(t *testing.T) {
	status := NewReadinessStatus()
	if !status.Ready() || status.State() != ReadinessReady {
		t.Fatalf("new readiness status = %q, want ready", status.State())
	}

	status.SetDraining()
	if status.Ready() || status.State() != ReadinessDraining {
		t.Fatalf("draining status = %q, want draining", status.State())
	}

	status.SetNotReady()
	if status.Ready() || status.State() != ReadinessNotReady {
		t.Fatalf("not-ready status = %q, want not ready", status.State())
	}

	status.SetReady()
	if !status.Ready() || status.State() != ReadinessReady {
		t.Fatalf("restored status = %q, want ready", status.State())
	}
}

func TestGatewayReadinessReadyResponse(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())

	statusCode, body := readinessResponse(t, gateway)
	if statusCode != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%v", statusCode, http.StatusOK, body)
	}
	if body["status"] != string(ReadinessReady) {
		t.Fatalf("readiness status = %q, want ready", body["status"])
	}
}

func TestGatewayReadinessDrainingResponse(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())
	gateway.SetDraining(true)

	statusCode, body := readinessResponse(t, gateway)
	if statusCode != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%v", statusCode, http.StatusServiceUnavailable, body)
	}
	if body["status"] != string(ReadinessDraining) {
		t.Fatalf("readiness status = %q, want draining", body["status"])
	}
	if !gateway.Health() {
		t.Fatal("draining readiness should not change the health flag")
	}

	rec := httptest.NewRecorder()
	gateway.handleLiveness().ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/health/live", nil))
	if rec.Code != http.StatusOK {
		t.Fatalf("liveness status = %d, want %d", rec.Code, http.StatusOK)
	}
}

func TestGatewayReadinessDrainTransitionRestoresReady(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())
	gateway.SetDraining(true)
	gateway.SetDraining(false)

	statusCode, body := readinessResponse(t, gateway)
	if statusCode != http.StatusOK {
		t.Fatalf("status = %d, want %d; body=%v", statusCode, http.StatusOK, body)
	}
	if body["status"] != string(ReadinessReady) {
		t.Fatalf("readiness status = %q, want ready", body["status"])
	}
}

func TestGatewayReadinessUnhealthyResponse(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())
	gateway.health.Store(false)

	statusCode, body := readinessResponse(t, gateway)
	if statusCode != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d; body=%v", statusCode, http.StatusServiceUnavailable, body)
	}
	if body["status"] != string(ReadinessNotReady) {
		t.Fatalf("readiness status = %q, want not ready", body["status"])
	}
}

func readinessResponse(t *testing.T, gateway *Gateway) (int, map[string]string) {
	t.Helper()

	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/health/ready", nil)
	gateway.handleReadiness().ServeHTTP(rec, req)

	body := make(map[string]string)
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode readiness body: %v; raw=%s", err, rec.Body.String())
	}
	return rec.Code, body
}
