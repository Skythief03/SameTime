import { invoke } from "@tauri-apps/api/core";
import type { PlayerAdapter, PlayerError, PlayerState, Unsubscribe } from "@/player";
import { createPlayerError, mapErrorToPlayerError } from "@/player";

type TimeListener = (currentTime: number, duration: number) => void;
type StateListener = (state: PlayerState) => void;
type ErrorListener = (error: PlayerError) => void;

export class TauriMpvAdapter implements PlayerAdapter {
  private state: PlayerState = {
    currentTime: 0,
    duration: 0,
    isPlaying: false,
    volume: 100,
    source: null,
  };

  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private timeListeners = new Set<TimeListener>();
  private stateListeners = new Set<StateListener>();
  private errorListeners = new Set<ErrorListener>();

  async checkAvailability(): Promise<void> {
    await invoke("mpv_check");
  }

  async calculateFileHash(source: string): Promise<string> {
    return invoke<string>("calculate_file_hash", { filePath: source });
  }

  async getFileSize(source: string): Promise<number> {
    return invoke<number>("get_file_size", { filePath: source });
  }

  async load(source: string): Promise<void> {
    await invoke("mpv_play", { filePath: source });
    this.state.source = source;
    this.state.currentTime = 0;
    this.state.duration = 0;
    this.startPolling();
    this.emitState();
  }

  async play(): Promise<void> {
    await invoke("mpv_set_pause", { paused: false });
    this.state.isPlaying = true;
    this.emitState();
  }

  async pause(): Promise<void> {
    await invoke("mpv_set_pause", { paused: true });
    this.state.isPlaying = false;
    this.emitState();
  }

  async seek(position: number): Promise<void> {
    await invoke("mpv_seek", { position });
    this.state.currentTime = position;
    this.emitTime();
    this.emitState();
  }

  async setVolume(volume: number): Promise<void> {
    await invoke("mpv_set_volume", { volume });
    this.state.volume = volume;
    this.emitState();
  }

  getState(): PlayerState {
    return { ...this.state };
  }

  onTimeUpdate(handler: TimeListener): Unsubscribe {
    this.timeListeners.add(handler);
    return () => this.timeListeners.delete(handler);
  }

  onStateChange(handler: StateListener): Unsubscribe {
    this.stateListeners.add(handler);
    return () => this.stateListeners.delete(handler);
  }

  onError(handler: ErrorListener): Unsubscribe {
    this.errorListeners.add(handler);
    return () => this.errorListeners.delete(handler);
  }

  async dispose(): Promise<void> {
    this.stopPolling();
    this.timeListeners.clear();
    this.stateListeners.clear();
    this.errorListeners.clear();
    await invoke("mpv_stop");
    this.state = {
      currentTime: 0,
      duration: 0,
      isPlaying: false,
      volume: 100,
      source: null,
    };
  }

  private startPolling() {
    if (this.pollTimer) return;

    this.pollTimer = setInterval(async () => {
      try {
        const pos = await invoke<number>("mpv_get_position");
        if (typeof pos === "number" && !Number.isNaN(pos)) {
          this.state.currentTime = pos;
          this.emitTime();
        }

        if (this.state.duration <= 0) {
          const dur = await invoke<number>("mpv_get_duration");
          if (typeof dur === "number" && !Number.isNaN(dur) && dur > 0) {
            this.state.duration = dur;
            this.emitTime();
          }
        }

        const paused = await invoke<boolean>("mpv_get_paused");
        if (typeof paused === "boolean") {
          this.state.isPlaying = !paused;
          this.emitState();
        }
      } catch (error) {
        this.emitError(mapErrorToPlayerError(error));
      }
    }, 500);
  }

  private stopPolling() {
    if (!this.pollTimer) return;
    clearInterval(this.pollTimer);
    this.pollTimer = null;
  }

  private emitTime() {
    for (const listener of this.timeListeners) {
      listener(this.state.currentTime, this.state.duration);
    }
  }

  private emitState() {
    const snapshot = this.getState();
    for (const listener of this.stateListeners) {
      listener(snapshot);
    }
  }

  private emitError(error: PlayerError) {
    for (const listener of this.errorListeners) {
      listener(error);
    }
  }
}

export class UnsupportedPlayerAdapter implements PlayerAdapter {
  async checkAvailability(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async calculateFileHash(): Promise<string> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async getFileSize(): Promise<number> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async load(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async play(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async pause(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async seek(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  async setVolume(): Promise<void> {
    throw createPlayerError("NOT_READY", "当前环境暂未实现播放器适配");
  }
  getState(): PlayerState {
    return { currentTime: 0, duration: 0, isPlaying: false, volume: 100, source: null };
  }
  async dispose(): Promise<void> {
    // noop
  }
  onTimeUpdate(): Unsubscribe {
    return () => {};
  }
  onStateChange(): Unsubscribe {
    return () => {};
  }
  onError(): Unsubscribe {
    return () => {};
  }
}
