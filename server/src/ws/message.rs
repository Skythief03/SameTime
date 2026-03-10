use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStateMember {
    pub user_id: String,
    pub username: String,
    pub is_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    // 同步相关
    SyncRequest {
        room_id: String,
        timestamp: f64,
        is_playing: bool,
    },
    SyncBroadcast {
        timestamp: f64,
        is_playing: bool,
        sender_id: String,
    },

    // 聊天相关
    ChatMessage {
        room_id: String,
        content: String,
        sender_id: String,
        sender_name: String,
        created_at: i64,
    },

    // 弹幕相关
    Danmaku {
        room_id: String,
        content: String,
        video_timestamp: f64,
        color: String,
        sender_id: String,
        sender_name: String,
    },

    // 语音信令
    VoiceOffer {
        room_id: String,
        target_user_id: String,
        offer: String,
    },
    VoiceAnswer {
        room_id: String,
        target_user_id: String,
        answer: String,
    },
    IceCandidate {
        room_id: String,
        target_user_id: String,
        candidate: String,
    },

    // 房间事件
    UserJoined {
        user_id: String,
        username: String,
    },
    UserLeft {
        user_id: String,
    },
    ReadyStateChanged {
        user_id: String,
        is_ready: bool,
    },
    RoomState {
        members: Vec<RoomStateMember>,
    },

    // 视频 hash 比对
    VideoHash {
        room_id: String,
        video_hash: String,
        file_name: String,
        sender_id: String,
        sender_name: String,
    },

    // 心跳
    Ping {},
    Pong {},

    // 错误
    Error {
        message: String,
    },
}