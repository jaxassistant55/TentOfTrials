/**
 * WebSocket reconnection metrics tracker
 *
 * Provides lightweight observable metrics for socket lifecycle testing and diagnostics.
 * Tracks reconnection attempts, disconnect reasons, and connection timestamps without
 * exposing sensitive data like tokens or full message payloads.
 */

import { useCallback, useRef, useState } from 'react';

export interface SocketMetrics {
  /** Number of reconnection attempts since component mount */
  reconnectAttemptCount: number;
  /** Reason for the last disconnect (e.g., 'clean', 'error', 'timeout', 'network') */
  lastDisconnectReason: string | null;
  /** ISO 8601 timestamp of the last successful connection, or null if never connected */
  lastConnectedTimestamp: string | null;
  /** Number of milliseconds since last successful connection, or null if never connected */
  timeSinceLastConnection: number | null;
}

export interface SocketMetricsCallbacks {
  /** Called when a reconnection attempt begins */
  onReconnectAttempt?: (attemptNumber: number, reason: string) => void;
  /** Called when a clean disconnect occurs */
  onCleanDisconnect?: () => void;
  /** Called when connection is successfully established */
  onConnected?: (timestamp: string) => void;
}

/**
 * Hook for tracking WebSocket connection metrics
 *
 * Usage:
 *   const metrics = useSocketMetrics({
 *     onReconnectAttempt: (attempt) => console.log(`Reconnecting (attempt ${attempt})`),
 *   });
 *
 *   // Call these methods when socket lifecycle events occur:
 *   metrics.recordConnected();
 *   metrics.recordReconnectAttempt('lost connection');
 *   metrics.recordDisconnect('clean', true); // true = manual disconnect
 */
export function useSocketMetrics(callbacks?: SocketMetricsCallbacks) {
  const [metrics, setMetrics] = useState<SocketMetrics>({
    reconnectAttemptCount: 0,
    lastDisconnectReason: null,
    lastConnectedTimestamp: null,
    timeSinceLastConnection: null,
  });

  const timestampIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Update the time since last connection on a regular interval
  const startTimestampTimer = useCallback(() => {
    if (timestampIntervalRef.current) {
      clearInterval(timestampIntervalRef.current);
    }
    timestampIntervalRef.current = setInterval(() => {
      setMetrics(prev => {
        if (!prev.lastConnectedTimestamp) {
          return prev;
        }
        const timeSince = Date.now() - new Date(prev.lastConnectedTimestamp).getTime();
        return {
          ...prev,
          timeSinceLastConnection: timeSince,
        };
      });
    }, 1000);
  }, []);

  const stopTimestampTimer = useCallback(() => {
    if (timestampIntervalRef.current) {
      clearInterval(timestampIntervalRef.current);
      timestampIntervalRef.current = null;
    }
  }, []);

  const recordConnected = useCallback(() => {
    const timestamp = new Date().toISOString();
    setMetrics(prev => ({
      ...prev,
      lastConnectedTimestamp: timestamp,
      timeSinceLastConnection: 0,
      reconnectAttemptCount: 0, // Reset attempt count on successful connection
    }));
    callbacks?.onConnected?.(timestamp);
    startTimestampTimer();
  }, [callbacks, startTimestampTimer]);

  const recordReconnectAttempt = useCallback((reason: string) => {
    setMetrics(prev => {
      const newCount = prev.reconnectAttemptCount + 1;
      callbacks?.onReconnectAttempt?.(newCount, reason);
      return {
        ...prev,
        reconnectAttemptCount: newCount,
        lastDisconnectReason: reason,
      };
    });
  }, [callbacks]);

  const recordDisconnect = useCallback((reason: string, isManual: boolean = false) => {
    stopTimestampTimer();
    setMetrics(prev => ({
      ...prev,
      lastDisconnectReason: isManual ? 'clean' : reason,
    }));
    if (isManual) {
      callbacks?.onCleanDisconnect?.();
    }
  }, [callbacks, stopTimestampTimer]);

  const reset = useCallback(() => {
    stopTimestampTimer();
    setMetrics({
      reconnectAttemptCount: 0,
      lastDisconnectReason: null,
      lastConnectedTimestamp: null,
      timeSinceLastConnection: null,
    });
  }, [stopTimestampTimer]);

  return {
    metrics,
    recordConnected,
    recordReconnectAttempt,
    recordDisconnect,
    reset,
  };
}
