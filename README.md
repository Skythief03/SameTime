# 🎬 SameTime - 同步观影平台

<p align="center">
  <strong>与朋友一起，跨越距离，同步观影。</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.1.0-blue" alt="version" />
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-green" alt="platform" />
  <img src="https://img.shields.io/badge/license-MIT-orange" alt="license" />
</p>

---

## 📖 简介

SameTime 是一个桌面端同步观影平台，允许用户创建房间，邀请好友一起观看同一份视频。与传统直播不同，视频文件存储在每个用户本地，系统仅同步播放时间戳，大幅降低服务器带宽成本。

### ✨ 核心特性

| 特性 | 描述 |
|------|------|
| 🏠 **房间系统** | 创建/加入房间，支持密码保护，房主自动转移 |
| ⏱️ **时间戳同步** | 毫秒级播放进度同步，2秒漂移容差，自动校准 |
| 📺 **多视频源** | 本地文件、服务器上传、磁力链接 |
| 🔗 **文件校验** | SHA256 采样 hash 比对，无需上传即可确认文件一致性 |
| 🎤 **语音通话** | WebRTC P2P 实时语音，回声消除 + 降噪 |
| 💬 **文字聊天** | 实时消息收发 |
| 🎯 **弹幕系统** | Canvas 渲染引擎，轨道分配，平滑滚动 |
| 🔒 **安全可靠** | JWT 认证、bcrypt 密码加密、文件上传校验 |
| 🏗️ **自托管** | 单二进制部署，Docker 一键启动 |

### 🎯 适用场景

- 👫 异地情侣同步观影
- 👨‍👩‍👧‍?? 2-10 人朋友聚会观影
- 🎬 小型兴趣社群观影活动

---

## 🏗️ 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                  客户端 (Tauri 2.x + Vue 3)              │
├─────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │ 视频播放  │ │ 语音通话  │ │ 弹幕渲染  │ │ 聊天界面  │   │
│  │  (MPV)   │ │ (WebRTC) │ │ (Canvas) │ │  (Vue)   │   │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘   │
│       └─────────────┴────────────┴────────────┘         │
│              同步状态管理 (Pinia + WebSocket)              │
└────────────────────────┬────────────────────────────────┘
                         │ WebSocket / HTTP
┌────────────────────────┼────────────────────────────────┐
│                  服务端 (Rust + Axum)                     │
├────────────────────────┼────────────────────────────────┤
│  ┌──────────┐ ┌────────┴────────┐ ┌──────────┐         │
│  │ REST API │ │ WebSocket 信令   │ │ 文件存储  │         │
│  └────┬─────┘ └────────┬────────┘ └────┬─────┘         │
│       └────────────────┴───────────────┘                │
│                    SQLite 数据库                          │
└─────────────────────────────────────────────────────────┘
```

---

## ??️ 技术栈

### 客户端

| 技术 | 用途 |
|------|------|
| [Tauri 2.x](https://v2.tauri.app/) | 跨平台桌面框架 |
| [Vue 3](https://vuejs.org/) + TypeScript | 响应式 UI |
| [Pinia](https://pinia.vuejs.org/) | 状态管理 |
| [Vue Router](https://router.vuejs.org/) | 路由管理 |
| [TailwindCSS 3](https://tailwindcss.com/) | 原子化样式 |
| [Vite 5](https://vitejs.dev/) | 构建工具 |
| [MPV](https://mpv.io/) | 外部视频播放器 (IPC 控制) |
| WebRTC | P2P 语音通话 |

### 服务端

| 技术 | 用途 |
|------|------|
| [Rust](https://www.rust-lang.org/) | 系统编程语言 |
| [Axum 0.7](https://docs.rs/axum/) | Web 框架 (HTTP + WebSocket) |
| [Tokio](https://tokio.rs/) | 异步运行时 |
| [SQLite](https://www.sqlite.org/) + SQLx | 数据库 |
| [DashMap](https://docs.rs/dashmap/) | 并发内存状态 |
| [bcrypt](https://docs.rs/bcrypt/) | 密码加密 |
| [jsonwebtoken](https://docs.rs/jsonwebtoken/) | JWT 认证 |

---

## 📁 项目结构

```
SameTime/
├── src/                         # 前端源码 (Vue 3)
│   ├── components/
│   │   ├── chat/                # 聊天面板
│   │   ├── common/              # 通用组件 (Toast 通知)
│   │   ├── danmaku/             # Canvas 弹幕渲染
│   │   ├── player/              # 视频播放器
│   │   └── room/                # 房间成员列表
│   ├── composables/             # 组合式函数
│   │   ├── useSync.ts           # 播放同步逻辑
│   │   ├── useVoice.ts          # WebRTC 语音
│   │   └── useWebSocket.ts      # WebSocket 封装
│   ├── pages/                   # 页面
│   │   ├── Home.vue             # 首页 (创建/加入房间)
│   │   ├── Room.vue             # 观影房间
│   │   └── Settings.vue         # 设置页
│   ├── stores/                  # Pinia 状态
│   │   ├── player.ts            # 播放器状态
│   │   ├── room.ts              # 房间状态 + WebSocket
│   │   └── user.ts              # 用户认证
│   ├── types/                   # TypeScript 类型定义
│   └── utils/                   # 工具函数
├── src-tauri/                   # Tauri 桌面端 (Rust)
│   └── src/
│       ├── lib.rs               # 命令注册 + 文件 hash
│       ├── mpv.rs               # MPV 播放器控制
│       └── main.rs              # 入口
├── server/                      # 服务端 (Rust + Axum)
│   ├── src/
│   │   ├── main.rs              # 路由 + 启动
│   │   ├── config.rs            # 环境变量配置
│   │   ├── error.rs             # 错误处理
│   │   ├── db/                  # 数据库 schema
│   │   ├── handlers/            # 请求处理器
│   │   │   ├── auth.rs          # 注册/登录
│   │   │   ├── room.rs          # 房间 CRUD + 密码验证
│   │   │   ├── file.rs          # 文件上传/下载
│   │   │   └── ws.rs            # WebSocket 连接
│   │   ├── models/              # 数据模型
│   │   ├── services/            # 业务逻辑
│   │   │   └── room_manager.rs  # 内存房间管理
│   │   └── ws/                  # WebSocket 消息类型
│   ├── Dockerfile               # Docker 构建
│   └── docker-compose.yml       # Docker Compose 编排
├── DEPLOY.md                    # 部署指南
├── package.json
└── vite.config.ts
```

---

## 🚀 快速开始

### 前置依赖

- **Node.js** 18+
- **pnpm** (包管理器)
- **Rust** 1.75+ (含 cargo)
- **MPV** 播放器

```bash
# macOS
brew install mpv

# Ubuntu / Debian
sudo apt install mpv

# Windows
# 下载安装 https://mpv.io/installation/
```

### 启动服务端

```bash
cd server

# 编译并运行
cargo run

# 服务端默认监听 http://localhost:8080
# 健康检查: curl http://localhost:8080/api/health
```

### 启动客户端 (开发模式)

```bash
# 安装依赖
pnpm install

# 启动 Tauri 开发模式
pnpm tauri dev

# 或仅启动前端 (浏览器开发)
pnpm dev
```

### Docker 部署服务端

```bash
cd server

# 一键启动
docker compose up -d

# 查看日志
docker compose logs -f
```

---

## 📋 API 参考

### REST API

| 方法 | 路径 | 描述 |
|------|------|------|
| `GET` | `/api/health` | 健康检查 |
| `POST` | `/api/auth/register` | 用户注册 |
| `POST` | `/api/auth/login` | 用户登录 |
| `POST` | `/api/rooms` | 创建房间 |
| `GET` | `/api/rooms/:id` | 获取房间信息 |
| `POST` | `/api/rooms/:id/join` | 加入房间 (支持密码) |
| `POST` | `/api/files/upload` | 上传视频文件 |
| `GET` | `/api/files/:id` | 下载文件 |

### WebSocket

连接地址: `ws://host:8080/ws/:room_id?user_id=xxx&username=xxx`

**消息类型：**

| 类型 | 方向 | 描述 |
|------|------|------|
| `sync_request` | 客户端 → 服务端 | 房主发送同步请求 |
| `sync_broadcast` | 服务端 → 客户端 | 同步广播 (时间戳+播放状态) |
| `chat_message` | 双向 | 聊天消息 |
| `danmaku` | 双向 | 弹幕消息 |
| `voice_offer/answer` | 双向 | WebRTC 语音信令 |
| `ice_candidate` | 双向 | ICE 候选 |
| `video_hash` | 双向 | 视频文件 hash 广播与比对 |
| `user_joined/left` | 服务端 → 客户端 | 用户进出房间 |
| `ping/pong` | 心跳 | 30 秒间隔保活 |

---

## ⚙️ 环境变量

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `SAMETIME_HOST` | `0.0.0.0` | 监听地址 |
| `SAMETIME_PORT` | `8080` | 监听端口 |
| `DATABASE_URL` | `sqlite:./data/sametime.db?mode=rwc` | 数据库路径 |
| `UPLOAD_DIR` | `./data/uploads` | 上传文件目录 |
| `MAX_FILE_SIZE` | `10737418240` (10GB) | 最大上传大小 |
| `JWT_SECRET` | `sametime-dev-secret-...` | JWT 签名密钥 ⚠️ |
| `JWT_EXPIRY_HOURS` | `168` (7天) | Token 有效期 |

> ⚠️ **生产环境务必修改 `JWT_SECRET`！**

---

## 🔒 安全特性

- **认证**：JWT Token (HS256)，支持注册/登录/快速匿名模式
- **房间密码**：bcrypt 加密存储 (cost=10)，加入时服务端验证
- **文件上传**：扩展名白名单 + Magic Bytes 检测 + 文件名净化
- **文件校验**：SHA256 采样 hash（头/中/尾各 1MB + 文件大小），本地比对无需上传
- **WebSocket**：30 秒心跳保活 + 指数退避重连 (最多 8 次)
- **房间管理**：房主掉线自动转移 + 空房间定时清理 (60 秒巡检)

---

## 🖥️ 客户端打包

```bash
# 构建各平台安装包
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- **macOS**: `.dmg` 安装包
- **Windows**: `.msi` / `.exe` 安装包
- **Linux**: `.deb` / `.AppImage` 安装包

---

## 🔧 开发指南

### 前端开发

```bash
pnpm dev              # 启动 Vite 开发服务器 (localhost:1420)
pnpm build            # 构建前端产物到 dist/
pnpm tauri dev        # 启动 Tauri 开发模式 (含桌面窗口)
```

### 服务端开发

```bash
cd server
cargo run             # 启动开发服务器
cargo check           # 类型检查
cargo build --release # 编译 release 版本
```

### 项目依赖

**前端核心依赖：**
- `vue` ^3.4.0 / `pinia` ^2.2.0 / `vue-router` ^4.4.0
- `@tauri-apps/api` ^2.0.0 / `@tauri-apps/plugin-shell` ^2.0.0
- `tailwindcss` ^3.4.10 / `vite` ^5.4.0

**服务端核心依赖：**
- `axum` 0.7 (HTTP + WebSocket + Multipart)
- `tokio` 1 (异步运行时)
- `sqlx` 0.7 (SQLite)
- `dashmap` 5 (并发 HashMap)
- `bcrypt` 0.15 / `jsonwebtoken` 9

---

## 📝 详细部署文档

完整的部署说明（Docker、手动部署、Nginx 反向代理、FAQ）请参阅 **[DEPLOY.md](./DEPLOY.md)**。

---

## 🗺️ 路线图

### 近期
- [ ] 观影历史记录
- [ ] 播放列表支持
- [ ] 自定义弹幕样式
- [ ] 表情反应系统

### 中期
- [ ] Web 客户端
- [ ] 移动端 App
- [ ] 插件系统

### 远期
- [ ] 多房间管理后台
- [ ] AI 字幕翻译
- [ ] 云同步观影记录

---

## 📄 License

MIT License © 2024 SameTime Team