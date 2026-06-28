import { describe, it, expect } from 'vitest';
import { ReconnectMetrics } from '../services/reconnectMetrics';

describe('ReconnectMetrics', () => {
  it('starts with zero counts', () => {
    const m = new ReconnectMetrics();
    const snap = m.snapshot();
    expect(snap.totalAttempts).toBe(0);
    expect(snap.successfulReconnects).toBe(0);
    expect(snap.failedReconnects).toBe(0);
    expect(snap.consecutiveFailures).toBe(0);
  });

  it('records attempts', () => {
    const m = new ReconnectMetrics();
    m.recordAttempt(1000);
    m.recordAttempt(2000);
    expect(m.snapshot().totalAttempts).toBe(2);
    expect(m.snapshot().lastBackoffMs).toBe(2000);
  });

  it('records successful reconnects', () => {
    const m = new ReconnectMetrics();
    m.recordAttempt(1000);
    m.recordSuccess(1000);
    expect(m.snapshot().successfulReconnects).toBe(1);
    expect(m.snapshot().consecutiveFailures).toBe(0);
    expect(m.snapshot().lastSuccessAt).not.toBeNull();
  });

  it('records failed reconnects and tracks consecutive failures', () => {
    const m = new ReconnectMetrics();
    m.recordFailure();
    m.recordFailure();
    expect(m.snapshot().failedReconnects).toBe(2);
    expect(m.snapshot().consecutiveFailures).toBe(2);
  });

  it('resets consecutive failures on success', () => {
    const m = new ReconnectMetrics();
    m.recordFailure();
    m.recordFailure();
    expect(m.snapshot().consecutiveFailures).toBe(2);
    m.recordSuccess(500);
    expect(m.snapshot().consecutiveFailures).toBe(0);
  });

  it('tracks backoff statistics', () => {
    const m = new ReconnectMetrics();
    m.recordAttempt(1000);
    m.recordAttempt(2000);
    m.recordAttempt(4000);
    const snap = m.snapshot();
    expect(snap.minBackoffMs).toBe(1000);
    expect(snap.maxBackoffMs).toBe(4000);
    expect(snap.averageBackoffMs).toBe(2333); // (1000+2000+4000)/3 ≈ 2333
  });

  it('handles zero backoff (initial attempt)', () => {
    const m = new ReconnectMetrics();
    m.recordAttempt(0);
    expect(m.snapshot().totalAttempts).toBe(1);
    expect(m.snapshot().averageBackoffMs).toBe(0);
    expect(m.snapshot().minBackoffMs).toBe(0);
  });

  it('resets all counters', () => {
    const m = new ReconnectMetrics();
    m.recordAttempt(1000);
    m.recordSuccess(1000);
    m.recordFailure();
    m.reset();
    const snap = m.snapshot();
    expect(snap.totalAttempts).toBe(0);
    expect(snap.successfulReconnects).toBe(0);
    expect(snap.failedReconnects).toBe(0);
    expect(snap.consecutiveFailures).toBe(0);
    expect(snap.lastAttemptAt).toBeNull();
    expect(snap.lastSuccessAt).toBeNull();
    expect(snap.minBackoffMs).toBe(0);
    expect(snap.maxBackoffMs).toBe(0);
  });

  it('snapshot returns lastAttemptAt timestamp', () => {
    const m = new ReconnectMetrics();
    const before = Date.now();
    m.recordAttempt(500);
    const after = Date.now();
    const snap = m.snapshot();
    expect(snap.lastAttemptAt).toBeGreaterThanOrEqual(before);
    expect(snap.lastAttemptAt).toBeLessThanOrEqual(after);
  });
});
