export type TelemetryEvent =
  | "player_load"
  | "player_play"
  | "player_pause"
  | "player_seek"
  | "player_error"
  | "ws_reconnect_attempt"
  | "ws_reconnect_success"
  | "ws_reconnect_fail"
  | "sync_drift";

const DEBUG_KEY = "sametime_debug";

const isTelemetryEnabled = (): boolean => {
  if (typeof window === "undefined") return false;
  return import.meta.env.DEV || localStorage.getItem(DEBUG_KEY) === "1";
};

export const track = (event: TelemetryEvent, payload: Record<string, unknown> = {}): void => {
  if (!isTelemetryEnabled()) return;
  console.info("[telemetry]", event, payload);
};
