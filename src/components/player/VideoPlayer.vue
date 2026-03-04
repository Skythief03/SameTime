<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { usePlayerStore } from "@/stores/player";
import { useRoomStore } from "@/stores/room";

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
let controlsTimeout: number | null = null;

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
      await playerStore.loadVideo(selected as string);
      broadcastVideoHash();
    }
  } catch (error) {
    console.error("Failed to select video:", error);
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

    const serverUrl = localStorage.getItem("serverUrl") || "http://localhost:8080";
    const response = await fetch(selected as string);
    const blob = await response.blob();

    const formData = new FormData();
    formData.append("file", blob, (selected as string).split("/").pop() || "video.mp4");

    const xhr = new XMLHttpRequest();
    xhr.open("POST", `${serverUrl}/api/files/upload`);

    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable) {
        uploadProgress.value = Math.round((e.loaded / e.total) * 100);
      }
    };

    xhr.onload = () => {
      isUploading.value = false;
      if (xhr.status === 200) {
        const result = JSON.parse(xhr.responseText);
        serverFileUrl.value = `${serverUrl}${result.url}`;
        showSourcePicker.value = false;
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
      v-if="!playerStore.videoPath && !showSourcePicker"
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

    <!-- 视频播放区域（MPV 将嵌入到这里） -->
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