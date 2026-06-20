package ws

import (
	"net/http"
	"testing"
)

func TestOriginPolicyAllowsConfiguredOrigin(t *testing.T) {
	policy := newOriginPolicy([]string{"https://trade.example.com"})
	req := originRequest("market.internal", "https://trade.example.com")

	if !policy.Check(req) {
		t.Fatal("configured origin should be allowed")
	}
}

func TestOriginPolicyRejectsUntrustedOriginByDefault(t *testing.T) {
	policy := newOriginPolicy(nil)
	req := originRequest("market.internal", "https://evil.example")

	if policy.Check(req) {
		t.Fatal("untrusted cross-site origin should be rejected")
	}
}

func TestOriginPolicyAllowsMissingOrigin(t *testing.T) {
	policy := newOriginPolicy(nil)
	req := originRequest("market.internal", "")

	if !policy.Check(req) {
		t.Fatal("missing Origin should be allowed for non-browser clients")
	}
}

func TestOriginPolicyAllowsLocalhostDevelopmentOrigin(t *testing.T) {
	policy := newOriginPolicy(nil)
	req := originRequest("127.0.0.1:9000", "http://localhost:5173")

	if !policy.Check(req) {
		t.Fatal("localhost development origin should be allowed for localhost server")
	}
}

func TestParseAllowedOriginsTrimsEmptyValues(t *testing.T) {
	got := parseAllowedOrigins(" https://a.example, ,https://b.example ")
	if len(got) != 2 {
		t.Fatalf("origins length = %d, want 2", len(got))
	}
	if got[0] != "https://a.example" || got[1] != "https://b.example" {
		t.Fatalf("origins = %#v", got)
	}
}

func originRequest(host, origin string) *http.Request {
	req, err := http.NewRequest(http.MethodGet, "http://"+host+"/ws", nil)
	if err != nil {
		panic(err)
	}
	req.Host = host
	if origin != "" {
		req.Header.Set("Origin", origin)
	}
	return req
}
