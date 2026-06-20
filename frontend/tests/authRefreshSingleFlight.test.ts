import assert from 'node:assert/strict';
import test from 'node:test';

import { createRefreshSingleFlight } from '../src/services/authRefreshSingleFlight.js';

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

test('concurrent successful refresh calls share one in-flight request and one state write', async () => {
  const request = deferred<{ accessToken: string; refreshToken: string }>();
  const nextTokens = { accessToken: 'access-next', refreshToken: 'refresh-next' };
  let performCount = 0;
  let successWrites = 0;
  let failureWrites = 0;

  const refresh = createRefreshSingleFlight({
    loadRefreshToken: () => 'refresh-current',
    performRefresh: async (refreshToken: string) => {
      performCount += 1;
      assert.equal(refreshToken, 'refresh-current');
      return request.promise;
    },
    applySuccess: (tokens) => {
      successWrites += 1;
      assert.deepEqual(tokens, nextTokens);
    },
    applyFailure: () => {
      failureWrites += 1;
    },
  });

  const first = refresh();
  const second = refresh();
  const third = refresh();

  assert.strictEqual(first, second);
  assert.strictEqual(second, third);
  assert.equal(performCount, 1);

  request.resolve(nextTokens);
  const results = await Promise.all([first, second, third]);

  assert.deepEqual(results, [nextTokens, nextTokens, nextTokens]);
  assert.equal(successWrites, 1);
  assert.equal(failureWrites, 0);
});

test('concurrent failed refresh calls share one in-flight request and one failure write', async () => {
  const request = deferred<{ accessToken: string; refreshToken: string }>();
  let performCount = 0;
  let successWrites = 0;
  let failureWrites = 0;

  const refresh = createRefreshSingleFlight({
    loadRefreshToken: () => 'refresh-current',
    performRefresh: async () => {
      performCount += 1;
      return request.promise;
    },
    applySuccess: () => {
      successWrites += 1;
    },
    applyFailure: () => {
      failureWrites += 1;
    },
  });

  const first = refresh();
  const second = refresh();

  assert.strictEqual(first, second);
  assert.equal(performCount, 1);

  request.reject(new Error('refresh failed'));
  const results = await Promise.all([first, second]);

  assert.deepEqual(results, [null, null]);
  assert.equal(successWrites, 0);
  assert.equal(failureWrites, 1);
});

test('a settled refresh allows the next refresh attempt to start a new request', async () => {
  let performCount = 0;

  const refresh = createRefreshSingleFlight({
    loadRefreshToken: () => 'refresh-current',
    performRefresh: async () => {
      performCount += 1;
      return { accessToken: `access-${performCount}`, refreshToken: `refresh-${performCount}` };
    },
    applySuccess: () => {},
    applyFailure: () => {},
  });

  assert.equal((await refresh())?.accessToken, 'access-1');
  assert.equal((await refresh())?.accessToken, 'access-2');
  assert.equal(performCount, 2);
});
