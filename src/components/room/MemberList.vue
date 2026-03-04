<script setup lang="ts">
import { computed } from "vue";
import { useRoomStore } from "@/stores/room";

const roomStore = useRoomStore();

const members = computed(() => roomStore.memberList);

const getStatusIcon = (member: { isReady: boolean; isMuted: boolean }) => {
  if (member.isMuted) return "🔇";
  if (member.isReady) return "✓";
  return "⏳";
};

const getStatusClass = (member: { isReady: boolean }) => {
  return member.isReady ? "text-green-400" : "text-yellow-400";
};
</script>

<template>
  <div class="p-3">
    <div class="flex items-center justify-between mb-3">
      <h3 class="font-medium text-sm text-gray-300">
        房间成员 ({{ members.length }})
      </h3>
    </div>

    <div class="space-y-2 max-h-40 overflow-y-auto">
      <div
        v-for="member in members"
        :key="member.userId"
        class="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-700/50 transition-colors"
      >
        <!-- 头像 -->
        <div class="w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-xs">
          {{ member.username.charAt(0).toUpperCase() }}
        </div>

        <!-- 名称 -->
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">
            {{ member.username }}
            <span
              v-if="member.userId === roomStore.currentRoom?.hostId"
              class="text-xs text-primary-400"
            >
              (房主)
            </span>
          </p>
        </div>

        <!-- 状态图标 -->
        <span :class="getStatusClass(member)" class="text-sm">
          {{ getStatusIcon(member) }}
        </span>
      </div>

      <!-- 空状态 -->
      <div
        v-if="members.length === 0"
        class="text-center text-gray-500 text-sm py-4"
      >
        暂无成员
      </div>
    </div>
  </div>
</template>