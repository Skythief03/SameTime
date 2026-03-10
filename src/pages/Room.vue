<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useRoomStore } from "@/stores/room";
import { usePlayerStore } from "@/stores/player";
import { showToast } from "@/utils/toast";
import VideoPlayer from "@/components/player/VideoPlayer.vue";
import ChatPanel from "@/components/chat/ChatPanel.vue";
import MemberList from "@/components/room/MemberList.vue";
import DanmakuLayer from "@/components/danmaku/DanmakuLayer.vue";

const route = useRoute();
const router = useRouter();
const roomStore = useRoomStore();
const playerStore = usePlayerStore();

const roomId = computed(() => route.params.id as string);
const showSidebar = ref(true);
const showDanmaku = ref(true);
const showVolumeSlider = ref(false);
const danmakuInput = ref("");

// 语音状态
const voiceActive = ref(false);
const voiceMuted = ref(false);
let localStream: MediaStream | null = null;

// 同步心跳
let syncTimer: ReturnType<typeof setInterval> | null = null;

onMounted(async () => {
  try {
    await roomStore.connectToRoom(roomId.value);
    showToast("已连接到房间", "success");
    startSyncHeartbeat();
  } catch (error: any) {
    showToast(error?.message || "连接房间失败，正在返回首页", "error", 4000);
    setTimeout(() => router.push("/"), 1500);
  }
});

onUnmounted(() => {
  stopSyncHeartbeat();
  stopVoice();
  roomStore.leaveRoom();
  playerStore.cleanup();
});

// 监听房主状态变化
watch(() => roomStore.isHost, (isHost) => {
  if (isHost) {
    startSyncHeartbeat();
  } else {
    stopSyncHeartbeat();
  }
});

const startSyncHeartbeat = () => {
  if (syncTimer) return;
  syncTimer = setInterval(() => {
    if (roomStore.isHost && playerStore.videoPath) {
      roomStore.sendMessage({
        type: "sync_request",
        room_id: roomId.value,
        timestamp: playerStore.currentTime,
        is_playing: playerStore.isPlaying,
      });
    }
  }, 5000);
};

const stopSyncHeartbeat = () => {
  if (syncTimer) {
    clearInterval(syncTimer);
    syncTimer = null;
  }
};

// 房主操作时立即发送同步
const handleTogglePlay = async () => {
  await playerStore.togglePlay();
  if (roomStore.isHost) {
    roomStore.sendMessage({
      type: "sync_request",
      room_id: roomId.value,
      timestamp: playerStore.currentTime,
      is_playing: playerStore.isPlaying,
    });
  }
};

const handleSeek = async (position: number) => {
  await playerStore.seek(position);
  if (roomStore.isHost) {
    roomStore.sendMessage({
      type: "sync_request",
      room_id: roomId.value,
      timestamp: position,
      is_playing: playerStore.isPlaying,
    });
  }
};

// 语音控制
const toggleVoice = async () => {
  if (voiceActive.value) {
    stopVoice();
  } else {
    try {
      localStream = await navigator.mediaDevices.getUserMedia({
        audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
        video: false,
      });
      voiceActive.value = true;
      voiceMuted.value = false;
    } catch (err) {
      console.error("Failed to get microphone:", err);
    }
  }
};

const toggleVoiceMute = () => {
  if (localStream) {
    localStream.getAudioTracks().forEach(t => {
      t.enabled = voiceMuted.value;
    });
    voiceMuted.value = !voiceMuted.value;
  }
};

const stopVoice = () => {
  if (localStream) {
    localStream.getTracks().forEach(t => t.stop());
    localStream = null;
  }
  voiceActive.value = false;
  voiceMuted.value = false;
};

// 弹幕发送
const sendDanmaku = () => {
  if (!danmakuInput.value.trim()) return;
  roomStore.sendMessage({
    type: "danmaku",
    room_id: roomId.value,
    content: danmakuInput.value.trim(),
    video_timestamp: playerStore.currentTime,
    color: "#ffffff",
  });
  danmakuInput.value = "";
};

const handleLeaveRoom = () => {
  roomStore.leaveRoom();
  router.push("/");
};

const toggleSidebar = () => {
  showSidebar.value = !showSidebar.value;
};
</script>

<template>
  <div class="h-screen flex flex-col bg-gray-900">
    <!-- 顶部栏 -->
    <header class="h-14 flex items-center justify-between px-4 bg-gray-800 border-b border-gray-700">
      <div class="flex items-center gap-4">
        <button
          class="text-gray-400 hover:text-white transition-colors"
          @click="handleLeaveRoom"
        >
          ← 离开
        </button>
        <div>
          <h1 class="font-medium">{{ roomStore.currentRoom?.name || '房间' }}</h1>
          <p class="text-xs text-gray-400">房间号: {{ roomId }}</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- 语音通话 -->
        <button
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          :class="{ 'text-green-400': voiceActive && !voiceMuted, 'text-red-400': voiceActive && voiceMuted, 'text-gray-400': !voiceActive }"
          @click="toggleVoice"
          :title="voiceActive ? '关闭语音' : '开启语音'"
        >
          🎤
        </button>
        <button
          v-if="voiceActive"
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          :class="{ 'text-red-400': voiceMuted, 'text-gray-400': !voiceMuted }"
          @click="toggleVoiceMute"
          :title="voiceMuted ? '取消静音' : '静音'"
        >
          {{ voiceMuted ? '🔇' : '🔊' }}
        </button>

        <div class="w-px h-6 bg-gray-700 mx-1"></div>

        <button
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          :class="{ 'text-primary-400': showDanmaku, 'text-gray-400': !showDanmaku }"
          @click="showDanmaku = !showDanmaku"
          title="弹幕开关"
        >
          💬
        </button>
        <button
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          :class="{ 'text-primary-400': showSidebar, 'text-gray-400': !showSidebar }"
          @click="toggleSidebar"
          title="侧边栏"
        >
          📋
        </button>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="flex-1 flex overflow-hidden">
      <!-- 视频区域 -->
      <div class="flex-1 relative bg-black">
        <VideoPlayer />
        <DanmakuLayer v-if="showDanmaku" />
      </div>

      <!-- 侧边栏 -->
      <aside
        v-show="showSidebar"
        class="w-80 flex flex-col min-h-0 bg-gray-800 border-l border-gray-700"
      >
        <!-- 成员列表 -->
        <MemberList class="border-b border-gray-700" />
        
        <!-- 聊天面板 -->
        <ChatPanel class="flex-1 min-h-0" />
      </aside>
    </main>

    <!-- 底部控制栏 -->
    <footer class="h-16 flex items-center justify-between px-4 bg-gray-800 border-t border-gray-700">
      <!-- 播放控制 -->
      <div class="flex items-center gap-4">
        <button
          class="p-2 rounded-full hover:bg-gray-700 transition-colors"
          @click="handleTogglePlay"
        >
          {{ playerStore.isPlaying ? '⏸️' : '▶️' }}
        </button>
        
        <div class="text-sm text-gray-400">
          {{ playerStore.formattedCurrentTime }} / {{ playerStore.formattedDuration }}
        </div>
      </div>

      <!-- 进度条 -->
      <div class="flex-1 mx-4">
        <input
          type="range"
          :value="playerStore.currentTime"
          :max="playerStore.duration || 100"
          step="0.1"
          class="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          @input="handleSeek(($event.target as HTMLInputElement).valueAsNumber)"
        />
      </div>

      <!-- 弹幕输入 -->
      <div class="flex items-center gap-2 mr-4">
        <input
          v-model="danmakuInput"
          type="text"
          placeholder="发送弹幕..."
          class="w-40 px-3 py-1.5 text-sm bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
          @keyup.enter="sendDanmaku"
        />
      </div>

      <!-- 右侧控制 -->
      <div class="flex items-center gap-2 relative">
        <!-- 音量控制 -->
        <div class="relative" @mouseenter="showVolumeSlider = true" @mouseleave="showVolumeSlider = false">
          <button class="p-2 rounded-lg hover:bg-gray-700 transition-colors">
            {{ playerStore.volume > 0 ? '🔊' : '🔇' }}
          </button>
          <div
            v-show="showVolumeSlider"
            class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 p-3 bg-gray-800 rounded-lg border border-gray-700 shadow-xl"
          >
            <input
              type="range"
              :value="playerStore.volume"
              min="0"
              max="100"
              class="w-24 h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
              @input="playerStore.setVolume(($event.target as HTMLInputElement).valueAsNumber)"
            />
          </div>
        </div>
        
        <!-- 全屏 -->
        <button
          class="p-2 rounded-lg hover:bg-gray-700 transition-colors"
          @click="playerStore.toggleFullscreen"
        >
          ⛶
        </button>
      </div>
    </footer>
  </div>
</template>