import type { PlayerAdapter, PlayerError, PlayerState, Unsubscribe } from "@/player";
import { createPlayerError, mapErrorToPlayerError } from "@/player";

type TimeListener = (currentTime: number, duration: number) => void;
type StateListener = (state: PlayerState) => void;
type ErrorListener = (error: PlayerError) => void;

export class WebVideoAdapter implements PlayerAdapter {
  private video: HTMLVideoElement;
  private state: PlayerState = {
    currentTime: 0,
    duration: 0,
    isPlaying: false,
    volume: 100,
    source: null,
  };

  private timeListeners = new Set<TimeListener>();
  private stateListeners = new Set<StateListener>();
  private errorListeners = new Set<ErrorListener>();

  constructor() {
    this.video = document.createElement("video");
    this.video.preload = "metadata";
    this.video.addEventListener("timeupdate", this.handleTimeUpdate);
    this.video.addEventListener("loadedmetadata", this.handleTimeUpdate);
    this.video.addEventListener("play", this.handleStateUpdate);
    this.video.addEventListener("pause", this.handleStateUpdate);
    this.video.addEventListener("error", this.handleError);
  }

  async checkAvailability(): Promise<void> {
    if (typeof window === "undefined") {
      throw createPlayerError("NOT_READY", "当前环境不支持 WebVideoAdapter");
    }
  }

  async calculateFileHash(): Promise<string> {
    throw createPlayerError("NOT_READY", "Web 端文件 hash 计算尚未接入（D1 子任务）");
  }

  async getFileSize(): Promise<number> {
    throw createPlayerError("NOT_READY", "Web 端文件大小读取尚未接入（D2 子任务）");
  }

  async load(source: string): Promise<void> {
    this.video.src = source;
    this.video.load();
    this.state.source = source;
    this.state.currentTime = 0;
    this.state.duration = 0;
    this.emitState();
  }

  async play(): Promise<void> {
    await this.video.play();
    this.state.isPlaying = true;
    this.emitState();
  }

  async pause(): Promise<void> {
    this.video.pause();
    this.state.isPlaying = false;
    this.emitState();
  }

  async seek(position: number): Promise<void> {
    this.video.currentTime = position;
    this.state.currentTime = position;
    this.emitTime();
    this.emitState();
  }

  async setVolume(volume: number): Promise<void> {
    this.video.volume = Math.max(0, Math.min(1, volume / 100));
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
    this.video.pause();
    this.video.removeAttribute("src");
    this.video.load();
    this.video.removeEventListener("timeupdate", this.handleTimeUpdate);
    this.video.removeEventListener("loadedmetadata", this.handleTimeUpdate);
    this.video.removeEventListener("play", this.handleStateUpdate);
    this.video.removeEventListener("pause", this.handleStateUpdate);
    this.video.removeEventListener("error", this.handleError);
    this.timeListeners.clear();
    this.stateListeners.clear();
    this.errorListeners.clear();
  }

  private handleTimeUpdate = () => {
    this.state.currentTime = this.video.currentTime || 0;
    this.state.duration = this.video.duration || 0;
    this.emitTime();
  };

  private handleStateUpdate = () => {
    this.state.isPlaying = !this.video.paused;
    this.state.currentTime = this.video.currentTime || this.state.currentTime;
    this.state.duration = this.video.duration || this.state.duration;
    this.emitState();
  };

  private handleError = () => {
    const err = this.video.error?.message || "Web 播放器错误";
    this.emitError(mapErrorToPlayerError(new Error(err)));
  };

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
