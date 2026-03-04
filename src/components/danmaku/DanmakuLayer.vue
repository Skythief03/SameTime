<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { usePlayerStore } from "@/stores/player";
import type { Danmaku } from "@/types";

const playerStore = usePlayerStore();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);
const width = ref(800);
const height = ref(600);

interface ActiveDanmaku {
  danmaku: Danmaku;
  x: number;
  track: number;
  width: number;
}

interface Track {
  id: number;
  endTime: number;
}

const TRACK_HEIGHT = 32;
const SPEED = 150;
const FONT_SIZE = 24;

const tracks = ref<Track[]>([]);
const activeDanmaku = ref<Map<string, ActiveDanmaku>>(new Map());
const pendingDanmaku = ref<Danmaku[]>([]);

let animationId: number | null = null;
let lastFrameTime = 0;

const initTracks = () => {
  const trackCount = Math.floor(height.value / TRACK_HEIGHT);
  tracks.value = Array.from({ length: trackCount }, (_, i) => ({
    id: i,
    endTime: 0,
  }));
};

const getAvailableTrack = (startTime: number): number => {
  for (const track of tracks.value) {
    if (track.endTime <= startTime) {
      return track.id;
    }
  }
  return Math.floor(Math.random() * tracks.value.length);
};

const render = (timestamp: number) => {
  const canvas = canvasRef.value;
  if (!canvas) {
    animationId = requestAnimationFrame(render);
    return;
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    animationId = requestAnimationFrame(render);
    return;
  }

  // 计算 delta time
  const deltaTime = lastFrameTime ? (timestamp - lastFrameTime) / 1000 : 0;
  lastFrameTime = timestamp;

  // 清空画布
  ctx.clearRect(0, 0, width.value, height.value);
  ctx.font = `bold ${FONT_SIZE}px "Microsoft YaHei", "PingFang SC", sans-serif`;

  const now = performance.now() / 1000;
  const currentVideoTime = playerStore.currentTime;

  // 添加新弹幕
  for (const danmaku of pendingDanmaku.value) {
    if (
      !activeDanmaku.value.has(danmaku.id) &&
      Math.abs(danmaku.timestamp - currentVideoTime) < 0.5
    ) {
      const textWidth = ctx.measureText(danmaku.content).width;
      const track = getAvailableTrack(now);
      const duration = (width.value + textWidth) / SPEED;

      tracks.value[track].endTime = now + duration * 0.7;
      activeDanmaku.value.set(danmaku.id, {
        danmaku,
        x: width.value,
        track,
        width: textWidth,
      });
    }
  }

  // 渲染活跃弹幕
  for (const [id, item] of activeDanmaku.value) {
    // 只在播放时移动弹幕
    if (playerStore.isPlaying) {
      item.x -= SPEED * deltaTime;
    }

    // 移出屏幕则删除
    if (item.x < -item.width) {
      activeDanmaku.value.delete(id);
      continue;
    }

    const y = item.track * TRACK_HEIGHT + FONT_SIZE;

    // 描边
    ctx.strokeStyle = "#000000";
    ctx.lineWidth = 3;
    ctx.lineJoin = "round";
    ctx.strokeText(item.danmaku.content, item.x, y);

    // 填充
    ctx.fillStyle = item.danmaku.color || "#ffffff";
    ctx.fillText(item.danmaku.content, item.x, y);
  }

  animationId = requestAnimationFrame(render);
};

const handleDanmaku = (event: CustomEvent) => {
  const d = event.detail;
  const danmaku: Danmaku = {
    id: crypto.randomUUID(),
    userId: d.sender_id || d.senderId || "",
    username: d.sender_name || d.senderName || "匿名",
    content: d.content || "",
    timestamp: d.video_timestamp ?? d.videoTimestamp ?? 0,
    color: d.color || "#ffffff",
    type: "scroll",
    createdAt: Date.now(),
  };

  pendingDanmaku.value.push(danmaku);

  // 限制待处理弹幕数量
  if (pendingDanmaku.value.length > 500) {
    pendingDanmaku.value = pendingDanmaku.value.slice(-300);
  }
};

const updateSize = () => {
  if (containerRef.value) {
    width.value = containerRef.value.clientWidth;
    height.value = containerRef.value.clientHeight;
    initTracks();
  }
};

onMounted(() => {
  updateSize();
  window.addEventListener("resize", updateSize);
  window.addEventListener("danmaku", handleDanmaku as EventListener);
  animationId = requestAnimationFrame(render);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateSize);
  window.removeEventListener("danmaku", handleDanmaku as EventListener);
  if (animationId) {
    cancelAnimationFrame(animationId);
  }
});

watch([() => width.value, () => height.value], () => {
  initTracks();
});
</script>

<template>
  <div ref="containerRef" class="absolute inset-0 pointer-events-none overflow-hidden">
    <canvas
      ref="canvasRef"
      :width="width"
      :height="height"
      class="w-full h-full"
    />
  </div>
</template>