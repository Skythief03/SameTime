<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { useRoomStore } from "@/stores/room";
import type { ChatMessage } from "@/types";

const roomStore = useRoomStore();

const messages = ref<ChatMessage[]>([]);
const inputText = ref("");
const messagesContainer = ref<HTMLDivElement | null>(null);
const MAX_MESSAGES = 500;

const trimMessages = () => {
  if (messages.value.length > MAX_MESSAGES) {
    messages.value = messages.value.slice(-MAX_MESSAGES);
  }
};

const sendMessage = () => {
  if (!inputText.value.trim()) return;

  const content = inputText.value.trim();
  const username = localStorage.getItem("username") || "匿名用户";
  const userId = localStorage.getItem("userId") || "";

  roomStore.sendMessage({
    type: "chat_message",
    room_id: roomStore.currentRoom?.id || "",
    content,
    created_at: Date.now(),
  });

  // 本地立即显示
  messages.value.push({
    id: crypto.randomUUID(),
    roomId: roomStore.currentRoom?.id || "",
    userId,
    username,
    content,
    createdAt: Date.now(),
  });
  trimMessages();
  scrollToBottom();
  inputText.value = "";
};

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
  });
};

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp);
  return `${date.getHours().toString().padStart(2, "0")}:${date.getMinutes().toString().padStart(2, "0")}`;
};

const handleChatMessage = (event: CustomEvent) => {
  const d = event.detail;
  const senderId = d.sender_id || d.senderId || "";
  const localUserId = localStorage.getItem("userId") || "";

  // 跳过自己发送的消息（本地已经显示）
  if (senderId === localUserId) return;

  const message: ChatMessage = {
    id: crypto.randomUUID(),
    roomId: d.room_id || d.roomId || "",
    userId: senderId,
    username: d.sender_name || d.senderName || "匿名用户",
    content: d.content || "",
    createdAt: d.created_at || d.createdAt || Date.now(),
  };

  messages.value.push(message);
  trimMessages();
  scrollToBottom();
};

onMounted(() => {
  window.addEventListener("chat-message", handleChatMessage as EventListener);
});

onUnmounted(() => {
  window.removeEventListener("chat-message", handleChatMessage as EventListener);
});
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- 标题 -->
    <div class="p-3 border-b border-gray-700">
      <h3 class="font-medium text-sm text-gray-300">聊天</h3>
    </div>

    <!-- 消息列表 -->
    <div
      ref="messagesContainer"
      class="flex-1 overflow-y-auto p-3 space-y-3"
    >
      <div
        v-for="msg in messages"
        :key="msg.id"
        class="group"
      >
        <div class="flex items-start gap-2">
          <!-- 头像 -->
          <div class="w-8 h-8 rounded-full bg-primary-600 flex items-center justify-center text-xs flex-shrink-0">
            {{ msg.username.charAt(0).toUpperCase() }}
          </div>
          
          <!-- 消息内容 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-baseline gap-2">
              <span class="text-sm font-medium text-gray-300">{{ msg.username }}</span>
              <span class="text-xs text-gray-500">{{ formatTime(msg.createdAt) }}</span>
            </div>
            <p class="text-sm text-gray-100 break-words">{{ msg.content }}</p>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-if="messages.length === 0"
        class="h-full flex items-center justify-center text-gray-500 text-sm"
      >
        暂无消息
      </div>
    </div>

    <!-- 输入框 -->
    <div class="p-3 border-t border-gray-700">
      <div class="flex gap-2">
        <input
          v-model="inputText"
          type="text"
          placeholder="发送消息..."
          class="flex-1 px-3 py-2 bg-gray-700 rounded-lg text-sm border border-gray-600 focus:border-primary-500 transition-colors"
          @keyup.enter="sendMessage"
        />
        <button
          class="px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded-lg text-sm transition-colors"
          :disabled="!inputText.trim()"
          @click="sendMessage"
        >
          发送
        </button>
      </div>
    </div>
  </div>
</template>