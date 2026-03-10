import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

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

  // 格式化时间显示
  const formatTime = (seconds: number): string => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);

    if (h > 0) {
      return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
    }
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  const formattedCurrentTime = computed(() => formatTime(currentTime.value));
  const formattedDuration = computed(() => formatTime(duration.value));

  // 加载视频
  const loadVideo = async (path: string) => {
    try {
      videoPath.value = path;
      await invoke("mpv_play", { filePath: path });

      // 计算文件 hash
      videoHash.value = await invoke<string>("calculate_file_hash", { filePath: path });

      // 获取文件大小
      videoFileSize.value = await invoke<number>("get_file_size", { filePath: path });

      // 开始轮询播放位置
      startPolling();
    } catch (error: any) {
      videoPath.value = null;
      videoHash.value = null;
      videoFileSize.value = null;
      const errorStr = String(error);
      if (errorStr.includes("Failed to start mpv") || errorStr.includes("not found")) {
        throw new Error("MPV_NOT_FOUND");
      }
      throw error;
    }
  };

  // 播放/暂停切换
  const togglePlay = async () => {
    try {
      const newState = !isPlaying.value;
      await invoke("mpv_set_pause", { paused: !newState });
      isPlaying.value = newState;
    } catch (error) {
      console.error("Failed to toggle play:", error);
    }
  };

  // 播放
  const play = async () => {
    try {
      await invoke("mpv_set_pause", { paused: false });
      isPlaying.value = true;
    } catch (error) {
      console.error("Failed to play:", error);
    }
  };

  // 暂停
  const pause = async () => {
    try {
      await invoke("mpv_set_pause", { paused: true });
      isPlaying.value = false;
    } catch (error) {
      console.error("Failed to pause:", error);
    }
  };

  // 跳转
  const seek = async (position: number) => {
    try {
      await invoke("mpv_seek", { position });
      currentTime.value = position;
    } catch (error) {
      console.error("Failed to seek:", error);
    }
  };

  // 设置音量
  const setVolume = async (value: number) => {
    try {
      await invoke("mpv_set_volume", { volume: value });
      volume.value = value;
    } catch (error) {
      console.error("Failed to set volume:", error);
    }
  };

  // 全屏切换
  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      isFullscreen.value = true;
    } else {
      document.exitFullscreen();
      isFullscreen.value = false;
    }
  };

  // 更新播放时间（由 mpv 事件触发）
  const updateTime = (time: number) => {
    currentTime.value = time;
  };

  // 更新时长
  const updateDuration = (dur: number) => {
    duration.value = dur;
  };

  // 同步到指定状态（被远程同步调用）
  const syncTo = async (timestamp: number, playing: boolean) => {
    const drift = Math.abs(currentTime.value - timestamp);
    
    // 如果漂移超过 2 秒，执行跳转
    if (drift > 2) {
      await seek(timestamp);
    }

    // 同步播放状态
    if (playing !== isPlaying.value) {
      if (playing) {
        await play();
      } else {
        await pause();
      }
    }
  };

  // 轮询 mpv 播放位置
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  const startPolling = () => {
    if (pollTimer) return;
    pollTimer = setInterval(async () => {
      if (!videoPath.value) return;
      try {
        const pos = await invoke<number>("mpv_get_position");
        if (typeof pos === "number" && !isNaN(pos)) {
          currentTime.value = pos;
        }
      } catch {
        // mpv may not be ready yet, ignore
      }
    }, 500);
  };

  const stopPolling = () => {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  };

  // 清理资源
  const cleanup = async () => {
    stopPolling();
    // 移除同步事件监听器
    window.removeEventListener("sync-broadcast", handleSyncBroadcast as EventListener);
    try {
      await invoke("mpv_stop");
    } catch (error) {
      console.error("Failed to stop mpv:", error);
    }

    videoPath.value = null;
    videoHash.value = null;
    videoFileSize.value = null;
    currentTime.value = 0;
    duration.value = 0;
    isPlaying.value = false;
  };

  // 监听同步事件
  const handleSyncBroadcast = (event: CustomEvent) => {
    const { timestamp, is_playing, sender_id } = event.detail;
    const userId = localStorage.getItem("userId");
    
    // 不处理自己发送的同步消息
    if (sender_id !== userId) {
      syncTo(timestamp, is_playing);
    }
  };

  // 注册事件监听
  window.addEventListener("sync-broadcast", handleSyncBroadcast as EventListener);

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
  };
});