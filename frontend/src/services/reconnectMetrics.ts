/**
 * WebSocket Reconnect Metrics
 *
 * Tracks reconnect attempts, successful reconnects, and backoff durations
 * for frontend WebSocket connections. Designed to be reported to the
 * telemetry service for monitoring and alerting.
 *
 * Usage:
 *   const metrics = new ReconnectMetrics();
 *   metrics.recordAttempt();
 *   metrics.recordSuccess(45);  // reconnect took 45ms
 *   metrics.recordFailure();
 *   console.log(metrics.snapshot());
 */

export interface ReconnectMetricsSnapshot {
  /** Total reconnect attempts since last reset */
  totalAttempts: number;
  /** Successful reconnects (connection re-established) */
  successfulReconnects: number;
  /** Failed reconnect attempts (gave up after max retries) */
  failedReconnects: number;
  /** Current consecutive failure count (resets on success) */
  consecutiveFailures: number;
  /** Last backoff delay in milliseconds */
  lastBackoffMs: number;
  /** Average backoff delay in milliseconds */
  averageBackoffMs: number;
  /** Minimum backoff delay observed */
  minBackoffMs: number;
  /** Maximum backoff delay observed */
  maxBackoffMs: number;
  /** Timestamp of the last reconnect attempt (ms since epoch) */
  lastAttemptAt: number | null;
  /** Timestamp of the last successful reconnect (ms since epoch) */
  lastSuccessAt: number | null;
}

export class ReconnectMetrics {
  private totalAttempts = 0;
  private successfulReconnects = 0;
  private failedReconnects = 0;
  private consecutiveFailures = 0;
  private backoffSum = 0;
  private backoffCount = 0;
  private _minBackoff = Infinity;
  private _maxBackoff = 0;
  private lastBackoffMs = 0;
  private lastAttemptAt: number | null = null;
  private lastSuccessAt: number | null = null;

  /** Record a reconnect attempt with the computed backoff delay. */
  recordAttempt(backoffMs: number = 0): void {
    this.totalAttempts++;
    this.lastAttemptAt = Date.now();
    this.lastBackoffMs = backoffMs;

    if (backoffMs > 0) {
      this.backoffSum += backoffMs;
      this.backoffCount++;
      if (backoffMs < this._minBackoff) this._minBackoff = backoffMs;
      if (backoffMs > this._maxBackoff) this._maxBackoff = backoffMs;
    }
  }

  /** Record a successful reconnect (connection re-established). */
  recordSuccess(backoffMs: number = 0): void {
    this.successfulReconnects++;
    this.consecutiveFailures = 0;
    this.lastSuccessAt = Date.now();
    this.lastBackoffMs = backoffMs;

    if (backoffMs > 0) {
      this.backoffSum += backoffMs;
      this.backoffCount++;
      if (backoffMs < this._minBackoff) this._minBackoff = backoffMs;
      if (backoffMs > this._maxBackoff) this._maxBackoff = backoffMs;
    }
  }

  /** Record a failed reconnect (gave up after max retries). */
  recordFailure(): void {
    this.failedReconnects++;
    this.consecutiveFailures++;
  }

  /** Return a snapshot of the current metrics. */
  snapshot(): ReconnectMetricsSnapshot {
    return {
      totalAttempts: this.totalAttempts,
      successfulReconnects: this.successfulReconnects,
      failedReconnects: this.failedReconnects,
      consecutiveFailures: this.consecutiveFailures,
      lastBackoffMs: this.lastBackoffMs,
      averageBackoffMs: this.backoffCount > 0 ? Math.round(this.backoffSum / this.backoffCount) : 0,
      minBackoffMs: this.backoffCount > 0 ? this._minBackoff : 0,
      maxBackoffMs: this._maxBackoff,
      lastAttemptAt: this.lastAttemptAt,
      lastSuccessAt: this.lastSuccessAt,
    };
  }

  /** Reset all counters. */
  reset(): void {
    this.totalAttempts = 0;
    this.successfulReconnects = 0;
    this.failedReconnects = 0;
    this.consecutiveFailures = 0;
    this.backoffSum = 0;
    this.backoffCount = 0;
    this._minBackoff = Infinity;
    this._maxBackoff = 0;
    this.lastBackoffMs = 0;
    this.lastAttemptAt = null;
    this.lastSuccessAt = null;
  }
}
