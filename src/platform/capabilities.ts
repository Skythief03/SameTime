import { isTauri, isWeb } from "./runtime";

export interface RuntimeCapabilities {
  isTauri: boolean;
  isWeb: boolean;
  canUseLocalFile: boolean;
  canUseWebRTC: boolean;
  canUseFullscreen: boolean;
}

export const getRuntimeCapabilities = (): RuntimeCapabilities => ({
  isTauri: isTauri(),
  isWeb: isWeb(),
  canUseLocalFile: typeof window !== "undefined",
  canUseWebRTC: typeof window !== "undefined" && !!window.RTCPeerConnection,
  canUseFullscreen:
    typeof document !== "undefined" && typeof document.documentElement.requestFullscreen === "function",
});
