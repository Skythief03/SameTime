export type PlayerErrorCode =
  | "UNKNOWN"
  | "NOT_READY"
  | "FILE_NOT_FOUND"
  | "UNSUPPORTED_CODEC"
  | "PLAY_ABORTED"
  | "NETWORK_ERROR";

export interface PlayerState {
  currentTime: number;
  duration: number;
  isPlaying: boolean;
  volume: number;
  source?: string | null;
}

export interface PlayerError {
  code: PlayerErrorCode;
  message: string;
  cause?: unknown;
}

export type Unsubscribe = () => void;

export interface PlayerAdapter {
  checkAvailability(): Promise<void>;
  calculateFileHash(source: string): Promise<string>;
  getFileSize(source: string): Promise<number>;
  load(source: string): Promise<void>;
  play(): Promise<void>;
  pause(): Promise<void>;
  seek(position: number): Promise<void>;
  setVolume(volume: number): Promise<void>;
  getState(): PlayerState;
  dispose(): Promise<void>;

  onTimeUpdate(handler: (currentTime: number, duration: number) => void): Unsubscribe;
  onStateChange(handler: (state: PlayerState) => void): Unsubscribe;
  onError(handler: (error: PlayerError) => void): Unsubscribe;
}
