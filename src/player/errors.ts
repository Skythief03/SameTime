import type { PlayerError, PlayerErrorCode } from "./types";

const FALLBACK_MESSAGE = "播放器发生未知错误";

export const createPlayerError = (
  code: PlayerErrorCode,
  message: string,
  cause?: unknown,
): PlayerError => ({ code, message, cause });

export const mapErrorToPlayerError = (error: unknown): PlayerError => {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    const code = (error as any).code as PlayerErrorCode;
    const message = (error as any).message as string;
    return createPlayerError(code || "UNKNOWN", message || FALLBACK_MESSAGE, error);
  }

  if (error instanceof Error) {
    if (error.message.includes("MPV_NOT_FOUND")) {
      return createPlayerError("NOT_READY", "未检测到可用播放器（MPV 未安装）", error);
    }

    return createPlayerError("UNKNOWN", error.message || FALLBACK_MESSAGE, error);
  }

  return createPlayerError("UNKNOWN", FALLBACK_MESSAGE, error);
};
