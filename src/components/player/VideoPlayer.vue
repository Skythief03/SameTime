<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { usePlayerStore } from "@/stores/player";
import { useRoomStore } from "@/stores/room";
import { getApiBaseUrl } from "@/platform";

const playerStore = usePlayerStore();
const roomStore = useRoomStore();

const containerRef = ref<HTMLDivElement | null>(null);
const showControls = ref(true);
const showSourcePicker = ref(false);
const magnetLink = ref("");
const serverFileUrl = ref("");
const uploadProgress = ref(0);
const isUploading = ref(false);
const hashMismatch = ref(false);
const mpvMissing = ref(false);
let controlsTimeout: number | null = null;

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
};

// 本地视频文件选择
const selectLocalVideo = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "视频文件",
          extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"],
        },
      ],
    });

    if (selected) {
      showSourcePicker.value = false;
      mpvMissing.value = false;
      await playerStore.loadVideo(selected as string);
      broadcastVideoHash();
    }
  } catch (error: any) {
    if (error?.message === "MPV_NOT_FOUND") {
      showSourcePicker.value = false;
      mpvMissing.value = true;
    } else {
      console.error("Failed to select video:", error);
    }
  }
};

// 上传视频到服务器
const uploadVideo = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "视频文件",
          extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"],
        },
      ],
    });

    if (!selected) return;

    isUploading.value = true;
    uploadProgress.value = 0;

    const serverUrl = getApiBaseUrl();
    const assetUrl = convertFileSrc(selected as string);
    const response = await fetch(assetUrl);
    const blob = await response.blob();

    const fileName = (selected as string).split(/[\\/]/).pop() || "video.mp4";
    const formData = new FormData();
    formData.append("file", blob, fileName);

    const xhr = new XMLHttpRequest();
    xhr.open("POST", `${serverUrl}/api/files/upload`);

    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable) {
        uploadProgress.value = Math.round((e.loaded / e.total) * 100);
      }
    };

    xhr.onload = async () => {
      isUploading.value = false;
      if (xhr.status === 200) {
        const result = JSON.parse(xhr.responseText);
        serverFileUrl.value = `${serverUrl}${result.url}`;
        showSourcePicker.value = false;
        // 上传成功后，用本地文件加载到播放器
        await playerStore.loadVideo(selected as string);
        broadcastVideoHash();
      }
    };

    xhr.onerror = () => {
      isUploading.value = false;
      console.error("Upload failed");
    };

    xhr.send(formData);
  } catch (error) {
    isUploading.value = false;
    console.error("Failed to upload video:", error);
  }
};

// 使用磁力链接（通知 aria2 下载）
const loadMagnetLink = async () => {
  if (!magnetLink.value.trim()) return;

  showSourcePicker.value = false;
  // aria2 集成：通过 Tauri invoke 调用 aria2 JSON-RPC
  // 暂时直接保存链接，后续 aria2 下载完成后加载
  console.log("Magnet link saved:", magnetLink.value);
  magnetLink.value = "";
};

const remoteUserHash = ref<string | null>(null);
const remoteUserName = ref<string>("");
const showMatchToast = ref(false);
let matchToastTimer: number | null = null;

// 执行 hash 比对的核心逻辑
const compareHash = () => {
  if (!remoteUserHash.value || !playerStore.videoHash) return;

  const mismatch = remoteUserHash.value !== playerStore.videoHash;
  hashMismatch.value = mismatch;

  if (!mismatch) {
    // 显示绿色一致提示，4 秒后自动消失
    showMatchToast.value = true;
    if (matchToastTimer) clearTimeout(matchToastTimer);
    matchToastTimer = window.setTimeout(() => {
      showMatchToast.value = false;
    }, 4000);
  }
};

// 广播视频 hash 给房间成员（结构化消息）
const broadcastVideoHash = () => {
  if (playerStore.videoHash && playerStore.videoPath) {
    const fileName = playerStore.videoPath.split(/[\\/]/).pop() || "unknown";
    roomStore.sendMessage({
      type: "video_hash",
      room_id: roomStore.currentRoom?.id || "",
      video_hash: playerStore.videoHash,
      file_name: fileName,
    });
    // 本地选择视频后，立即与已存储的远端 hash 比对
    compareHash();
  }
};

// 监听房间 hash 比对事件
const handleVideoHashCheck = (event: CustomEvent) => {
  const detail = event.detail;
  const senderId = detail?.sender_id;
  const userId = localStorage.getItem("userId");

  // 忽略自己发出的消息
  if (senderId === userId) return;

  remoteUserHash.value = detail?.video_hash || null;
  remoteUserName.value = detail?.sender_name || "其他用户";

  compareHash();
};

// 当本地 videoHash 变化（选择了新视频），重新比对
watch(() => playerStore.videoHash, () => {
  if (playerStore.videoHash && remoteUserHash.value) {
    compareHash();
  }
});

const dismissHashWarning = () => {
  hashMismatch.value = false;
};

const handleMouseMove = () => {
  showControls.value = true;
  
  if (controlsTimeout) {
    clearTimeout(controlsTimeout);
  }
  
  controlsTimeout = window.setTimeout(() => {
    if (playerStore.isPlaying) {
      showControls.value = false;
    }
  }, 3000);
};

const handleDoubleClick = () => {
  playerStore.toggleFullscreen();
};

onMounted(() => {
  window.addEventListener("video-hash-check", handleVideoHashCheck as EventListener);
});

onUnmounted(() => {
  window.removeEventListener("video-hash-check", handleVideoHashCheck as EventListener);
  if (controlsTimeout) {
    clearTimeout(controlsTimeout);
  }
  if (matchToastTimer) {
    clearTimeout(matchToastTimer);
  }
});
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full h-full bg-black flex items-center justify-center"
    @mousemove="handleMouseMove"
    @dblclick="handleDoubleClick"
  >
    <!-- Hash 比对结果 -->
    <div
      v-if="hashMismatch"
      class="absolute top-4 left-1/2 -translate-x-1/2 z-20 px-4 py-3 bg-yellow-600/90 rounded-lg text-sm flex items-center gap-3 max-w-lg"
    >
      <span>⚠️ 你的视频文件与 {{ remoteUserName }} 不一致（校验码不匹配），同步可能不准确</span>
      <button class="text-xs underline hover:text-white whitespace-nowrap" @click="showSourcePicker = true">重新选择</button>
      <button class="text-xs opacity-70 hover:opacity-100" @click="dismissHashWarning">✕</button>
    </div>
    <transition name="fade">
      <div
        v-if="showMatchToast"
        class="absolute top-4 left-1/2 -translate-x-1/2 z-20 px-4 py-2 bg-green-600/90 rounded-lg text-sm flex items-center gap-2"
      >
        ✅ 视频文件校验一致，可以同步观影
      </div>
    </transition>

    <!-- 上传进度条 -->
    <div v-if="isUploading" class="absolute top-4 left-1/2 -translate-x-1/2 z-20 px-4 py-2 bg-gray-800/90 rounded-lg text-sm">
      <div class="flex items-center gap-3">
        <span>上传中...</span>
        <div class="w-32 h-2 bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-primary-500 transition-all" :style="{ width: uploadProgress + '%' }"></div>
        </div>
        <span>{{ uploadProgress }}%</span>
      </div>
    </div>

    <!-- 无视频时的占位 -->
    <div
      v-if="mpvMissing"
      class="text-center max-w-md mx-auto p-6"
    >
      <div class="text-5xl mb-4">&#9888;&#65039;</div>
      <h3 class="text-lg font-medium mb-2">未检测到 MPV 播放器</h3>
      <p class="text-gray-400 text-sm mb-4">SameTime 需要 MPV 播放器来播放视频，请先安装：</p>
      <div class="text-left text-sm space-y-2 bg-gray-800 p-4 rounded-lg">
        <p><strong>macOS:</strong> <code class="bg-gray-700 px-1.5 py-0.5 rounded">brew install mpv</code></p>
        <p><strong>Linux:</strong> <code class="bg-gray-700 px-1.5 py-0.5 rounded">sudo apt install mpv</code></p>
        <p><strong>Windows:</strong> 从 mpv.io 下载安装并添加到系统 PATH</p>
      </div>
      <button
        class="mt-4 px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg text-sm transition-colors"
        @click="mpvMissing = false; showSourcePicker = true"
      >
        已安装，重试
      </button>
    </div>
    <div
      v-else-if="!playerStore.videoPath && !showSourcePicker"
      class="text-center"
    >
      <div class="text-6xl mb-4">🎬</div>
      <p class="text-gray-400 mb-4">暂未选择视频</p>
      <button
        class="btn btn-primary"
        @click="showSourcePicker = true"
      >
        选择视频源
      </button>
    </div>

    <!-- 视频源选择面板 -->
    <div
      v-if="showSourcePicker"
      class="text-center max-w-md mx-auto p-6"
    >
      <h3 class="text-lg font-medium mb-6">选择视频源</h3>

      <div class="space-y-3">
        <!-- 本地文件 -->
        <button
          class="w-full p-4 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center gap-4 transition-colors"
          @click="selectLocalVideo"
        >
          <span class="text-2xl">📁</span>
          <div class="text-left">
            <p class="font-medium">本地文件</p>
            <p class="text-xs text-gray-400">选择电脑上的视频文件</p>
          </div>
        </button>

        <!-- 上传到服务器 -->
        <button
          class="w-full p-4 bg-gray-800 hover:bg-gray-700 rounded-lg flex items-center gap-4 transition-colors"
          @click="uploadVideo"
        >
          <span class="text-2xl">☁️</span>
          <div class="text-left">
            <p class="font-medium">上传到服务器</p>
            <p class="text-xs text-gray-400">上传视频供所有成员下载</p>
          </div>
        </button>

        <!-- 磁力链接 -->
        <div class="w-full p-4 bg-gray-800 rounded-lg">
          <div class="flex items-center gap-4 mb-3">
            <span class="text-2xl">🧲</span>
            <div class="text-left">
              <p class="font-medium">磁力链接</p>
              <p class="text-xs text-gray-400">通过磁力链接下载视频</p>
            </div>
          </div>
          <div class="flex gap-2">
            <input
              v-model="magnetLink"
              type="text"
              placeholder="magnet:?xt=urn:btih:..."
              class="flex-1 px-3 py-2 bg-gray-700 rounded-lg text-sm border border-gray-600 focus:border-primary-500 transition-colors"
              @keyup.enter="loadMagnetLink"
            />
            <button
              class="btn btn-primary text-sm"
              :disabled="!magnetLink.trim()"
              @click="loadMagnetLink"
            >
              下载
            </button>
          </div>
        </div>
      </div>

      <button
        v-if="playerStore.videoPath"
        class="mt-4 text-sm text-gray-400 hover:text-white transition-colors"
        @click="showSourcePicker = false"
      >
        取消
      </button>
    </div>

    <!-- 视频播放区域（MPV 外部窗口覆盖此区域） -->
    <div
      v-else
      id="mpv-container"
      class="w-full h-full"
    >
      <!-- MPV 播放器窗口 -->
    </div>

    <!-- 悬浮控制层 -->
    <div
      v-if="playerStore.videoPath && showControls"
      class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent pointer-events-none"
    >
      <!-- 左上角文件信息 -->
      <div class="absolute top-3 left-3 pointer-events-auto text-xs text-gray-300 bg-black/50 rounded-lg px-3 py-2 max-w-xs" @click.stop>
        <p class="font-medium truncate">{{ playerStore.videoFileName }}</p>
        <p v-if="playerStore.videoFileSize" class="text-gray-400">
          {{ formatFileSize(playerStore.videoFileSize) }}
        </p>
        <p v-if="playerStore.videoHash" class="text-gray-500 font-mono">
          Hash: {{ playerStore.videoHash?.substring(0, 12) }}...
        </p>
      </div>

      <!-- 右上角更换视频按钮 -->
      <button
        class="absolute top-3 right-3 pointer-events-auto px-3 py-1.5 text-xs bg-gray-700/80 hover:bg-gray-600 rounded-lg transition-colors"
        @click.stop="showSourcePicker = true"
      >
        更换视频
      </button>

      <!-- 中央播放按钮 -->
      <button
        class="absolute inset-0 flex items-center justify-center pointer-events-auto"
        @click="playerStore.togglePlay"
      >
        <div
          v-if="!playerStore.isPlaying"
          class="w-20 h-20 rounded-full bg-white/20 backdrop-blur flex items-center justify-center hover:bg-white/30 transition-colors"
        >
          <span class="text-4xl ml-1">▶</span>
        </div>
      </button>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active {
  transition: opacity 0.3s ease;
}
.fade-leave-active {
  transition: opacity 0.6s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>