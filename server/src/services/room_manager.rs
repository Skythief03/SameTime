use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

use crate::ws::message::WsMessage;

pub struct Room {
    pub id: String,
    pub name: String,
    pub host_id: RwLock<String>,
    pub video_hash: RwLock<Option<String>>,
    pub current_time: RwLock<f64>,
    pub is_playing: RwLock<bool>,
    pub members: DashMap<String, RoomMember>,
    pub broadcast: broadcast::Sender<WsMessage>,
    pub created_at: i64,
}

pub struct RoomMember {
    pub user_id: String,
    pub username: String,
    pub is_ready: bool,
}

pub struct RoomManager {
    rooms: DashMap<String, Arc<Room>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
        }
    }

    pub fn create_room(
        &self,
        room_id: String,
        name: String,
        host_id: String,
        host_name: String,
    ) -> Arc<Room> {
        let (tx, _) = broadcast::channel(256);

        let room = Arc::new(Room {
            id: room_id.clone(),
            name: name.clone(),
            host_id: RwLock::new(host_id.clone()),
            video_hash: RwLock::new(None),
            current_time: RwLock::new(0.0),
            is_playing: RwLock::new(false),
            members: DashMap::new(),
            broadcast: tx,
            created_at: chrono::Utc::now().timestamp(),
        });

        room.members.insert(
            host_id.clone(),
            RoomMember {
                user_id: host_id.clone(),
                username: host_name.clone(),
                is_ready: true,
            },
        );

        self.rooms.insert(room_id.clone(), room.clone());

        tracing::info!("Room created in memory: id={}, name={}, host={}, total_rooms={}", room_id, name, host_name, self.rooms.len());

        room
    }

    pub fn get_room(&self, room_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(room_id).map(|r| r.clone())
    }

    pub fn join_room(
        &self,
        room_id: &str,
        user_id: String,
        username: String,
    ) -> Option<Arc<Room>> {
        if let Some(room) = self.rooms.get(room_id) {
            room.members.insert(
                user_id.clone(),
                RoomMember {
                    user_id: user_id.clone(),
                    username: username.clone(),
                    is_ready: false,
                },
            );

            tracing::info!("User joined room: room_id={}, user_id={}, username={}, members={}", room_id, user_id, username, room.members.len());

            Some(room.clone())
        } else {
            tracing::warn!("Join room failed: room_id={} not found", room_id);
            None
        }
    }

    pub fn leave_room(&self, room_id: &str, user_id: &str) {
        // 使用 remove_if 原子地检查空房间并删除，避免 TOCTOU 竞态
        let removed = self.rooms.remove_if(room_id, |_, room| {
            room.members.remove(user_id);

            // 广播用户离开
            let _ = room.broadcast.send(WsMessage::UserLeft {
                user_id: user_id.to_string(),
            });

            if room.members.is_empty() {
                tracing::info!("Room empty, removing: room_id={}", room_id);
                true // 原子删除
            } else {
                // 如果房主离开，转移房主
                let current_host = room.host_id.read().map(|h| h.clone()).unwrap_or_default();
                if current_host == user_id {
                    if let Some(first) = room.members.iter().next() {
                        let new_host = first.user_id.clone();
                        tracing::info!("Host transferred: room_id={}, old={}, new={}", room_id, user_id, new_host);
                        if let Ok(mut h) = room.host_id.write() { *h = new_host; }
                    }
                }

                tracing::info!("User left room: room_id={}, user_id={}, remaining={}", room_id, user_id, room.members.len());
                false // 保留房间
            }
        });

        if removed.is_some() {
            tracing::info!("Active rooms: {}", self.rooms.len());
        }
    }

    pub fn update_sync_state(&self, room_id: &str, timestamp: f64, is_playing: bool) {
        if let Some(room) = self.rooms.get(room_id) {
            if let Ok(mut t) = room.current_time.write() { *t = timestamp; }
            if let Ok(mut p) = room.is_playing.write() { *p = is_playing; }
        }
    }

    /// Remove empty rooms and rooms older than max_age_secs with no active playback
    pub fn cleanup_stale_rooms(&self, max_age_secs: i64) {
        let now = chrono::Utc::now().timestamp();
        let mut to_remove = Vec::new();

        for entry in self.rooms.iter() {
            let room = entry.value();
            // Remove empty rooms
            if room.members.is_empty() {
                to_remove.push(entry.key().clone());
                continue;
            }
            // Remove rooms older than max age with no active playback
            if now - room.created_at > max_age_secs {
                let is_playing = room.is_playing.read().map(|v| *v).unwrap_or(false);
                if !is_playing {
                    to_remove.push(entry.key().clone());
                }
            }
        }

        for id in &to_remove {
            tracing::info!("Cleaning up stale room: id={}", id);
            self.rooms.remove(id);
        }

        if !to_remove.is_empty() {
            tracing::info!("Cleaned up {} stale room(s), active rooms: {}", to_remove.len(), self.rooms.len());
        }
    }

    /// Get total number of active rooms
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}
