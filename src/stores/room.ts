import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Room, RoomMember, WsMessage } from "@/types";
import { showToast } from "@/utils/toast";

export const useRoomStore = defineStore("room", () => {
  const currentRoom = ref<Room | null>(null);
  const members = ref<Map<string, RoomMember>>(new Map());
  const ws = ref<WebSocket | null>(null);
  const connectionStatus = ref<"disconnected" | "connecting" | "connected">("disconnected");

  const isHost = computed(() => {
    const userId = localStorage.getItem("userId");
    return currentRoom.value?.hostId === userId;
  });

  const memberList = computed(() => Array.from(members.value.values()));

  const getServerUrl = () => {
    return localStorage.getItem("serverUrl") || "http://localhost:8080";
  };

  const getWsUrl = () => {
    const serverUrl = getServerUrl();
    return serverUrl.replace(/^http/, "ws");
  };

  const createRoom = async (name: string, password?: string): Promise<Room> => {
    const serverUrl = getServerUrl();
    const token = localStorage.getItem("token");

    const body: Record<string, string> = { name };
    if (password) body.password = password;

    const response = await fetch(`${serverUrl}/api/rooms`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const err = await response.json().catch(() => ({}));
      throw new Error(err.error || "Failed to create room");
    }

    const room = await response.json();
    currentRoom.value = room;
    return room;
  };

  const joinRoom = async (roomId: string, password?: string): Promise<Room> => {
    const serverUrl = getServerUrl();
    const token = localStorage.getItem("token");

    const body: Record<string, string> = {};
    if (password) body.password = password;

    const response = await fetch(`${serverUrl}/api/rooms/${roomId}/join`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const err = await response.json().catch(() => ({}));
      throw new Error(err.error || "Failed to join room");
    }

    const room = await response.json();
    currentRoom.value = room;
    return room;
  };

  const connectToRoom = async (roomId: string) => {
    intentionalClose = false;

    if (ws.value) {
      // 标记旧连接为有意关闭，避免触发重连
      intentionalClose = true;
      ws.value.close();
      ws.value = null;
      intentionalClose = false;
    }

    connectionStatus.value = "connecting";
    const userId = localStorage.getItem("userId") || "";
    const username = localStorage.getItem("username") || "匿名用户";
    const wsUrl = `${getWsUrl()}/ws/${roomId}?user_id=${encodeURIComponent(userId)}&username=${encodeURIComponent(username)}`;

    let connected = false;

    return new Promise<void>((resolve, reject) => {
      const socket = new WebSocket(wsUrl);

      socket.onopen = () => {
        connected = true;
        connectionStatus.value = "connected";
        reconnectAttempts = 0;
        ws.value = socket;
        startHeartbeat();
        resolve();
      };

      socket.onclose = () => {
        connectionStatus.value = "disconnected";
        ws.value = null;
        stopHeartbeat();
        // 仅在已成功连接过的情况下才自动重连
        if (connected) {
          attemptReconnect(roomId);
        }
      };

      socket.onerror = (error) => {
        connectionStatus.value = "disconnected";
        if (!connected) {
          reject(error);
        }
      };

      socket.onmessage = (event) => {
        try {
          const message: WsMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error("Failed to parse WebSocket message:", error);
        }
      };
    });
  };

  const handleMessage = (message: WsMessage) => {
    switch (message.type) {
      case "user_joined": {
        const msg = message as any;
        members.value.set(msg.user_id, {
          userId: msg.user_id,
          username: msg.username,
          isReady: false,
          isMuted: false,
        });
        break;
      }

      case "user_left": {
        const msg = message as any;
        members.value.delete(msg.user_id);
        break;
      }

      case "ready_state_changed": {
        const msg = message as any;
        const member = members.value.get(msg.user_id);
        if (member) {
          member.isReady = msg.is_ready;
        }
        break;
      }

      case "sync_broadcast":
        // 同步播放状态，由 player store 处理
        window.dispatchEvent(new CustomEvent("sync-broadcast", { detail: message }));
        break;

      case "chat_message":
        window.dispatchEvent(new CustomEvent("chat-message", { detail: message }));
        break;

      case "danmaku":
        window.dispatchEvent(new CustomEvent("danmaku", { detail: message }));
        break;

      case "video_hash":
        window.dispatchEvent(new CustomEvent("video-hash-check", { detail: message }));
        break;

      default:
        console.log("Unknown message type:", message);
    }
  };

  const sendMessage = (message: Record<string, unknown>) => {
    if (ws.value && ws.value.readyState === WebSocket.OPEN) {
      const userId = localStorage.getItem("userId") || "";
      const username = localStorage.getItem("username") || "匿名用户";
      
      ws.value.send(JSON.stringify({
        ...message,
        sender_id: userId,
        sender_name: username,
      }));
    }
  };

  // WebSocket 心跳 & 重连
  let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let reconnectAttempts = 0;
  const MAX_RECONNECT = 8;
  let intentionalClose = false;

  const startHeartbeat = () => {
    stopHeartbeat();
    heartbeatTimer = setInterval(() => {
      if (ws.value && ws.value.readyState === WebSocket.OPEN) {
        ws.value.send(JSON.stringify({ type: "ping" }));
      }
    }, 30000);
  };

  const stopHeartbeat = () => {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer);
      heartbeatTimer = null;
    }
  };

  const attemptReconnect = (roomId: string) => {
    if (intentionalClose) return;
    if (reconnectAttempts >= MAX_RECONNECT) {
      showToast("连接已断开，请返回重新加入房间", "error", 5000);
      return;
    }

    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
    reconnectAttempts++;
    showToast(`连接断开，${Math.round(delay / 1000)}秒后重连 (${reconnectAttempts}/${MAX_RECONNECT})`, "warning");

    reconnectTimer = setTimeout(() => {
      connectToRoom(roomId).catch(() => {
        // will retry via onclose
      });
    }, delay);
  };

  const leaveRoom = () => {
    intentionalClose = true;
    stopHeartbeat();
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (ws.value) {
      // 清除事件处理器防止触发 onclose 重连逻辑
      ws.value.onclose = null;
      ws.value.onerror = null;
      ws.value.onmessage = null;
      ws.value.close();
      ws.value = null;
    }
    currentRoom.value = null;
    members.value.clear();
    connectionStatus.value = "disconnected";
    reconnectAttempts = 0;
  };

  return {
    currentRoom,
    members,
    memberList,
    connectionStatus,
    isHost,
    createRoom,
    joinRoom,
    connectToRoom,
    sendMessage,
    leaveRoom,
  };
});