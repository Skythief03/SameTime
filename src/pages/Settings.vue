<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/user";
import { showToast } from "@/utils/toast";
import { getApiBaseUrl, getDefaultApiBaseUrl, setApiBaseUrl } from "@/platform";

const router = useRouter();
const userStore = useUserStore();

const serverUrl = ref(getDefaultApiBaseUrl());
const downloadPath = ref("");
const username = ref("");
const connectionStatus = ref<"idle" | "testing" | "success" | "error">("idle");
const isFirstTime = ref(false);

onMounted(() => {
  isFirstTime.value = !localStorage.getItem("sametime_configured");
  // 从本地存储加载设置
  serverUrl.value = getApiBaseUrl();
  downloadPath.value = localStorage.getItem("downloadPath") || "";
  username.value = userStore.username || "";
});

const saveSettings = () => {
  setApiBaseUrl(serverUrl.value);
  localStorage.setItem("downloadPath", downloadPath.value);
  
  if (username.value && username.value !== userStore.username) {
    userStore.setUsername(username.value);
  }
};

const testConnection = async () => {
  connectionStatus.value = "testing";
  
  try {
    const response = await fetch(`${serverUrl.value}/api/health`, {
      method: "GET",
      signal: AbortSignal.timeout(5000),
    });
    
    if (response.ok) {
      connectionStatus.value = "success";
    } else {
      connectionStatus.value = "error";
    }
  } catch {
    connectionStatus.value = "error";
  }
  
  setTimeout(() => {
    connectionStatus.value = "idle";
  }, 3000);
};

const handleSave = () => {
  saveSettings();
  showToast("设置已保存", "success");
  localStorage.setItem("sametime_configured", "true");
  router.push("/");
};

const resetSettings = () => {
  serverUrl.value = getDefaultApiBaseUrl();
  downloadPath.value = "";
  connectionStatus.value = "idle";
  showToast("已恢复默认设置", "info");
};
</script>

<template>
  <div class="min-h-screen p-8">
    <div class="max-w-2xl mx-auto">
      <!-- 返回按钮 -->
      <button
        class="mb-8 text-gray-400 hover:text-white transition-colors"
        @click="router.push('/')"
      >
        ← 返回首页
      </button>

      <h1 class="text-3xl font-bold mb-8">设置</h1>

      <!-- 首次启动提示 -->
      <div
        v-if="isFirstTime"
        class="mb-6 p-4 rounded-lg bg-primary-600/20 border border-primary-500/30"
      >
        <p class="text-primary-300 text-sm">
          👋 首次使用 SameTime，请先配置服务器地址并测试连接。
        </p>
      </div>

      <!-- 服务器设置 -->
      <section class="card mb-6">
        <h2 class="text-xl font-semibold mb-4">服务器设置</h2>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            服务器地址
          </label>
          <div class="flex gap-2">
            <input
              v-model="serverUrl"
              type="text"
              placeholder="http://localhost:8080"
              class="flex-1 px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
            />
            <button
              class="btn btn-secondary"
              :disabled="connectionStatus === 'testing'"
              @click="testConnection"
            >
              {{ connectionStatus === 'testing' ? '测试中...' : '测试连接' }}
            </button>
          </div>
          
          <p
            v-if="connectionStatus === 'success'"
            class="mt-2 text-sm text-green-400"
          >
            ✓ 连接成功
          </p>
          <p
            v-if="connectionStatus === 'error'"
            class="mt-2 text-sm text-red-400"
          >
            ✗ 连接失败，请检查服务器地址
          </p>
        </div>
      </section>

      <!-- 用户设置 -->
      <section class="card mb-6">
        <h2 class="text-xl font-semibold mb-4">用户设置</h2>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            用户名
          </label>
          <input
            v-model="username"
            type="text"
            placeholder="输入你的昵称"
            class="w-full px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
          />
        </div>
      </section>

      <!-- 下载设置 -->
      <section class="card mb-6">
        <h2 class="text-xl font-semibold mb-4">下载设置</h2>
        
        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-300 mb-2">
            下载目录
          </label>
          <div class="flex gap-2">
            <input
              v-model="downloadPath"
              type="text"
              placeholder="选择下载目录"
              class="flex-1 px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
              readonly
            />
            <button class="btn btn-secondary">
              浏览...
            </button>
          </div>
          <p class="mt-2 text-sm text-gray-400">
            磁力链接下载的视频将保存到此目录
          </p>
        </div>
      </section>

      <!-- 保存按钮 -->
      <div class="flex justify-between">
        <button
          class="text-sm text-gray-500 hover:text-gray-300 transition-colors"
          @click="resetSettings"
        >
          恢复默认
        </button>
        <div class="flex gap-4">
          <button
            class="btn btn-secondary"
            @click="router.push('/')"
          >
            取消
          </button>
          <button
            class="btn btn-primary"
            @click="handleSave"
          >
            保存设置
          </button>
        </div>
      </div>
    </div>
  </div>
</template>