import { ref, onUnmounted } from "vue";
import type { WsMessage } from "@/types";
import { getWsBaseUrl } from "@/platform";

export type WsMessageHandler = (message: WsMessage) => void;

export function useWebSocket() {
  const ws = ref<WebSocket | null>(null);
  const status = ref<"disconnected" | "connecting" | "connected">("disconnected");
  const handlers = ref<Map<string, WsMessageHandler[]>>(new Map());

  let reconnectTimer: number | null = null;
  let reconnectAttempts = 0;
  const MAX_RECONNECT_ATTEMPTS = 10;
  const BASE_RECONNECT_DELAY = 1000;

  const connect = (roomId: string): Promise<void> => {
    return new Promise((resolve, reject) => {
      if (ws.value) {
        ws.value.close();
      }

      status.value = "connecting";
      const url = `${getWsBaseUrl()}/ws/${roomId}`;
      const socket = new WebSocket(url);

      socket.onopen = () => {
        status.value = "connected";
        reconnectAttempts = 0;
        ws.value = socket;
        resolve();
      };

      socket.onclose = () => {
        status.value = "disconnected";
        ws.value = null;
        attemptReconnect(roomId);
      };

      socket.onerror = (err) => {
        status.value = "disconnected";
        if (reconnectAttempts === 0) {
          reject(err);
        }
      };

      socket.onmessage = (event) => {
        try {
          const message: WsMessage = JSON.parse(event.data);
          dispatch(message);
        } catch (e) {
          console.error("Failed to parse WS message:", e);
        }
      };
    });
  };

  const attemptReconnect = (roomId: string) => {
    if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) return;

    const delay = BASE_RECONNECT_DELAY * Math.pow(2, Math.min(reconnectAttempts, 5));
    reconnectAttempts++;

    reconnectTimer = window.setTimeout(() => {
      connect(roomId).catch(() => {
        // Will retry via onclose
      });
    }, delay);
  };

  const disconnect = () => {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    reconnectAttempts = MAX_RECONNECT_ATTEMPTS; // Prevent reconnect
    if (ws.value) {
      ws.value.close();
      ws.value = null;
    }
    status.value = "disconnected";
  };

  const send = (message: Record<string, unknown>) => {
    if (ws.value && ws.value.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify(message));
    }
  };

  const on = (type: string, handler: WsMessageHandler) => {
    if (!handlers.value.has(type)) {
      handlers.value.set(type, []);
    }
    handlers.value.get(type)!.push(handler);
  };

  const off = (type: string, handler: WsMessageHandler) => {
    const list = handlers.value.get(type);
    if (list) {
      const idx = list.indexOf(handler);
      if (idx !== -1) list.splice(idx, 1);
    }
  };

  const dispatch = (message: WsMessage) => {
    const list = handlers.value.get(message.type);
    if (list) {
      for (const handler of list) {
        handler(message);
      }
    }
    // Also dispatch to wildcard listeners
    const wildcardList = handlers.value.get("*");
    if (wildcardList) {
      for (const handler of wildcardList) {
        handler(message);
      }
    }
  };

  onUnmounted(() => {
    disconnect();
  });

  return {
    ws,
    status,
    connect,
    disconnect,
    send,
    on,
    off,
  };
}