# `uh-devtool` — CLI / HTTP / Watch

## 缘由

`uh-devtool` 是 code viewer 的**运行时壳**。它把 `uh-devtool-analyze` 包装成 3 个用户入口：

- `analyze <PATH>` — 一次性分析单个文件或目录，输出 JSON。
- `serve` — 启动 HTTP server（默认 7700），前端通过 `/api/*` 拿数据。
- `watch` — `serve` + notify watcher，文件变更时增量 re-analyze。

为什么不直接让前端跑 `syn`：前端是 TS 浏览器环境，syn 是 Rust；分进程是必然。

为什么不引入 web 框架（axum / actix）：本工具只暴露 4 个 endpoint，tiny_http 够用。YAGNI。

## 解决的问题

1. **本地 code viewer 体验**——开发时跑 `uh-devtool watch` + `pnpm dev`，改 `///` 注释 2s 内浏览器自动更新。
2. **CI 友好**——`analyze` 子命令输出 JSON，可以接入 lint / 文档生成流水线。
3. **零部署**——纯本地工具，不需要 docker / 容器。

## 设计哲学

1. **lib 与 bin 分离**——所有逻辑在 `uh-devtool-analyze`，bin 只做参数解析 + IO。
2. **HTTP 极简**——4 个 endpoint：`/api/health`、`/api/index`、`/api/file?path=...`、内置占位 `/`。
3. **轮询而非推送**（v0.1）——前端 2s 轮询 `/api/index` + 当前文件，避免 SSE 复杂度。
4. **Watch 防抖**——200ms 内的事件折叠成一次 re-analyze，避免高频文件写入触发风暴。
5. **失败可见**——`analyze_dir` 失败的文件在 stderr 输出 warning，但不中断其他文件。

## 具体细节设计

### 子命令

```bash
uh-devtool analyze <PATH>            # JSON to stdout
uh-devtool serve [--root X --port N] # HTTP server
uh-devtool watch [--root X --port N] # serve + notify
```

### HTTP API

| Path | Method | 用途 |
|---|---|---|
| `/` | GET | 占位 HTML（真实 SPA 在 `apps/web/src/review/`） |
| `/api/health` | GET | `{"ok": true}` — 健康检查 |
| `/api/index` | GET | 文件摘要列表（用于文件树） |
| `/api/file?path=<rel>` | GET | 单文件完整分析（用于详情） |

### Watch 行为

- notify `RecursiveMode::Recursive` 监听 `--root` 目录。
- 事件类型 `Create / Modify / Remove` 触发 re-analyze。
- 200ms 防抖：高频写入只重算一次。
- 整个 state 在 `Arc<RwLock>` 里共享给 HTTP handler。

## 与前端的关系

`apps/web/vite.config.ts` 配置 `proxy: { "/api": "http://127.0.0.1:7700" }`——开发时 `pnpm dev` 把 `/api/*` 转发到 `uh-devtool serve`。
