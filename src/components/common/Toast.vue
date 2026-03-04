<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

interface ToastItem {
  id: string;
  message: string;
  type: "success" | "error" | "warning" | "info";
  duration: number;
}

const toasts = ref<ToastItem[]>([]);

const addToast = (event: CustomEvent) => {
  const { message, type = "info", duration = 3000 } = event.detail;
  const id = crypto.randomUUID();
  toasts.value.push({ id, message, type, duration });

  setTimeout(() => {
    removeToast(id);
  }, duration);
};

const removeToast = (id: string) => {
  const idx = toasts.value.findIndex((t) => t.id === id);
  if (idx !== -1) {
    toasts.value.splice(idx, 1);
  }
};

const typeStyles: Record<string, string> = {
  success: "bg-green-600/90 border-green-500",
  error: "bg-red-600/90 border-red-500",
  warning: "bg-yellow-600/90 border-yellow-500",
  info: "bg-blue-600/90 border-blue-500",
};

const typeIcons: Record<string, string> = {
  success: "✓",
  error: "✕",
  warning: "⚠",
  info: "ℹ",
};

onMounted(() => {
  window.addEventListener("app-toast", addToast as EventListener);
});

onUnmounted(() => {
  window.removeEventListener("app-toast", addToast as EventListener);
});
</script>

<template>
  <Teleport to="body">
    <div class="fixed top-4 right-4 z-[9999] space-y-2 pointer-events-none">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="pointer-events-auto px-4 py-3 rounded-lg border shadow-lg text-sm flex items-center gap-2 max-w-sm cursor-pointer transition-all"
          :class="typeStyles[toast.type]"
          @click="removeToast(toast.id)"
        >
          <span class="font-bold text-lg">{{ typeIcons[toast.type] }}</span>
          <span>{{ toast.message }}</span>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-enter-active {
  transition: all 0.3s ease-out;
}
.toast-leave-active {
  transition: all 0.2s ease-in;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(30px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>