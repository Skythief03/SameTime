const DEFAULT_API_BASE_URL = "http://localhost:8080";

const normalizeBaseUrl = (url: string): string => url.replace(/\/$/, "");

export const getDefaultApiBaseUrl = (): string => {
  const envUrl = import.meta.env.VITE_API_BASE_URL as string | undefined;
  return normalizeBaseUrl(envUrl || DEFAULT_API_BASE_URL);
};

export const getApiBaseUrl = (): string => {
  if (typeof window === "undefined") return getDefaultApiBaseUrl();
  const saved = localStorage.getItem("serverUrl");
  return normalizeBaseUrl(saved || getDefaultApiBaseUrl());
};

export const getWsBaseUrl = (): string => {
  const envWs = import.meta.env.VITE_WS_BASE_URL as string | undefined;
  if (envWs) return normalizeBaseUrl(envWs);

  return getApiBaseUrl().replace(/^http/, "ws");
};

export const setApiBaseUrl = (url: string): void => {
  if (typeof window === "undefined") return;
  localStorage.setItem("serverUrl", normalizeBaseUrl(url));
};

export const resetApiBaseUrl = (): void => {
  if (typeof window === "undefined") return;
  localStorage.setItem("serverUrl", getDefaultApiBaseUrl());
};
