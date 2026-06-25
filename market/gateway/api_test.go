package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestReadinessEndpointReturnsReady(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())
	server := httptest.NewServer(gateway.buildHandler())
	defer server.Close()

	resp, err := http.Get(server.URL + "/health/ready")
	if err != nil {
		t.Fatalf("GET /health/ready: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("status = %d, want %d", resp.StatusCode, http.StatusOK)
	}

	var body map[string]string
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("decode response body: %v", err)
	}
	if body["status"] != "ready" {
		t.Fatalf("status body = %q, want %q", body["status"], "ready")
	}
}

func TestReadinessEndpointReturnsNotReadyWhenGatewayUnhealthy(t *testing.T) {
	gateway := NewGateway(DefaultGatewayConfig())
	gateway.health.Store(false)
	server := httptest.NewServer(gateway.buildHandler())
	defer server.Close()

	resp, err := http.Get(server.URL + "/health/ready")
	if err != nil {
		t.Fatalf("GET /health/ready: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusServiceUnavailable {
		t.Fatalf("status = %d, want %d", resp.StatusCode, http.StatusServiceUnavailable)
	}

	var body map[string]string
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("decode response body: %v", err)
	}
	if body["status"] != "not ready" {
		t.Fatalf("status body = %q, want %q", body["status"], "not ready")
	}
}
