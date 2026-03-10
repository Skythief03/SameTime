export const isTauri = (): boolean => {
  if (typeof window === "undefined") return false;
  return "__TAURI_INTERNALS__" in window || "__TAURI__" in window;
};

export const isWeb = (): boolean => !isTauri();
