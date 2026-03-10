// 用户相关类型
export interface User {
  id: string;
  username: string;
  avatarUrl?: string;
}

// 房间相关类型
export interface Room {
  id: string;
  name: string;
  hostId: string;
  videoHash?: string;
  currentTime: number;
  isPlaying: boolean;
  createdAt: number;
}

export interface RoomMember {
  userId: string;
  username: string;
  isReady: boolean;
  isMuted: boolean;
}

// 视频源相关类型
export type VideoSourceType = "local" | "server" | "magnet" | "webdav";

export interface VideoSource {
  type: VideoSourceType;
  path: string;
  hash: string;
  size: number;
  name: string;
}

export interface VideoMeta {
  source: VideoSource;
  duration: number;
  resolution: { width: number; height: number };
}

// 聊天消息类型
export interface ChatMessage {
  id: string;
  roomId: string;
  userId: string;
  username: string;
  content: string;
  createdAt: number;
}

// 弹幕类型
export interface Danmaku {
  id: string;
  userId: string;
  username: string;
  content: string;
  timestamp: number;
  color: string;
  type: "scroll" | "top" | "bottom";
  createdAt: number;
}

// WebSocket 消息类型
export type WsMessage =
  | SyncRequestMessage
  | SyncBroadcastMessage
  | ChatMessageMessage
  | DanmakuMessage
  | VoiceOfferMessage
  | VoiceAnswerMessage
  | IceCandidateMessage
  | UserJoinedMessage
  | UserLeftMessage
  | ReadyStateChangedMessage
  | RoomStateMessage
  | VideoHashMessage
  | ErrorMessage;

export interface SyncRequestMessage {
  type: "sync_request";
  roomId: string;
  timestamp: number;
  isPlaying: boolean;
}

export interface SyncBroadcastMessage {
  type: "sync_broadcast";
  timestamp: number;
  isPlaying: boolean;
  senderId: string;
}

export interface ChatMessageMessage {
  type: "chat_message";
  roomId: string;
  content: string;
  senderId: string;
  senderName: string;
  createdAt: number;
}

export interface DanmakuMessage {
  type: "danmaku";
  roomId: string;
  content: string;
  videoTimestamp: number;
  color: string;
  senderId: string;
  senderName: string;
}

export interface VoiceOfferMessage {
  type: "voice_offer";
  roomId: string;
  targetUserId: string;
  offer: string;
}

export interface VoiceAnswerMessage {
  type: "voice_answer";
  roomId: string;
  targetUserId: string;
  answer: string;
}

export interface IceCandidateMessage {
  type: "ice_candidate";
  roomId: string;
  targetUserId: string;
  candidate: string;
}

export interface UserJoinedMessage {
  type: "user_joined";
  userId: string;
  username: string;
}

export interface UserLeftMessage {
  type: "user_left";
  userId: string;
}

export interface ReadyStateChangedMessage {
  type: "ready_state_changed";
  userId: string;
  isReady: boolean;
}

export interface RoomStateMessage {
  type: "room_state";
  members: Array<{
    user_id: string;
    username: string;
    is_ready: boolean;
  }>;
}

export interface VideoHashMessage {
  type: "video_hash";
  roomId: string;
  videoHash: string;
  fileName: string;
  senderId: string;
  senderName: string;
}

export interface ErrorMessage {
  type: "error";
  message: string;
}