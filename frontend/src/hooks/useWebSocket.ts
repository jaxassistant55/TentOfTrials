import { useCallback, useEffect, useRef, useState } from 'react';

// ---------------------------------------------------------------------------
// TYPES
// ---------------------------------------------------------------------------

export interface WSMessage {
  type: string;
  channel?: string;
  payload: unknown;
  id?: string;
  timestamp?: number;
}

export interface WSSubscription {
  channel: string;
  filter?: Record<string, unknown>;
  callback: (data: unknown) => void;
}

export type WSConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error';

export interface WSOptions {
  url: string;
  protocols?: string | string[];
  autoConnect?: boolean;
  reconnect?: boolean;
  maxReconnectAttempts?: number;
  reconnectBaseDelay?: number;
  reconnectMaxDelay?: number;
  reconnectJitter?: number;
  pingInterval?: number;
  pongTimeout?: number;
  messageQueueSize?: number;
  debug?: boolean;
  onOpen?: (event: Event) => void;
  onClose?: (event: CloseEvent) => void;
  onError?: (event: Event) => void;
  onMessage?: (message: WSMessage) => void;
}

export interface WSState {
  connectionState: WSConnectionState;
  lastMessage: WSMessage | null;
  reconnectAttempt: number;
  queueSize: number;
  subscriptions: number;
  totalMessagesSent: number;
  totalMessagesReceived: number;
  errors: number;
  latencyMs: number | null;
  /** Timestamp (ms) of the most recent successful connection. Null if never connected. */
  lastConnectedAt: number | null;
  /** Reason for the most recent disconnect. Sanitized — never contains raw payloads. */
  lastDisconnectReason: string | null;
}

export interface WSConnectionMetrics {
  /** Number of reconnection attempts since the last successful connection. */
  reconnectAttempt: number;
  /** Timestamp (ms) of the most recent successful connection. */
  lastConnectedAt: number | null;
  /** Sanitized reason for the most recent disconnect. */
  lastDisconnectReason: string | null;
  /** Total successful connections made (cumulative). */
  totalConnections: number;
  /** Total disconnects (clean or otherwise). */
  totalDisconnects: number;
}

interface QueuedMessage {
  message: WSMessage;
  timestamp: number;
  retries: number;
}

const DEFAULT_OPTIONS: Required<Omit<WSOptions, 'url' | 'protocols' | 'onOpen' | 'onClose' | 'onError' | 'onMessage'>> = {
  autoConnect: true,
  reconnect: true,
  maxReconnectAttempts: 10,
  reconnectBaseDelay: 1000,
  reconnectMaxDelay: 30000,
  reconnectJitter: 1000,
  pingInterval: 30000,
  pongTimeout: 10000,
  messageQueueSize: 100,
  debug: false,
};

/** Sanitize a close reason to avoid leaking sensitive data in metrics. */
function sanitizeCloseReason(reason: string | undefined, code: number): string {
  if (!reason || reason.length === 0) {
    return `code:${code}`;
  }
  // Strip any long text that might contain payloads or tokens
  const trimmed = reason.slice(0, 64);
  // Mask anything that looks like a secret or token
  const masked = trimmed.replace(/[a-zA-Z0-9_-]{24,}/g, '[REDACTED]');
  return masked || `code:${code}`;
}

export function useWebSocket(options: WSOptions) {
  const mergedOptions = { ...DEFAULT_OPTIONS, ...options };
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pingTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pongTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const messageQueueRef = useRef<QueuedMessage[]>([]);
  const subscriptionsRef = useRef<Map<string, WSSubscription>>(new Map());
  const reconnectAttemptRef = useRef(0);
  const mountedRef = useRef(true);
  const messageIdRef = useRef(0);
  const pingStartRef = useRef(0);

  // Metrics refs (not in React state to avoid unnecessary re-renders)
  const totalConnectionsRef = useRef(0);
  const totalDisconnectsRef = useRef(0);
  const lastDisconnectReasonRef = useRef<string | null>(null);
  const lastConnectedAtRef = useRef<number | null>(null);

  const [state, setState] = useState<WSState>({
    connectionState: 'disconnected',
    lastMessage: null,
    reconnectAttempt: 0,
    queueSize: 0,
    subscriptions: 0,
    totalMessagesSent: 0,
    totalMessagesReceived: 0,
    errors: 0,
    latencyMs: null,
    lastConnectedAt: null,
    lastDisconnectReason: null,
  });

  const updateState = useCallback((partial: Partial<WSState>) => {
    setState(prev => ({ ...prev, ...partial }));
  }, []);

  const getConnectionMetrics = useCallback((): WSConnectionMetrics => ({
    reconnectAttempt: reconnectAttemptRef.current,
    lastConnectedAt: lastConnectedAtRef.current,
    lastDisconnectReason: lastDisconnectReasonRef.current,
    totalConnections: totalConnectionsRef.current,
    totalDisconnects: totalDisconnectsRef.current,
  }), []);

  const sendMessage = useCallback((message: WSMessage) => {
    const ws = wsRef.current;
    const msgStr = JSON.stringify(message);

    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(msgStr);
      updateState({ totalMessagesSent: state.totalMessagesSent + 1 });
    } else {
      if (messageQueueRef.current.length < mergedOptions.messageQueueSize) {
        messageQueueRef.current.push({ message, timestamp: Date.now(), retries: 0 });
        updateState({ queueSize: messageQueueRef.current.length });
      } else if (mergedOptions.debug) {
        console.warn('[WS] Message queue full, dropping message:', message.type);
      }
    }
  }, [mergedOptions, updateState, state.totalMessagesSent]);

  const subscribe = useCallback((subscription: WSSubscription) => {
    subscriptionsRef.current.set(subscription.channel, subscription);
    updateState({ subscriptions: subscriptionsRef.current.size });

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      sendMessage({
        type: 'subscribe',
        channel: subscription.channel,
        payload: subscription.filter || {},
      });
    }
  }, [sendMessage, updateState]);

  const unsubscribe = useCallback((channel: string) => {
    subscriptionsRef.current.delete(channel);
    updateState({ subscriptions: subscriptionsRef.current.size });

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      sendMessage({ type: 'unsubscribe', channel, payload: null });
    }
  }, [sendMessage, updateState]);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN || wsRef.current?.readyState === WebSocket.CONNECTING) {
      return;
    }

    updateState({ connectionState: 'connecting', reconnectAttempt: reconnectAttemptRef.current });

    try {
      const ws = new WebSocket(mergedOptions.url, mergedOptions.protocols);
      wsRef.current = ws;

      ws.onopen = (event) => {
        if (!mountedRef.current) return;
        reconnectAttemptRef.current = 0;
        totalConnectionsRef.current++;
        const now = Date.now();
        lastConnectedAtRef.current = now;
        updateState({
          connectionState: 'connected',
          reconnectAttempt: 0,
          lastConnectedAt: now,
        });

        subscriptionsRef.current.forEach((sub, channel) => {
          sendMessage({ type: 'subscribe', channel, payload: sub.filter || {} });
        });

        while (messageQueueRef.current.length > 0) {
          const queued = messageQueueRef.current.shift()!;
          sendMessage(queued.message);
        }
        updateState({ queueSize: 0 });

        startPing();
        mergedOptions.onOpen?.(event);
      };

      ws.onmessage = (event) => {
        if (!mountedRef.current) return;
        try {
          const message: WSMessage = JSON.parse(event.data);
          if (message.type === 'pong') {
            const latency = Date.now() - pingStartRef.current;
            updateState({ latencyMs: latency });
            clearPongTimeout();
            return;
          }
          updateState({
            lastMessage: message,
            totalMessagesReceived: state.totalMessagesReceived + 1,
          });
          if (message.channel) {
            const sub = subscriptionsRef.current.get(message.channel);
            if (sub) {
              try {
                sub.callback(message.payload);
              } catch (err) {
                if (mergedOptions.debug) {
                  console.error(`[WS] Subscriber error for channel ${message.channel}:`, err);
                }
              }
            }
          }
          mergedOptions.onMessage?.(message);
        } catch (err) {
          if (mergedOptions.debug) {
            console.error('[WS] Failed to parse message:', err);
          }
        }
      };

      ws.onclose = (event) => {
        if (!mountedRef.current) return;
        wsRef.current = null;
        stopPing();
        totalDisconnectsRef.current++;
        const sanitizedReason = sanitizeCloseReason(event.reason, event.code);
        lastDisconnectReasonRef.current = sanitizedReason;
        updateState({
          connectionState: 'disconnected',
          lastDisconnectReason: sanitizedReason,
        });
        mergedOptions.onClose?.(event);
        scheduleReconnect();
      };

      ws.onerror = (event) => {
        if (!mountedRef.current) return;
        updateState(prev => ({ ...prev, errors: prev.errors + 1, connectionState: 'error' }));
        mergedOptions.onError?.(event);
      };
    } catch (err) {
      if (!mountedRef.current) return;
      updateState(prev => ({ ...prev, errors: prev.errors + 1, connectionState: 'error' }));
      if (mergedOptions.debug) {
        console.error('[WS] Connection error:', err);
      }
      scheduleReconnect();
    }
  }, [mergedOptions, sendMessage, updateState, state.totalMessagesReceived]);

  const disconnect = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close(1000, 'Client disconnect');
      wsRef.current = null;
    }
    stopPing();
    updateState({ connectionState: 'disconnected', reconnectAttempt: 0 });
  }, [updateState]);

  const scheduleReconnect = useCallback(() => {
    if (!mergedOptions.reconnect || reconnectAttemptRef.current >= mergedOptions.maxReconnectAttempts) {
      updateState({ connectionState: 'error' });
      return;
    }
    const delay = Math.min(
      mergedOptions.reconnectBaseDelay * Math.pow(2, reconnectAttemptRef.current),
      mergedOptions.reconnectMaxDelay
    ) + Math.random() * mergedOptions.reconnectJitter;
    reconnectAttemptRef.current++;
    if (mergedOptions.debug) {
      console.log(`[WS] Reconnecting in ${Math.round(delay)}ms (attempt ${reconnectAttemptRef.current})`);
    }
    updateState({ connectionState: 'reconnecting', reconnectAttempt: reconnectAttemptRef.current });
    reconnectTimerRef.current = setTimeout(() => {
      if (mountedRef.current) connect();
    }, delay);
  }, [mergedOptions, connect, updateState]);

  const startPing = useCallback(() => {
    stopPing();
    pingTimerRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        pingStartRef.current = Date.now();
        wsRef.current.send(JSON.stringify({ type: 'ping' }));
        pongTimerRef.current = setTimeout(() => {
          if (mergedOptions.debug) {
            console.warn('[WS] Pong timeout, closing connection');
          }
          updateState({ latencyMs: null });
          wsRef.current?.close(4000, 'Pong timeout');
        }, mergedOptions.pongTimeout);
      }
    }, mergedOptions.pingInterval);
  }, [mergedOptions, updateState]);

  const stopPing = useCallback(() => {
    if (pingTimerRef.current) {
      clearInterval(pingTimerRef.current);
      pingTimerRef.current = null;
    }
    clearPongTimeout();
  }, []);

  const clearPongTimeout = useCallback(() => {
    if (pongTimerRef.current) {
      clearTimeout(pongTimerRef.current);
      pongTimerRef.current = null;
    }
  }, []);

  const send = useCallback((type: string, payload: unknown, channel?: string) => {
    const id = `msg_${++messageIdRef.current}`;
    sendMessage({ id, type, channel, payload, timestamp: Date.now() });
    return id;
  }, [sendMessage]);

  useEffect(() => {
    mountedRef.current = true;
    if (mergedOptions.autoConnect) {
      connect();
    }
    return () => {
      mountedRef.current = false;
      disconnect();
    };
  }, []);

  return {
    ...state,
    connect,
    disconnect,
    send,
    subscribe,
    unsubscribe,
    isConnected: wsRef.current?.readyState === WebSocket.OPEN,
    getConnectionMetrics,
  };
}
