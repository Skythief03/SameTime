use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;

use crate::ws::message::WsMessage;
use crate::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    pub user_id: Option<String>,
    pub username: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    let user_id = query.user_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let username = query.username.unwrap_or_else(|| "匿名用户".to_string());

    tracing::info!("WebSocket upgrade: room_id={}, user_id={}, username={}", room_id, user_id, username);

    ws.on_upgrade(move |socket| handle_socket(socket, room_id, user_id, username, state))
}

async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    user_id: String,
    username: String,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();

    let room = match state.room_manager.get_room(&room_id) {
        Some(r) => r,
        None => {
            tracing::warn!("WebSocket connect failed: room_id={} not found, user_id={}", room_id, user_id);
            let err_msg = serde_json::to_string(&WsMessage::Error {
                message: "Room not found".to_string(),
            })
            .unwrap_or_default();
            let _ = sender.send(Message::Text(err_msg.into())).await;
            return;
        }
    };

    tracing::info!("User connected: room_id={}, user_id={}, username={}", room_id, user_id, username);

    // 广播用户加入
    let _ = room.broadcast.send(WsMessage::UserJoined {
        user_id: user_id.clone(),
        username: username.clone(),
    });

    let mut rx = room.broadcast.subscribe();

    // Task: 从 broadcast channel 接收消息并转发给客户端
    let send_user_id = user_id.clone();
    let send_room_id = room_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text.into())).await.is_err() {
                    tracing::debug!("WebSocket send failed, closing: room_id={}, user_id={}", send_room_id, send_user_id);
                    break;
                }
            }
        }
    });

    // Task: 从客户端接收消息，处理后广播
    let broadcast_tx = room.broadcast.clone();
    let room_manager = state.room_manager.clone();
    let recv_room_id = room_id.clone();
    let recv_user_id = user_id.clone();

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let text_str: &str = &text;
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(text_str) {
                        match &ws_msg {
                            WsMessage::SyncRequest {
                                timestamp,
                                is_playing,
                                ..
                            } => {
                                tracing::debug!("Sync: room_id={}, user_id={}, time={:.1}, playing={}", recv_room_id, recv_user_id, timestamp, is_playing);
                                // 更新房间同步状态
                                room_manager.update_sync_state(
                                    &recv_room_id,
                                    *timestamp,
                                    *is_playing,
                                );
                                // 广播同步消息，携带发送者信息
                                let _ = broadcast_tx.send(WsMessage::SyncBroadcast {
                                    timestamp: *timestamp,
                                    is_playing: *is_playing,
                                    sender_id: recv_user_id.clone(),
                                });
                            }
                            WsMessage::ChatMessage { content, sender_name, .. } => {
                                tracing::info!("Chat: room_id={}, from={}, content={}", recv_room_id, sender_name, content);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::Danmaku { content, sender_name, .. } => {
                                tracing::info!("Danmaku: room_id={}, from={}, content={}", recv_room_id, sender_name, content);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::VoiceOffer { target_user_id, .. } => {
                                tracing::debug!("VoiceOffer: room_id={}, from={}, to={}", recv_room_id, recv_user_id, target_user_id);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::VoiceAnswer { target_user_id, .. } => {
                                tracing::debug!("VoiceAnswer: room_id={}, from={}, to={}", recv_room_id, recv_user_id, target_user_id);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::IceCandidate { target_user_id, .. } => {
                                tracing::debug!("IceCandidate: room_id={}, from={}, to={}", recv_room_id, recv_user_id, target_user_id);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::ReadyStateChanged { user_id, is_ready } => {
                                tracing::info!("ReadyState: room_id={}, user_id={}, ready={}", recv_room_id, user_id, is_ready);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::VideoHash { sender_name, video_hash, .. } => {
                                tracing::info!("VideoHash: room_id={}, from={}, hash={}", recv_room_id, sender_name, &video_hash[..16.min(video_hash.len())]);
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::Ping {} => {
                                let _ = broadcast_tx.send(WsMessage::Pong {});
                            }
                            _ => {
                                tracing::debug!("Unknown message from user_id={} in room_id={}", recv_user_id, recv_room_id);
                            }
                        }
                    } else {
                        tracing::warn!("Failed to parse WebSocket message: room_id={}, user_id={}, text={}", recv_room_id, recv_user_id, &text_str[..100.min(text_str.len())]);
                    }
                }
                Message::Close(_) => {
                    tracing::debug!("WebSocket close frame: room_id={}, user_id={}", recv_room_id, recv_user_id);
                    break;
                }
                _ => {}
            }
        }
    });

    // 等待任一任务结束
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // 清理：用户离开房间
    let _ = state
        .room_manager
        .leave_room(&room_id, &user_id);

    // 广播用户离开（获取房间可能已经不存在，忽略错误）
    if let Some(room) = state.room_manager.get_room(&room_id) {
        let _ = room.broadcast.send(WsMessage::UserLeft {
            user_id: user_id.clone(),
        });
    }

    tracing::info!("User disconnected: room_id={}, user_id={}, username={}", room_id, user_id, username);
}
