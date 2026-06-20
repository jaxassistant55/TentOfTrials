# Frontend Socket Metrics

## Overview

The `useSocketMetrics` hook provides lightweight observable metrics for WebSocket connection lifecycle testing and diagnostics. It tracks reconnection attempts, disconnect reasons, and connection timestamps without exposing sensitive data like authentication tokens or full message payloads.

## Installation & Usage

### Basic Usage

```typescript
import { useSocketMetrics } from '@/hooks/useSocketMetrics';

function MyComponent() {
  const metrics = useSocketMetrics();

  return (
    <div>
      <p>Connection attempts: {metrics.metrics.reconnectAttemptCount}</p>
      <p>Last disconnect: {metrics.metrics.lastDisconnectReason}</p>
      <p>Time since connected: {metrics.metrics.timeSinceLastConnection}ms</p>
    </div>
  );
}
```

### With Callbacks

```typescript
const metrics = useSocketMetrics({
  onConnected: (timestamp) => console.log(`Connected at ${timestamp}`),
  onReconnectAttempt: (attempt, reason) => {
    console.log(`Reconnection attempt ${attempt}: ${reason}`);
  },
  onCleanDisconnect: () => console.log('Clean disconnect'),
});
```

## Metrics Fields

### `reconnectAttemptCount: number`

The number of reconnection attempts since component mount. This is automatically reset to 0 on successful connection.

**Use cases:**
- Detecting excessive reconnection loops
- Testing exponential backoff behavior
- Monitoring connection stability

### `lastDisconnectReason: string | null`

The reason for the most recent disconnect. Common values:

- `'clean'` - Manual disconnect (e.g., user logout, page navigation)
- `'connection lost'` - Network connectivity issue
- `'peer reset'` - Server closed connection unexpectedly
- `'timeout'` - Pong timeout or connection establishment timeout
- `'network error'` - General network error

**Use cases:**
- Diagnosing connection failures
- Distinguishing between user-initiated and error-based disconnects
- Telemetry and error categorization

### `lastConnectedTimestamp: string | null`

ISO 8601 formatted timestamp of the last successful connection (e.g., `2024-06-20T14:30:45.123Z`), or `null` if never connected.

**Use cases:**
- Measuring session duration
- Debugging stale connection issues
- Recording connection history

### `timeSinceLastConnection: number | null`

Milliseconds elapsed since the last successful connection, updated every second. Returns `null` if never connected. Automatically paused when disconnected.

**Use cases:**
- Measuring connection age
- Implementing timeout logic
- UI progress indicators

## Integration with useWebSocket

When integrating metrics with the existing `useWebSocket` hook, call the appropriate methods during socket lifecycle events:

```typescript
function SocketComponent() {
  const metrics = useSocketMetrics();
  const ws = useWebSocket({
    url: 'ws://example.com/socket',
    onOpen: (event) => {
      metrics.recordConnected();
    },
    onClose: (event) => {
      const reason = event.code === 1000 ? 'clean' : `close code ${event.code}`;
      metrics.recordDisconnect(reason, event.code === 1000);
    },
    onError: (event) => {
      metrics.recordReconnectAttempt('connection error');
    },
  });

  return (
    <div>
      <p>Attempts: {metrics.metrics.reconnectAttemptCount}</p>
    </div>
  );
}
```

## API Reference

### `useSocketMetrics(callbacks?: SocketMetricsCallbacks)`

**Parameters:**
- `callbacks` (optional): Object with optional callback functions

**Callbacks:**
- `onConnected(timestamp: string)` - Called when connection succeeds
- `onReconnectAttempt(attempt: number, reason: string)` - Called on reconnection attempt
- `onCleanDisconnect()` - Called on manual disconnect

**Returns:**

```typescript
{
  metrics: SocketMetrics;           // Current metrics state
  recordConnected(): void;          // Record successful connection
  recordReconnectAttempt(reason: string): void;  // Record reconnection attempt
  recordDisconnect(reason: string, isManual?: boolean): void;  // Record disconnect
  reset(): void;                    // Reset all metrics to initial state
}
```

## Testing

The hook is designed for testing with standard React testing utilities:

```typescript
import { renderHook, act } from '@testing-library/react';
import { useSocketMetrics } from '@/hooks/useSocketMetrics';

test('increments reconnect attempts', () => {
  const { result } = renderHook(() => useSocketMetrics());

  act(() => {
    result.current.recordReconnectAttempt('timeout');
  });

  expect(result.current.metrics.reconnectAttemptCount).toBe(1);
});
```

## Security Considerations

- **No token exposure**: Metrics do not capture authentication tokens or credentials
- **No payload logging**: Message payloads are not recorded, only metadata
- **Sanitized reasons**: Disconnect reasons are standardized strings without raw error messages
- **Client-side only**: Metrics remain in browser memory and are not automatically sent to server

## Performance

- **Lightweight**: Minimal memory overhead, no expensive computations
- **Optional timer**: Only active interval timer when connected
- **No network impact**: Purely client-side metrics
- **Callback cleanup**: Stale callbacks are handled gracefully

## Changelog

### v1.0 (Initial Release)
- Initial metrics implementation
- Support for reconnect attempt tracking
- Disconnect reason recording
- Connection timestamp tracking
- Comprehensive test coverage
