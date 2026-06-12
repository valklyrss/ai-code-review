# AI 代码旁路审核平台

这是一个独立外挂式代码审核平台，不修改 GitLab，不依赖 GitLab Webhook、CI、API 或 MR 评论。系统通过轮询 Git 仓库分支发现新 commit，拉取 mirror 仓库，计算 diff，调用 OpenAI-compatible AI 模型审核，并将任务、文件、问题结果保存到 SQLite。

## 功能特性

- 多仓库、多分支规则扫描，支持 `*`、`dev,master`、`release-*`。
- 后台定时扫描，默认 60 秒；worker 默认并发 1，适合 2G 内存小服务器。
- 所有 Git 操作通过系统 `git` 命令执行，带 timeout。
- 每个文件单独调用 AI 审核，支持 DeepSeek 等 OpenAI-compatible API。
- HIGH / CRITICAL 问题可通过 SMTP 邮件通知提交人和仓库负责人。
- Vue3 + Element Plus 前端展示仓库、任务、任务详情、问题清单。
- Rust Axum 后端托管前端 build 后的静态文件。

## 技术栈

后端：Rust、Axum、Tokio、SQLite、SQLx、Reqwest、Serde YAML、Lettre、Tracing、Tower HTTP。

前端：Vue 3、Vite、TypeScript、Element Plus、Axios。

## 本地开发

```bash
cd ai-code-review/backend
cp config.example.yaml config.yaml
cargo run -- --config ./config.yaml
```

另开终端启动前端：

```bash
cd ai-code-review/frontend
npm install
npm run dev
```

前端开发服务器默认访问 `http://127.0.0.1:5173`，API 会代理到 `http://127.0.0.1:18080`。

## 构建部署

```bash
cd ai-code-review/frontend
npm install
npm run build

cd ../backend
cargo build --release
cp config.example.yaml config.yaml
./target/release/ai-code-review-backend --config ./config.yaml
```

`npm run build` 会把前端产物输出到 `backend/static/`，由 Rust 服务托管。访问非 `/api` 路径时会回退到 `index.html`，支持 Vue Router history 模式。

## 配置说明

主要配置在 `backend/config.yaml`：

- `server.host/port`：Web 服务监听地址。
- `database.url`：SQLite 地址，默认 `sqlite:../data/ai-review.db`。
- `scanner.interval_seconds`：定时扫描间隔。
- `scanner.max_concurrent_tasks`：审核 worker 并发数，默认建议 `1`。
- `scanner.max_diff_lines/max_file_diff_lines`：diff 大小保护。
- `git.command_path`：系统 git 路径。
- `git.repo_base_dir`：mirror 仓库目录。
- `ai.base_url/api_key/model`：OpenAI-compatible API 配置。
- `mail.enabled`：是否启用 SMTP 告警。
- `review.allowed_extensions/ignore_paths/ignore_extensions`：审核文件过滤规则。

日志不会主动打印 `ai.api_key`、`mail.password`，Git URL 中常见 token 参数会脱敏。

## SQLite 初始化

后端启动时会自动创建数据库文件并运行 `backend/migrations/001_init.sql`。不需要手动执行 SQL。

## Git SSH Key

推荐使用 SSH 地址，例如：

```text
git@192.168.1.10:group/project.git
```

在运行服务的 Linux 用户下配置私钥：

```bash
ssh-keygen -t ed25519 -C ai-review
cat ~/.ssh/id_ed25519.pub
ssh -T git@192.168.1.10
```

确保该用户执行 `git ls-remote --heads <repo_url>` 成功。

## DeepSeek API

示例：

```yaml
ai:
  provider: "openai-compatible"
  base_url: "https://api.deepseek.com/v1"
  api_key: "sk-..."
  model: "deepseek-chat"
```

如果 `api_key` 为空，V1 会跳过实际 AI 调用并返回无问题结果，方便先验证 Git 扫描链路。

## SMTP 邮件

```yaml
mail:
  enabled: true
  smtp_host: "smtp.example.com"
  smtp_port: 465
  username: "ai-review@example.com"
  password: "..."
  from: "AI代码审核 <ai-review@example.com>"
```

只有存在 HIGH / CRITICAL 问题时才发送邮件。邮件发送失败不会让审核任务失败，只记录日志。

## systemd 示例

```ini
[Unit]
Description=AI Code Review
After=network.target

[Service]
WorkingDirectory=/opt/ai-code-review/backend
ExecStart=/opt/ai-code-review/backend/ai-code-review-backend --config /opt/ai-code-review/backend/config.yaml
Restart=always
RestartSec=5
User=ai-review

[Install]
WantedBy=multi-user.target
```

## 常见问题

- `git ls-remote` 失败：检查 SSH key、known_hosts、仓库地址和运行服务的用户权限。
- AI 审核无结果：检查 `ai.api_key`、`base_url`、模型名和网络连通性。
- 前端刷新 404：确认访问的是 Rust 后端端口，且已执行 `npm run build`。
- SQLite 锁：V1 默认单 worker，建议保持 `max_concurrent_tasks: 1`。
