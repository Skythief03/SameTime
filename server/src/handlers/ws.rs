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
            let err_msg = serde_json::to_string(&WsMessage::Error {
                message: "Room not found".to_string(),
            })
            .unwrap_or_default();
            let _ = sender.send(Message::Text(err_msg.into())).await;
            return;
        }
    };

    // 广播用户加入
    let _ = room.broadcast.send(WsMessage::UserJoined {
        user_id: user_id.clone(),
        username: username.clone(),
    });

    let mut rx = room.broadcast.subscribe();

    // Task: 从 broadcast channel 接收消息并转发给客户端
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text.into())).await.is_err() {
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
                            WsMessage::ChatMessage { .. }
                            | WsMessage::Danmaku { .. } => {
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::VoiceOffer { .. }
                            | WsMessage::VoiceAnswer { .. }
                            | WsMessage::IceCandidate { .. } => {
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            WsMessage::ReadyStateChanged { .. }
                            | WsMessage::VideoHash { .. } => {
                                let _ = broadcast_tx.send(ws_msg);
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
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

    tracing::info!("User {} disconnected from room {}", user_id, room_id);
}