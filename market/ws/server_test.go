package ws

import (
	"net/http/httptest"
	"testing"
)

func TestCheckOriginAllowsConfiguredOrigin(t *testing.T) {
	t.Setenv(allowedOriginsEnv, "https://app.example.com")

	req := httptest.NewRequest("GET", "http://market.example.com/ws", nil)
	req.Header.Set("Origin", "https://app.example.com")

	if !checkOrigin(req) {
		t.Fatal("configured origin should be allowed")
	}
}

func TestCheckOriginRejectsUntrustedOrigin(t *testing.T) {
	t.Setenv(allowedOriginsEnv, "https://app.example.com")

	req := httptest.NewRequest("GET", "http://market.example.com/ws", nil)
	req.Header.Set("Origin", "https://evil.example.com")

	if checkOrigin(req) {
		t.Fatal("unconfigured cross-site origin should be rejected")
	}
}

func TestCheckOriginAllowsMissingOrigin(t *testing.T) {
	t.Setenv(allowedOriginsEnv, "")

	req := httptest.NewRequest("GET", "http://market.example.com/ws", nil)

	if !checkOrigin(req) {
		t.Fatal("missing Origin should be allowed for non-browser clients")
	}
}

func TestCheckOriginAllowsLocalhostDevelopmentOrigin(t *testing.T) {
	t.Setenv(allowedOriginsEnv, "")

	req := httptest.NewRequest("GET", "http://127.0.0.1:8080/ws", nil)
	req.Header.Set("Origin", "http://localhost:5173")

	if !checkOrigin(req) {
		t.Fatal("localhost development origin should be allowed")
	}
}
