import { ref, onUnmounted } from "vue";

export interface SyncState {
  timestamp: number;
  isPlaying: boolean;
  lastSyncTime: number;
  senderId: string;
}

export function useSync(
  sendFn: (msg: Record<string, unknown>) => void,
  roomId: string,
  userId: string,
  isHost: boolean
) {
  const syncState = ref<SyncState>({
    timestamp: 0,
    isPlaying: false,
    lastSyncTime: 0,
    senderId: "",
  });

  const SYNC_THRESHOLD = 2.0; // seconds of drift before seeking
  const SYNC_INTERVAL = 5000; // periodic sync every 5s

  let syncTimer: number | null = null;

  // Send sync request (host only)
  const sendSync = (timestamp: number, isPlaying: boolean) => {
    if (!isHost) return;

    sendFn({
      type: "sync_request",
      room_id: roomId,
      timestamp,
      is_playing: isPlaying,
    });
  };

  // Handle incoming sync broadcast
  const handleSyncBroadcast = (
    data: { timestamp: number; is_playing: boolean; sender_id: string },
    localTimestamp: number,
    seekFn: (t: number) => void,
    setPlayingFn: (p: boolean) => void
  ) => {
    syncState.value = {
      timestamp: data.timestamp,
      isPlaying: data.is_playing,
      lastSyncTime: Date.now(),
      senderId: data.sender_id,
    };

    // Skip if we sent this message
    if (data.sender_id === userId) return;

    const drift = Math.abs(localTimestamp - data.timestamp);

    // Only seek if drift exceeds threshold
    if (drift > SYNC_THRESHOLD) {
      seekFn(data.timestamp);
    }

    setPlayingFn(data.is_playing);
  };

  // Start periodic sync heartbeat (host only)
  const startPeriodicSync = (
    getTimestampFn: () => number,
    getPlayingFn: () => boolean
  ) => {
    if (!isHost) return;

    syncTimer = window.setInterval(() => {
      sendSync(getTimestampFn(), getPlayingFn());
    }, SYNC_INTERVAL);
  };

  const stopPeriodicSync = () => {
    if (syncTimer) {
      clearInterval(syncTimer);
      syncTimer = null;
    }
  };

  onUnmounted(() => {
    stopPeriodicSync();
  });

  return {
    syncState,
    sendSync,
    handleSyncBroadcast,
    startPeriodicSync,
    stopPeriodicSync,
  };
}