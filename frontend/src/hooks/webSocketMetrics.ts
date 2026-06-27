export interface WSConnectionMetrics {
  reconnectAttemptCount: number;
  lastDisconnectReason: string | null;
  lastConnectedAt: number | null;
}

export interface WSDisconnectLike {
  code?: number;
  reason?: string;
  wasClean?: boolean;
}

export function createInitialWSConnectionMetrics(): WSConnectionMetrics {
  return {
    reconnectAttemptCount: 0,
    lastDisconnectReason: null,
    lastConnectedAt: null,
  };
}

export function recordWSConnected(
  metrics: WSConnectionMetrics,
  connectedAt: number = Date.now(),
): WSConnectionMetrics {
  return {
    ...metrics,
    lastConnectedAt: connectedAt,
  };
}

export function recordWSReconnectAttempt(metrics: WSConnectionMetrics): WSConnectionMetrics {
  return {
    ...metrics,
    reconnectAttemptCount: metrics.reconnectAttemptCount + 1,
  };
}

export function recordWSDisconnect(
  metrics: WSConnectionMetrics,
  event: WSDisconnectLike | string,
): WSConnectionMetrics {
  return {
    ...metrics,
    lastDisconnectReason: normaliseDisconnectReason(event),
  };
}

export function isStaleWSCallback(callbackGeneration: number, activeGeneration: number): boolean {
  return callbackGeneration !== activeGeneration;
}

function normaliseDisconnectReason(event: WSDisconnectLike | string): string {
  if (typeof event === 'string') {
    return event || 'unknown';
  }

  if (event.reason) {
    return event.reason;
  }

  if (event.code !== undefined) {
    const prefix = event.wasClean ? 'clean' : 'unclean';
    return `${prefix} close (${event.code})`;
  }

  return 'unknown';
}
