<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useUserStore } from "@/stores/user";
import { useRoomStore } from "@/stores/room";
import { showToast } from "@/utils/toast";

const router = useRouter();
const userStore = useUserStore();
const roomStore = useRoomStore();

const roomId = ref("");
const roomName = ref("");
const roomPassword = ref("");
const joinPassword = ref("");
const showCreateModal = ref(false);
const showAuthModal = ref(false);
const authMode = ref<"login" | "register">("login");
const authUsername = ref("");
const authPassword = ref("");
const authError = ref("");
const authLoading = ref(false);
const quickName = ref("");

const handleJoinRoom = async () => {
  if (!roomId.value.trim()) return;
  
  try {
    await roomStore.joinRoom(roomId.value.trim(), joinPassword.value || undefined);
    router.push(`/room/${roomId.value.trim()}`);
  } catch (error: any) {
    showToast(error?.message || "加入房间失败", "error");
  }
};

const handleCreateRoom = async () => {
  if (!roomName.value.trim()) return;
  
  try {
    const room = await roomStore.createRoom(roomName.value.trim(), roomPassword.value || undefined);
    showCreateModal.value = false;
    roomName.value = "";
    roomPassword.value = "";
    showToast(`房间 "${room.name}" 创建成功！`, "success");
    router.push(`/room/${room.id}`);
  } catch (error: any) {
    showToast("创建房间失败", "error");
  }
};

const handleAuth = async () => {
  if (!authUsername.value.trim() || !authPassword.value) return;
  authError.value = "";
  authLoading.value = true;

  try {
    if (authMode.value === "login") {
      await userStore.login(authUsername.value.trim(), authPassword.value);
    } else {
      await userStore.register(authUsername.value.trim(), authPassword.value);
    }
    showAuthModal.value = false;
    authUsername.value = "";
    authPassword.value = "";
  } catch (error: any) {
    authError.value = authMode.value === "login" ? "登录失败，请检查用户名和密码" : "注册失败，用户名可能已被占用";
  } finally {
    authLoading.value = false;
  }
};

const handleQuickStart = () => {
  if (!quickName.value.trim()) return;
  userStore.setUsername(quickName.value.trim());
};

const handleLogout = () => {
  userStore.logout();
};
</script>

<template>
  <div class="min-h-screen flex flex-col items-center justify-center p-8">
    <!-- Logo 和标题 -->
    <div class="text-center mb-12">
      <h1 class="text-5xl font-bold text-primary-400 mb-4">SameTime</h1>
      <p class="text-gray-400 text-lg">与朋友一起同步观影</p>
    </div>

    <!-- 主卡片 -->
    <div class="card w-full max-w-md">
      <!-- 用户状态 - 已设置用户名 -->
      <div v-if="userStore.username" class="mb-6 pb-6 border-b border-gray-700">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-primary-600 flex items-center justify-center font-bold">
              {{ userStore.username.charAt(0).toUpperCase() }}
            </div>
            <div>
              <p class="font-medium">{{ userStore.username }}</p>
              <p class="text-sm text-gray-400">{{ userStore.isLoggedIn ? '已登录' : '快速模式' }}</p>
            </div>
          </div>
          <button class="text-sm text-gray-400 hover:text-red-400 transition-colors" @click="handleLogout">
            退出
          </button>
        </div>
      </div>

      <!-- 未设置用户名时的快速入口 -->
      <div v-else class="mb-6 pb-6 border-b border-gray-700">
        <div class="mb-3">
          <label class="block text-sm font-medium text-gray-300 mb-2">设置昵称</label>
          <div class="flex gap-2">
            <input
              v-model="quickName"
              type="text"
              placeholder="输入你的昵称"
              class="flex-1 px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
              @keyup.enter="handleQuickStart"
            />
            <button class="btn btn-primary" :disabled="!quickName.trim()" @click="handleQuickStart">
              确定
            </button>
          </div>
        </div>
        <button
          class="text-sm text-primary-400 hover:text-primary-300 transition-colors"
          @click="showAuthModal = true; authMode = 'login'"
        >
          已有账号？登录 / 注册
        </button>
      </div>

      <!-- 加入房间 -->
      <div class="mb-6">
        <label class="block text-sm font-medium text-gray-300 mb-2">
          加入房间
        </label>
        <div class="flex gap-2">
          <input
            v-model="roomId"
            type="text"
            placeholder="输入房间号"
            class="flex-1 px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
            @keyup.enter="handleJoinRoom"
          />
          <button
            class="btn btn-primary"
            :disabled="!roomId.trim()"
            @click="handleJoinRoom"
          >
            加入
          </button>
        </div>
        <input
          v-model="joinPassword"
          type="password"
          placeholder="房间密码（如有）"
          class="w-full mt-2 px-4 py-2 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors text-sm"
        />
      </div>

      <!-- 分隔线 -->
      <div class="flex items-center gap-4 mb-6">
        <div class="flex-1 h-px bg-gray-700"></div>
        <span class="text-gray-500 text-sm">或者</span>
        <div class="flex-1 h-px bg-gray-700"></div>
      </div>

      <!-- 创建房间按钮 -->
      <button
        class="w-full btn btn-secondary py-3"
        @click="showCreateModal = true"
      >
        创建新房间
      </button>
    </div>

    <!-- 设置入口 -->
    <button
      class="mt-8 text-gray-400 hover:text-white transition-colors"
      @click="router.push('/settings')"
    >
      ⚙️ 设置
    </button>

    <!-- 登录/注册弹窗 -->
    <Teleport to="body">
      <div
        v-if="showAuthModal"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
        @click.self="showAuthModal = false"
      >
        <div class="card w-full max-w-sm mx-4">
          <!-- 切换标签 -->
          <div class="flex mb-6 bg-gray-700 rounded-lg p-1">
            <button
              class="flex-1 py-2 rounded-md text-sm font-medium transition-colors"
              :class="authMode === 'login' ? 'bg-primary-600 text-white' : 'text-gray-400 hover:text-white'"
              @click="authMode = 'login'; authError = ''"
            >
              登录
            </button>
            <button
              class="flex-1 py-2 rounded-md text-sm font-medium transition-colors"
              :class="authMode === 'register' ? 'bg-primary-600 text-white' : 'text-gray-400 hover:text-white'"
              @click="authMode = 'register'; authError = ''"
            >
              注册
            </button>
          </div>

          <!-- 表单 -->
          <div class="space-y-4 mb-6">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">用户名</label>
              <input
                v-model="authUsername"
                type="text"
                placeholder="输入用户名"
                class="w-full px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1">密码</label>
              <input
                v-model="authPassword"
                type="password"
                :placeholder="authMode === 'register' ? '设置密码（至少4位）' : '输入密码'"
                class="w-full px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
                @keyup.enter="handleAuth"
              />
            </div>
          </div>

          <!-- 错误提示 -->
          <p v-if="authError" class="text-red-400 text-sm mb-4">{{ authError }}</p>

          <!-- 按钮 -->
          <div class="flex gap-3">
            <button class="flex-1 btn btn-secondary" @click="showAuthModal = false">
              取消
            </button>
            <button
              class="flex-1 btn btn-primary"
              :disabled="!authUsername.trim() || !authPassword || authLoading"
              @click="handleAuth"
            >
              {{ authLoading ? '处理中...' : (authMode === 'login' ? '登录' : '注册') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 创建房间弹窗 -->
    <Teleport to="body">
      <div
        v-if="showCreateModal"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
        @click.self="showCreateModal = false"
      >
        <div class="card w-full max-w-sm mx-4">
          <h2 class="text-xl font-bold mb-4">创建房间</h2>
          
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-300 mb-2">
              房间名称
            </label>
            <input
              v-model="roomName"
              type="text"
              placeholder="给房间起个名字"
              class="w-full px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
              @keyup.enter="handleCreateRoom"
            />
          </div>

          <div class="mb-6">
            <label class="block text-sm font-medium text-gray-300 mb-2">
              房间密码（可选）
            </label>
            <input
              v-model="roomPassword"
              type="password"
              placeholder="设置后需密码才能加入"
              class="w-full px-4 py-3 bg-gray-700 rounded-lg border border-gray-600 focus:border-primary-500 transition-colors"
            />
          </div>

          <div class="flex gap-3">
            <button
              class="flex-1 btn btn-secondary"
              @click="showCreateModal = false"
            >
              取消
            </button>
            <button
              class="flex-1 btn btn-primary"
              :disabled="!roomName.trim()"
              @click="handleCreateRoom"
            >
              创建
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>