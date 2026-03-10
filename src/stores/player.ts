import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { track } from "@/utils/telemetry";
import { createPlayerAdapter } from "@/player/adapters";
import { mapErrorToPlayerError } from "@/player";

const adapter = createPlayerAdapter();

export const usePlayerStore = defineStore("player", () => {
  const currentTime = ref(0);
  const duration = ref(0);
  const isPlaying = ref(false);
  const volume = ref(100);
  const isFullscreen = ref(false);
  const videoPath = ref<string | null>(null);
  const videoHash = ref<string | null>(null);
  const videoFileSize = ref<number | null>(null);

  const videoFileName = computed(() => {
    if (!videoPath.value) return null;
    return videoPath.value.split(/[\\/]/).pop() || null;
  });

  const formatTime = (seconds: number): string => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  const formattedCurrentTime = computed(() => formatTime(currentTime.value));
  const formattedDuration = computed(() => formatTime(duration.value));

  const unsubscribeTime = adapter.onTimeUpdate((time, dur) => {
    currentTime.value = time;
    if (dur > 0) duration.value = dur;
  });

  const unsubscribeState = adapter.onStateChange((state) => {
    isPlaying.value = state.isPlaying;
    volume.value = state.volume;
    currentTime.value = state.currentTime;
    if (state.duration > 0) duration.value = state.duration;
  });

  const unsubscribeError = adapter.onError((error) => {
    track("player_error", { code: error.code, message: error.message });
  });

  const loadVideo = async (path: string) => {
    try {
      await adapter.checkAvailability();
    } catch {
      throw new Error("MPV_NOT_FOUND");
    }

    try {
      videoPath.value = path;
      duration.value = 0;
      currentTime.value = 0;

      await adapter.load(path);
      track("player_load", { source: path });

      videoHash.value = await adapter.calculateFileHash(path);
      videoFileSize.value = await adapter.getFileSize(path);
    } catch (error) {
      const mapped = mapErrorToPlayerError(error);
      track("player_error", { action: "load", code: mapped.code, message: mapped.message });
      videoPath.value = null;
      videoHash.value = null;
      videoFileSize.value = null;
      throw error;
    }
  };

  const togglePlay = async () => {
    try {
      const next = !isPlaying.value;
      if (next) {
        await adapter.play();
      } else {
        await adapter.pause();
      }
      track(next ? "player_play" : "player_pause", { action: "toggle" });
    } catch (error) {
      track("player_error", { action: "toggle_play", error: String(error) });
      console.error("Failed to toggle play:", error);
    }
  };

  const play = async () => {
    try {
      await adapter.play();
      track("player_play", { action: "play" });
    } catch (error) {
      track("player_error", { action: "play", error: String(error) });
      console.error("Failed to play:", error);
    }
  };

  const pause = async () => {
    try {
      await adapter.pause();
      track("player_pause", { action: "pause" });
    } catch (error) {
      track("player_error", { action: "pause", error: String(error) });
      console.error("Failed to pause:", error);
    }
  };

  const seek = async (position: number) => {
    try {
      await adapter.seek(position);
      currentTime.value = position;
      track("player_seek", { position });
    } catch (error) {
      track("player_error", { action: "seek", error: String(error), position });
      console.error("Failed to seek:", error);
    }
  };

  const setVolume = async (value: number) => {
    try {
      await adapter.setVolume(value);
      volume.value = value;
    } catch (error) {
      track("player_error", { action: "volume", error: String(error), value });
      console.error("Failed to set volume:", error);
    }
  };

  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      isFullscreen.value = true;
    } else {
      document.exitFullscreen();
      isFullscreen.value = false;
    }
  };

  const updateTime = (time: number) => {
    currentTime.value = time;
  };

  const updateDuration = (dur: number) => {
    duration.value = dur;
  };

  const syncTo = async (timestamp: number, playing: boolean) => {
    const drift = Math.abs(currentTime.value - timestamp);
    track("sync_drift", { drift, timestamp, current: currentTime.value });

    if (drift > 2) {
      await seek(timestamp);
    }

    if (playing !== isPlaying.value) {
      if (playing) {
        await play();
      } else {
        await pause();
      }
    }
  };

  const startPolling = () => {
    // polling moved into adapter
  };

  const stopPolling = () => {
    // polling moved into adapter
  };

  const cleanup = async () => {
    window.removeEventListener("sync-broadcast", handleSyncBroadcast as EventListener);
    try {
      await adapter.dispose();
    } catch (error) {
      track("player_error", { action: "cleanup", error: String(error) });
      console.error("Failed to dispose player adapter:", error);
    }

    videoPath.value = null;
    videoHash.value = null;
    videoFileSize.value = null;
    currentTime.value = 0;
    duration.value = 0;
    isPlaying.value = false;
  };

  const handleSyncBroadcast = (event: CustomEvent) => {
    const { timestamp, is_playing, sender_id } = event.detail;
    const userId = localStorage.getItem("userId");

    if (sender_id !== userId) {
      syncTo(timestamp, is_playing);
    }
  };

  window.addEventListener("sync-broadcast", handleSyncBroadcast as EventListener);

  const disposeListeners = () => {
    unsubscribeTime();
    unsubscribeState();
    unsubscribeError();
  };

  return {
    currentTime,
    duration,
    isPlaying,
    volume,
    isFullscreen,
    videoPath,
    videoHash,
    videoFileName,
    videoFileSize,
    formattedCurrentTime,
    formattedDuration,
    loadVideo,
    togglePlay,
    play,
    pause,
    seek,
    setVolume,
    toggleFullscreen,
    updateTime,
    updateDuration,
    syncTo,
    startPolling,
    stopPolling,
    cleanup,
    disposeListeners,
  };
});
