# SameTime 待修复 Bug 清单

> 生成时间：2026-03-10
> 基于代码审计结果整理，按严重程度排序

---

## HIGH 严重

### 1. 语音通话功能完全不工作
- **文件**: `src/pages/Room.vue`
- **描述**: `toggleVoice` 只调用了 `getUserMedia` 获取麦克风，但从未引入和使用 `src/composables/useVoice.ts` composable。麦克风音频采集后无处可去，没有建立 RTCPeerConnection，没有发送信令消息，也没有接收远端音频。用户看到麦克风图标变绿，但实际没有任何音频传输。

### 2. WebRTC 信令消息在前端被丢弃
- **文件**: `src/stores/room.ts` `handleMessage()`
- **描述**: WebSocket 消息处理的 switch-case 中没有 `voice_offer`、`voice_answer`、`ice_candidate` 三种类型的处理分支，这些消息会走到 `default` 分支被忽略。即使接入了 `useVoice`，客户端也收不到 WebRTC 信令。

### 3. WebRTC 信令消息广播给所有用户而非定向发送
- **文件**: `server/src/handlers/ws.rs`
- **描述**: `VoiceOffer`、`VoiceAnswer`、`IceCandidate` 消息都有 `target_user_id` 字段指定目标用户，但服务端直接 `broadcast_tx.send()` 广播给房间所有人。导致：(1) 隐私泄露（SDP 包含网络拓扑信息）；(2) 多人房间中信令会发给错误的 peer；(3) 浪费带宽。

---

## MEDIUM 严重

### 4. ~~重连链在首次重连失败后断裂~~ ✅ 已修复
- **文件**: `src/stores/room.ts`
- **描述**: `connectToRoom` 中引入了 `connected` 标志，`onclose` 仅在 `connected=true` 时才触发重连。但重连时调用的 `connectToRoom` 如果连接失败（服务器宕机），`connected` 始终为 `false`，`onclose` 不会再次调用 `attemptReconnect`，重连循环永久停止。
- **修复**: `.catch()` 中手动调用 `attemptReconnect` 继续重连链。

### 5. ~~聊天消息列表无上限，内存持续增长~~ ✅ 已修复
- **文件**: `src/components/chat/ChatPanel.vue`
- **描述**: `messages` 数组只增不减，每条收发消息都 push 进去。长时间观影+活跃聊天场景下内存会持续增长。弹幕层有 500 条上限裁剪，但聊天面板没有。
- **修复**: 添加 `MAX_MESSAGES=500` 上限，超出时裁剪旧消息。

### 6. ~~`leave_room` 存在 TOCTOU 竞态~~ ✅ 已修复
- **文件**: `server/src/services/room_manager.rs`
- **描述**: `leave_room` 先检查 `room.members.is_empty()`，然后 `self.rooms.remove(room_id)`。在检查和删除之间，另一个线程可能通过 `join_room` 加入了新成员，导致有成员的房间被删除。
- **修复**: 使用 DashMap `remove_if` 原子操作，在同一把锁内完成检查和删除。

### 7. ~~Content-Disposition header 注入~~ ✅ 已修复
- **文件**: `server/src/handlers/file.rs`
- **描述**: 下载时文件名直接拼入 `Content-Disposition` header：`format!("attachment; filename=\"{}\"", filename)`。`sanitize_filename` 未过滤双引号 `"`，含双引号的文件名可破坏 HTTP header 格式。
- **修复**: 下载时将文件名中的 `"` 和 `\` 替换为 `_`。

### 8. 上传接口无鉴权
- **文件**: `server/src/handlers/file.rs`
- **描述**: `upload` 函数没有鉴权中间件，`uploader_id` 硬编码为 `"anonymous"`。任何人都可以向服务器上传文件。

### 9. 前端上传请求未携带 Authorization header
- **文件**: `src/components/player/VideoPlayer.vue`
- **描述**: `uploadVideo` 中的 XHR 请求未设置 `Authorization` header，与 `createRoom`/`joinRoom` 等请求不一致。即使后端加了鉴权也会失败。

### 10. ~~弹幕 pendingDanmaku 只增不减，每帧遍历量持续增长~~ ✅ 已修复
- **文件**: `src/components/danmaku/DanmakuLayer.vue`
- **描述**: 弹幕被添加到 `pendingDanmaku` 后，即使已显示完毕也不会被移除。每帧渲染循环都要遍历全部 pending 弹幕检查是否需要显示，随时间推移性能下降。裁剪逻辑仅在超过 500 条时触发。
- **修复**: 弹幕激活后立即从 `pendingDanmaku` 中移除。

---

## LOW 严重

### 11. 全屏状态不同步
- **文件**: `src/stores/player.ts`
- **描述**: `toggleFullscreen` 未监听 `fullscreenchange` 事件。用户按 Esc 退出全屏后，`isFullscreen` 仍为 `true`，UI 状态与实际不符。

### 12. mpv 崩溃后轮询定时器不停止
- **文件**: `src/stores/player.ts`
- **描述**: `startPolling` 在 `loadVideo` 时启动，仅在 `cleanup()` 时停止。如果 mpv 进程崩溃或视频播放结束，轮询定时器持续运行，每 500ms 发起失败的 IPC 调用。

### 13. 注册用户名唯一性检查存在竞态
- **文件**: `server/src/handlers/auth.rs`
- **描述**: 先 SELECT 检查用户名是否存在，再 INSERT。两步之间另一个请求可能注册了相同用户名。如果数据库有 UNIQUE 约束会返回泛化错误而非友好提示。

### 14. 登录/注册接口无速率限制
- **文件**: `server/src/handlers/auth.rs`
- **描述**: 无任何频率限制，可被暴力破解密码或注册洪泛攻击。

### 15. 创建/加入房间前未检查用户名
- **文件**: `src/pages/Home.vue`
- **描述**: 未强制用户在创建或加入房间前设置用户名，可能以空用户名或 "匿名用户" 进入房间。

### 16. 设置页下载目录浏览按钮无响应
- **文件**: `src/pages/Settings.vue`
- **描述**: 下载目录的"浏览..."按钮没有绑定 `@click` 事件处理器，输入框为 `readonly`，用户无法通过 UI 选择下载目录。

### 17. 文件下载 UUID 前缀碰撞风险
- **文件**: `server/src/handlers/file.rs`
- **描述**: 通过 `file_id[..8]` 前缀匹配磁盘文件，8 位十六进制有 4 亿种组合，极小概率碰撞可能返回错误文件。

### 18. 多用户 hash 比对只存最后一个用户
- **文件**: `src/components/player/VideoPlayer.vue`
- **描述**: `remoteUserHash` 只保存最近一个远端用户的视频 hash。多人房间中后来者的 hash 会覆盖之前的，无法同时与所有成员比对。
