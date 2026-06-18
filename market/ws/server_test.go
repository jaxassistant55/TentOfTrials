package ws

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"go.uber.org/zap"
)

func testWSServer(t *testing.T, hub *Hub) *httptest.Server {
	t.Helper()

	server := NewServer(hub, nil, zap.NewNop(), 0)
	return httptest.NewServer(http.HandlerFunc(server.handleWebSocket))
}

func wsURL(serverURL string) string {
	return "ws" + strings.TrimPrefix(serverURL, "http") + "/ws"
}

func waitForCondition(t *testing.T, timeout time.Duration, condition func() bool) {
	t.Helper()

	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if condition() {
			return
		}
		time.Sleep(20 * time.Millisecond)
	}
	t.Fatalf("condition was not met within %s", timeout)
}

func TestHeartbeatIntervalFromEnv(t *testing.T) {
	t.Setenv("WS_HEARTBEAT_INTERVAL_SECS", "2")
	if got := heartbeatIntervalFromEnv(); got != 2*time.Second {
		t.Fatalf("heartbeat interval = %s, want 2s", got)
	}

	t.Setenv("WS_HEARTBEAT_INTERVAL_SECS", "not-a-number")
	if got := heartbeatIntervalFromEnv(); got != defaultHeartbeatInterval {
		t.Fatalf("invalid heartbeat interval = %s, want default %s", got, defaultHeartbeatInterval)
	}
}

func TestIdleClientDisconnectsWithoutPong(t *testing.T) {
	t.Setenv("WS_HEARTBEAT_INTERVAL_SECS", "1")

	hub := NewHub(zap.NewNop())
	go hub.Run()

	testServer := testWSServer(t, hub)
	defer testServer.Close()

	conn, _, err := websocket.DefaultDialer.Dial(wsURL(testServer.URL), nil)
	if err != nil {
		t.Fatalf("dial websocket: %v", err)
	}
	defer conn.Close()

	waitForCondition(t, time.Second, func() bool {
		return hub.ActiveConnectionCount() == 1
	})

	waitForCondition(t, 4*time.Second, func() bool {
		return hub.ActiveConnectionCount() == 0
	})
}

func TestPongingClientStaysConnectedAndTracksLastPong(t *testing.T) {
	t.Setenv("WS_HEARTBEAT_INTERVAL_SECS", "1")

	hub := NewHub(zap.NewNop())
	go hub.Run()

	testServer := testWSServer(t, hub)
	defer testServer.Close()

	conn, _, err := websocket.DefaultDialer.Dial(wsURL(testServer.URL), nil)
	if err != nil {
		t.Fatalf("dial websocket: %v", err)
	}
	defer conn.Close()

	done := make(chan struct{})
	go func() {
		defer close(done)
		for {
			if _, _, err := conn.ReadMessage(); err != nil {
				return
			}
		}
	}()

	waitForCondition(t, time.Second, func() bool {
		return hub.ActiveConnectionCount() == 1
	})

	time.Sleep(2500 * time.Millisecond)

	if got := hub.ActiveConnectionCount(); got != 1 {
		t.Fatalf("active connections = %d, want 1", got)
	}

	health := hub.ConnectionHealth()
	if len(health) != 1 {
		t.Fatalf("connection health entries = %d, want 1", len(health))
	}
	if time.Since(health[0].LastPong) > 2*time.Second {
		t.Fatalf("last pong was not refreshed recently: %s", health[0].LastPong)
	}

	_ = conn.Close()
	<-done
}
