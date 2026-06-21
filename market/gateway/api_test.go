package gateway

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHandleReadiness(t *testing.T) {
	config := GatewayConfig{}
	g := NewGateway(config)

	// Test Success
	t.Run("Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/readiness", nil)
		rr := httptest.NewRecorder()

		handler := g.handleReadiness()
		handler.ServeHTTP(rr, req)

		if status := rr.Code; status != http.StatusOK {
			t.Errorf("handler returned wrong status code: got %v want %v",
				status, http.StatusOK)
		}

		var response map[string]string
		if err := json.NewDecoder(rr.Body).Decode(&response); err != nil {
			t.Fatalf("failed to decode response: %v", err)
		}

		if status, ok := response["status"]; !ok || status != "ready" {
			t.Errorf("handler returned unexpected body: got %v want ready", response)
		}
	})

	// Test Failure (Not Ready)
	t.Run("Failure", func(t *testing.T) {
		g.health.Store(false)

		req := httptest.NewRequest(http.MethodGet, "/readiness", nil)
		rr := httptest.NewRecorder()

		handler := g.handleReadiness()
		handler.ServeHTTP(rr, req)

		if status := rr.Code; status != http.StatusServiceUnavailable {
			t.Errorf("handler returned wrong status code: got %v want %v",
				status, http.StatusServiceUnavailable)
		}

		var response map[string]string
		if err := json.NewDecoder(rr.Body).Decode(&response); err != nil {
			t.Fatalf("failed to decode response: %v", err)
		}

		if status, ok := response["status"]; !ok || status != "not ready" {
			t.Errorf("handler returned unexpected body: got %v want not ready", response)
		}
	})
}
