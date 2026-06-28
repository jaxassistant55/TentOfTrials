import assert from 'node:assert/strict';
import test from 'node:test';
import url from 'node:url';

const helperUrl = url.pathToFileURL(new URL('../src/hooks/webSocketLifecycle.ts', import.meta.url).pathname).href;
const {
  beginWSConnection,
  clearWSReconnectTimer,
  createWSConnectionLifecycle,
  isActiveWSConnection,
  isWSLifecycleMounted,
  markWSMounted,
  markWSReconnectTimerFired,
  markWSUnmounted,
  setWSReconnectTimer,
} = await import(`${helperUrl}?t=${Date.now()}`);

function createTimerHarness() {
  let nextId = 0;
  const timers = new Map();
  const cleared = [];

  return {
    cleared,
    setTimeout(callback, delay) {
      const id = ++nextId;
      timers.set(id, { callback, delay });
      return id;
    },
    clearTimeout(id) {
      cleared.push(id);
      timers.delete(id);
    },
    runAll() {
      for (const [id, timer] of [...timers]) {
        timers.delete(id);
        timer.callback();
      }
    },
  };
}

test('unmount clears a pending reconnect timer before it can run', () => {
  const lifecycle = createWSConnectionLifecycle();
  const timers = createTimerHarness();
  let reconnects = 0;

  const timer = timers.setTimeout(() => {
    if (isWSLifecycleMounted(lifecycle)) {
      reconnects += 1;
    }
  }, 250);
  setWSReconnectTimer(lifecycle, timer);

  markWSUnmounted(lifecycle, timers.clearTimeout);
  timers.runAll();

  assert.equal(reconnects, 0);
  assert.equal(lifecycle.reconnectTimer, null);
  assert.deepEqual(timers.cleared, [timer]);
});

test('stale socket callbacks are rejected after a newer connection exists', () => {
  const lifecycle = createWSConnectionLifecycle();
  const firstGeneration = beginWSConnection(lifecycle);
  const secondGeneration = beginWSConnection(lifecycle);

  assert.equal(isActiveWSConnection(lifecycle, firstGeneration, true), false);
  assert.equal(isActiveWSConnection(lifecycle, secondGeneration, true), true);
  assert.equal(isActiveWSConnection(lifecycle, secondGeneration, false), false);
});

test('unmount rejects callbacks from the latest connection generation', () => {
  const lifecycle = createWSConnectionLifecycle();
  const timers = createTimerHarness();
  const generation = beginWSConnection(lifecycle);

  markWSUnmounted(lifecycle, timers.clearTimeout);

  assert.equal(isActiveWSConnection(lifecycle, generation, true), false);
});

test('reconnect timer can fire once and be replaced deterministically', () => {
  const lifecycle = createWSConnectionLifecycle();
  const timers = createTimerHarness();
  const first = timers.setTimeout(() => markWSReconnectTimerFired(lifecycle), 100);
  const second = timers.setTimeout(() => markWSReconnectTimerFired(lifecycle), 200);

  setWSReconnectTimer(lifecycle, first);
  clearWSReconnectTimer(lifecycle, timers.clearTimeout);
  setWSReconnectTimer(lifecycle, second);
  markWSMounted(lifecycle);
  timers.runAll();

  assert.equal(lifecycle.reconnectTimer, null);
  assert.deepEqual(timers.cleared, [first]);
});
