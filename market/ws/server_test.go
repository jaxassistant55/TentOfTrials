package ws

import (
	"net/http"
	"testing"
)

func TestCheckOriginRejectsUntrustedOriginByDefault(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, "")

	req := originRequest("https://attacker.example", "market.example.com")
	if checkOrigin(req) {
		t.Fatal("untrusted origin should be rejected by default")
	}
}

func TestCheckOriginAllowsConfiguredOrigin(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, " https://app.example.com,https://admin.example.com ")

	req := originRequest("https://app.example.com", "market.example.com")
	if !checkOrigin(req) {
		t.Fatal("configured origin should be allowed")
	}
}

func TestCheckOriginAllowsLoopbackDevelopmentOrigin(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, "")

	req := originRequest("http://localhost:5173", "127.0.0.1:9000")
	if !checkOrigin(req) {
		t.Fatal("loopback development origin should be allowed for loopback server host")
	}
}

func TestCheckOriginRejectsLoopbackOriginForPublicHost(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, "")

	req := originRequest("http://localhost:5173", "market.example.com")
	if checkOrigin(req) {
		t.Fatal("loopback origin should not be a blanket exception for public server hosts")
	}
}

func TestCheckOriginAllowsMissingOrigin(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, "")

	req := originRequest("", "market.example.com")
	if !checkOrigin(req) {
		t.Fatal("missing Origin should be allowed for non-browser clients")
	}
}

func TestCheckOriginRejectsMalformedOrigin(t *testing.T) {
	t.Setenv(wsAllowedOriginsEnv, "https://app.example.com")

	req := originRequest("https://app.example.com/path", "market.example.com")
	if checkOrigin(req) {
		t.Fatal("origin with path should be rejected")
	}
}

func originRequest(origin, host string) *http.Request {
	req := &http.Request{
		Header: make(http.Header),
		Host:   host,
	}
	if origin != "" {
		req.Header.Set("Origin", origin)
	}
	return req
}
