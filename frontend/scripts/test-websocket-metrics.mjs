import assert from 'node:assert/strict';
import test from 'node:test';
import url from 'node:url';

const helperUrl = url.pathToFileURL(new URL('../src/hooks/webSocketMetrics.ts', import.meta.url).pathname).href;
const {
  createInitialWSConnectionMetrics,
  isStaleWSCallback,
  recordWSConnected,
  recordWSDisconnect,
  recordWSReconnectAttempt,
} = await import(`${helperUrl}?t=${Date.now()}`);

test('connection metrics initial state is empty and safe to expose', () => {
  assert.deepEqual(createInitialWSConnectionMetrics(), {
    reconnectAttemptCount: 0,
    lastDisconnectReason: null,
    lastConnectedAt: null,
  });
});

test('reconnect attempts increment without storing socket payloads', () => {
  const first = recordWSReconnectAttempt(createInitialWSConnectionMetrics());
  const second = recordWSReconnectAttempt(first);

  assert.equal(first.reconnectAttemptCount, 1);
  assert.equal(second.reconnectAttemptCount, 2);
  assert.deepEqual(Object.keys(second).sort(), [
    'lastConnectedAt',
    'lastDisconnectReason',
    'reconnectAttemptCount',
  ]);
});

test('clean disconnect records only the close reason', () => {
  const metrics = recordWSDisconnect(createInitialWSConnectionMetrics(), {
    code: 1000,
    reason: 'Client disconnect',
    wasClean: true,
  });

  assert.equal(metrics.lastDisconnectReason, 'Client disconnect');
});

test('connected timestamp and stale callback detection are deterministic', () => {
  const connected = recordWSConnected(createInitialWSConnectionMetrics(), 1700000000000);

  assert.equal(connected.lastConnectedAt, 1700000000000);
  assert.equal(isStaleWSCallback(1, 2), true);
  assert.equal(isStaleWSCallback(2, 2), false);
});
