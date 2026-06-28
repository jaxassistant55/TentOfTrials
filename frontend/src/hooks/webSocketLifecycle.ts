export interface WSConnectionLifecycle<Timer = unknown> {
  mounted: boolean;
  generation: number;
  reconnectTimer: Timer | null;
}

export function createWSConnectionLifecycle<Timer = unknown>(): WSConnectionLifecycle<Timer> {
  return {
    mounted: true,
    generation: 0,
    reconnectTimer: null,
  };
}

export function markWSMounted<Timer>(lifecycle: WSConnectionLifecycle<Timer>): void {
  lifecycle.mounted = true;
}

export function beginWSConnection<Timer>(lifecycle: WSConnectionLifecycle<Timer>): number {
  lifecycle.generation += 1;
  return lifecycle.generation;
}

export function isActiveWSConnection<Timer>(
  lifecycle: WSConnectionLifecycle<Timer>,
  generation: number,
  socketIsCurrent: boolean,
): boolean {
  return lifecycle.mounted && socketIsCurrent && lifecycle.generation === generation;
}

export function setWSReconnectTimer<Timer>(
  lifecycle: WSConnectionLifecycle<Timer>,
  timer: Timer,
): Timer {
  lifecycle.reconnectTimer = timer;
  return timer;
}

export function clearWSReconnectTimer<Timer>(
  lifecycle: WSConnectionLifecycle<Timer>,
  clearTimer: (timer: Timer) => void,
): void {
  if (lifecycle.reconnectTimer !== null) {
    clearTimer(lifecycle.reconnectTimer);
    lifecycle.reconnectTimer = null;
  }
}

export function markWSReconnectTimerFired<Timer>(lifecycle: WSConnectionLifecycle<Timer>): void {
  lifecycle.reconnectTimer = null;
}

export function markWSUnmounted<Timer>(
  lifecycle: WSConnectionLifecycle<Timer>,
  clearTimer: (timer: Timer) => void,
): void {
  lifecycle.mounted = false;
  lifecycle.generation += 1;
  clearWSReconnectTimer(lifecycle, clearTimer);
}

export function isWSLifecycleMounted<Timer>(lifecycle: WSConnectionLifecycle<Timer>): boolean {
  return lifecycle.mounted;
}
