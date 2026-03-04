import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useUserStore = defineStore("user", () => {
  const userId = ref<string | null>(null);
  const username = ref<string | null>(null);
  const token = ref<string | null>(null);

  const isLoggedIn = computed(() => !!token.value);

  // 初始化时从 localStorage 读取
  const init = () => {
    userId.value = localStorage.getItem("userId");
    username.value = localStorage.getItem("username");
    token.value = localStorage.getItem("token");
  };

  const setUsername = (name: string) => {
    username.value = name;
    localStorage.setItem("username", name);
    
    // 如果没有 userId，生成一个临时 ID
    if (!userId.value) {
      userId.value = crypto.randomUUID();
      localStorage.setItem("userId", userId.value);
    }
  };

  const login = async (name: string, password: string) => {
    const serverUrl = localStorage.getItem("serverUrl") || "http://localhost:8080";
    
    const response = await fetch(`${serverUrl}/api/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username: name, password }),
    });

    if (!response.ok) {
      throw new Error("Login failed");
    }

    const data = await response.json();
    userId.value = data.user_id;
    username.value = data.username;
    token.value = data.token;

    localStorage.setItem("userId", data.user_id);
    localStorage.setItem("username", data.username);
    localStorage.setItem("token", data.token);
  };

  const register = async (name: string, password: string) => {
    const serverUrl = localStorage.getItem("serverUrl") || "http://localhost:8080";
    
    const response = await fetch(`${serverUrl}/api/auth/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username: name, password }),
    });

    if (!response.ok) {
      throw new Error("Registration failed");
    }

    const data = await response.json();
    userId.value = data.user_id;
    username.value = data.username;
    token.value = data.token;

    localStorage.setItem("userId", data.user_id);
    localStorage.setItem("username", data.username);
    localStorage.setItem("token", data.token);
  };

  const logout = () => {
    userId.value = null;
    username.value = null;
    token.value = null;

    localStorage.removeItem("userId");
    localStorage.removeItem("username");
    localStorage.removeItem("token");
  };

  // 自动初始化
  init();

  return {
    userId,
    username,
    token,
    isLoggedIn,
    setUsername,
    login,
    register,
    logout,
  };
});