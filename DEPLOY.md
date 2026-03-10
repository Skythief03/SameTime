# SameTime 部署指南

## 目录
- [服务端部署](#服务端部署)
- [客户端安装](#客户端安装)
- [常见问题 FAQ](#常见问题-faq)

---

## 服务端部署

### 方式一：Docker 部署（推荐）

**前置要求：** Docker 和 Docker Compose

```bash
cd server

# 生成安全配置文件（首次部署时执行）
echo "JWT_SECRET=$(openssl rand -base64 48)" > .env
echo "SAMETIME_PORT=9527" >> .env
echo "SAMETIME_HOST=0.0.0.0" >> .env

# 查看生成的配置
cat .env

# 加载配置并启动
source .env && docker compose up -d

# 查看运行状态
docker compose ps

# 查看日志
docker compose logs -f sametime-server

# 停止服务
docker compose down
```

数据持久化在 Docker volume `sametime-data` 中，包含 SQLite 数据库和上传文件。

#### 代码更新后重新部署

```bash
cd server

# 拉取最新代码
git pull

# 重新构建镜像并启动（--build 强制重新编译）
docker compose up -d --build

# 确认新版本运行正常
docker compose ps
docker compose logs -f sametime-server
```

> 💡 `--build` 会利用 Docker 缓存，只有变更的代码层会重新编译，依赖层会复用缓存。
> 数据卷 `sametime-data` 不受影响，数据库和上传文件会保留。

### 方式二：手动编译部署

**前置要求：** Rust 1.75+、SQLite3

```bash
cd server

# 编译 Release 版本
cargo build --release

# 创建数据目录
mkdir -p data/uploads

# 生成安全配置文件（首次部署时执行一次）
cat > .env << 'EOF'
SAMETIME_HOST=0.0.0.0
SAMETIME_PORT=9527
DATABASE_URL=sqlite:./data/sametime.db?mode=rwc
UPLOAD_DIR=./data/uploads
MAX_FILE_SIZE=10737418240
JWT_EXPIRY_HOURS=168
EOF
echo "JWT_SECRET=$(openssl rand -base64 48)" >> .env

# 查看当前配置
cat .env

# 加载配置并启动
source .env && ./target/release/sametime-server
```

> 💡 后续重启只需 `source .env && ./target/release/sametime-server`，无需重新生成密钥。

### `.env` 配置文件管理

`.env` 文件存放在 `server/` 目录下，包含所有运行时配置。

```bash
# 查看当前配置
cat server/.env

# 修改某项配置（例如修改端口）
sed -i 's/SAMETIME_PORT=.*/SAMETIME_PORT=8443/' server/.env

# 重新生成 JWT 密钥（注意：会使所有已签发的 Token 失效）
sed -i '/^JWT_SECRET=/d' server/.env
echo "JWT_SECRET=$(openssl rand -base64 48)" >> server/.env
```

> ⚠️ **安全提示：**
> - `.env` 已在 `.gitignore` 中，**不会被提交到 Git 仓库**
> - 请勿将 `.env` 文件通过聊天、邮件等方式明文传输
> - 建议使用非默认端口（如 `9527`）减少扫描器自动探测
> - JWT 密钥长度建议 ≥ 48 字节（`openssl rand -base64 48`）

### 环境变量说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `SAMETIME_HOST` | `0.0.0.0` | 监听地址 |
| `SAMETIME_PORT` | `8080` | 监听端口 |
| `DATABASE_URL` | `sqlite:./data/sametime.db?mode=rwc` | SQLite 数据库路径 |
| `UPLOAD_DIR` | `./data/uploads` | 上传文件存储目录 |
| `MAX_FILE_SIZE` | `10737418240` (10GB) | 最大上传文件大小（字节） |
| `JWT_SECRET` | `sametime-dev-secret-...` | JWT 签名密钥（**生产环境必须修改**） |
| `JWT_EXPIRY_HOURS` | `168` (7天) | JWT Token 有效期 |

### 反向代理配置（Nginx 示例）

```nginx
server {
    listen 80;
    server_name sametime.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400s;
    }

    # 上传文件大小限制
    client_max_body_size 10G;
}
```

建议启用 HTTPS（使用 Let's Encrypt）以确保 WebSocket 和 WebRTC 连接安全。

### 健康检查

```bash
curl http://localhost:8080/api/health
# 应返回 200 OK
```

---

## 客户端安装

### 前置依赖

SameTime 客户端需要安装以下外部程序：

1. **MPV 播放器**（必须）
   - macOS: `brew install mpv`
   - Windows: 下载 https://mpv.io/installation/
   - Linux: `sudo apt install mpv`（Ubuntu/Debian）

2. **aria2**（可选，磁力链接下载）
   - macOS: `brew install aria2`
   - Windows: 下载 https://aria2.github.io/
   - Linux: `sudo apt install aria2`

### 从源码构建

**前置要求：** Node.js 18+、pnpm、Rust 1.75+

```bash
# 安装前端依赖
pnpm install

# 开发模式
pnpm tauri dev

# 构建安装包
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录：
- macOS: `.dmg` 安装包
- Windows: `.msi` / `.exe` 安装包
- Linux: `.deb` / `.AppImage` 安装包

### 首次使用

1. 启动 SameTime 客户端
2. 点击 **⚙️ 设置**
3. 配置服务器地址（如 `http://your-server:8080`）
4. 点击 **测试连接** 确认服务可达
5. 设置用户昵称
6. 点击 **保存设置**
7. 返回首页，创建或加入房间

---

## 常见问题 FAQ

### Q: WebSocket 连接失败？
**A:** 确认以下几点：
- 服务端已启动且端口可达
- 如使用反向代理，确保已配置 WebSocket 代理（`Upgrade` 和 `Connection` 头）
- 浏览器控制台查看具体错误信息

### Q: 视频不同步？
**A:** SameTime 使用房主时间戳同步，2 秒内的偏差视为正常。如果偏差较大：
- 确认所有人网络延迟在合理范围内
- 房主端播放器状态需正常（MPV 正在播放）
- 尝试房主手动暂停再播放触发重新同步

### Q: 上传文件失败？
**A:** 检查以下项：
- 文件大小未超过限制（默认 10GB）
- 文件格式在白名单内（mp4, mkv, avi, mov, wmv, flv, webm）
- 服务端磁盘空间充足
- 如使用 Nginx 反向代理，需设置 `client_max_body_size`

### Q: 语音通话无法连接？
**A:** WebRTC 需要以下条件：
- 浏览器/WebView 已授权麦克风访问
- 如在 NAT 环境下，可能需要 TURN 服务器
- 当前版本使用 P2P 直连，需确保客户端之间网络互通

### Q: 房间密码忘记了？
**A:** 当前版本无密码找回功能。可以直接创建新房间。如果是管理员，可直接操作 SQLite 数据库：
```bash
sqlite3 data/sametime.db "UPDATE rooms SET password_hash = NULL WHERE id = 'room_id';"
```

### Q: 如何备份数据？
**A:** 所有数据存储在 `server/data/` 目录：
- `sametime.db` — 数据库文件
- `uploads/` — 上传的视频文件

Docker 部署时对应 volume `sametime-data`，可用 `docker cp` 导出。

### Q: 生产环境安全建议？
1. **必须修改** `JWT_SECRET` 环境变量
2. 启用 HTTPS（WebSocket 需要 WSS、WebRTC 需要安全上下文）
3. 配置防火墙，仅开放必要端口
4. 定期备份数据库
5. 考虑设置上传文件大小限制