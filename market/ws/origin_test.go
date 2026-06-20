package ws

import (
	"net/http"
	"os"
	"testing"
)

func TestMakeCheckOrigin_AllowedExactMatch(t *testing.T) {
	check := makeCheckOrigin([]string{"https://example.com"})
	r, _ := http.NewRequest("GET", "/ws", nil)
	r.Header.Set("Origin", "https://example.com")
	if !check(r) {
		t.Fatal("expected origin to be allowed")
	}
}

func TestMakeCheckOrigin_AllowedPrefixMatch(t *testing.T) {
	check := makeCheckOrigin([]string{"http://localhost"})
	r, _ := http.NewRequest("GET", "/ws", nil)
	r.Header.Set("Origin", "http://localhost:3000")
	if !check(r) {
		t.Fatal("expected origin with localhost prefix to be allowed")
	}
}

func TestMakeCheckOrigin_Rejected(t *testing.T) {
	check := makeCheckOrigin([]string{"http://localhost"})
	r, _ := http.NewRequest("GET", "/ws", nil)
	r.Header.Set("Origin", "https://evil.com")
	if check(r) {
		t.Fatal("expected origin to be rejected")
	}
}

func TestMakeCheckOrigin_MissingOrigin(t *testing.T) {
	check := makeCheckOrigin([]string{"http://localhost"})
	r, _ := http.NewRequest("GET", "/ws", nil)
	if check(r) {
		t.Fatal("expected connection with no Origin to be rejected")
	}
}

func TestMakeCheckOrigin_EmptyAllowlist(t *testing.T) {
	check := makeCheckOrigin([]string{})
	r, _ := http.NewRequest("GET", "/ws", nil)
	r.Header.Set("Origin", "http://localhost:3000")
	if check(r) {
		t.Fatal("expected origin to be rejected with empty allowlist")
	}
}

func TestOriginAllowlistFromEnv_Defaults(t *testing.T) {
	os.Unsetenv(envAllowedOrigins)
	origins := originAllowlistFromEnv()
	if len(origins) == 0 {
		t.Fatal("expected default localhost origins")
	}
	found := false
	for _, o := range origins {
		if o == "http://localhost" {
			found = true
			break
		}
	}
	if !found {
		t.Fatal("expected http://localhost in default origins")
	}
}

func TestOriginAllowlistFromEnv_Custom(t *testing.T) {
	os.Setenv(envAllowedOrigins, "https://app.example.com,https://admin.example.com")
	defer os.Unsetenv(envAllowedOrigins)

	origins := originAllowlistFromEnv()
	if len(origins) != 2 {
		t.Fatalf("expected 2 origins, got %d", len(origins))
	}
	if origins[0] != "https://app.example.com" {
		t.Fatalf("unexpected first origin: %s", origins[0])
	}
	if origins[1] != "https://admin.example.com" {
		t.Fatalf("unexpected second origin: %s", origins[1])
	}
}

func TestOriginAllowlistFromEnv_EmptyFallsBackToDefault(t *testing.T) {
	os.Setenv(envAllowedOrigins, " , , ")
	defer os.Unsetenv(envAllowedOrigins)

	origins := originAllowlistFromEnv()
	if len(origins) == 0 {
		t.Fatal("expected fallback to default origins")
	}
}
