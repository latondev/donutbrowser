# Kiến trúc kỹ thuật

## Sơ đồ tổng thể

```
┌─────────────────────────────────────────────────────────────────┐
│                    Next.js Frontend (webview)                    │
│  src/app/page.tsx · src/components/* · src/hooks/* · src/lib/*   │
│   Giao tiếp qua: Tauri invoke() + Tauri events listen()          │
└────────────────────────────┬────────────────────────────────────┘
                             │  IPC (tauri::command + emit)
                             │  ~230+ Tauri commands (lib.rs)
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Tauri Main Process (Rust)                    │
│  src-tauri/src/lib.rs — run() entry, setup, invoke_handler        │
│                                                                  │
│  ┌────────────┐ ┌─────────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ profile/   │ │ proxy_*     │ │ vpn/     │ │ sync/        │   │
│  │ manager    │ │ manager     │ │ wireguard│ │ engine       │   │
│  │ password   │ │ storage     │ │ tunnel   │ │ scheduler    │   │
│  │ encryption │ │ server      │ │ socks5   │ │ encryption   │   │
│  │ clear_on…  │ │ runner      │ │ storage  │ │ manifest     │   │
│  └────────────┘ └─────────────┘ └──────────┘ └──────────────┘   │
│  ┌────────────┐ ┌─────────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ wayfern_   │ │ browser_     │ │ api_     │ │ mcp_server   │   │
│  │ manager    │ │ runner       │ │ server   │ │ (MCP)        │   │
│  │ (CDP ws)   │ │ version_mgr  │ │ (axum)   │ │              │   │
│  └────────────┘ └─────────────┘ └──────────┘ └──────────────┘   │
│  ┌────────────┐ ┌─────────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ extension_ │ │ group_       │ │ cookie_  │ │ dns_blocklist│   │
│  │ manager    │ │ manager      │ │ manager  │ │ (Hagezi)     │   │
│  └────────────┘ └─────────────┘ └──────────┘ └──────────────┘   │
│  ┌────────────┐ ┌─────────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ cloud_auth │ │ commercial_  │ │ team_lock│ │ traffic_stats│   │
│  │            │ │ license      │ │          │ │              │   │
│  └────────────┘ └─────────────┘ └──────────┘ └──────────────┘   │
└─────────┬──────────────┬────────────────┬────────────────────────┘
          │              │                │
          ▼              ▼                ▼
   ┌──────────┐   ┌───────────┐    ┌────────────────┐
   │ Wayfern  │   │ donut-    │    │ donut-sync     │
   │ browser  │   │ proxy     │    │ (NestJS, S3)   │
   │ (Chromium│   │ (Rust bin)│    │ self-hostable  │
   │  fork)   │   │           │    │                │
   └──────────┘   └─────┬─────┘    └────────────────┘
                         │
                         ▼
                  Upstream proxy / SOCKS5 / Shadowsocks
                  (per profile, có geolocation check)
```

## Luồng giao tiếp

### 1. Frontend → Backend (Tauri commands)
- Next.js gọi `invoke("command_name", { args })` từ `@tauri-apps/api/core`
- ~230+ commands đăng ký trong `lib.rs::invoke_handler!`
- Command trả `Result<T, String>` — error là JSON `{"code": "..."}` string để frontend translate

### 2. Backend → Frontend (Tauri events)
- Rust emit event qua `events::emit("event-name", payload)` (`src-tauri/src/events/mod.rs`)
- Frontend listen qua `listen("event-name", callback)` từ `@tauri-apps/api/event`
- Global emitter pattern: `GLOBAL_EMITTER` OnceLock, set 1 lần ở startup
- Các event chính: `profile-running-changed`, `show-toast`, `close-confirm-requested`, `traffic-stats-changed`, `vpn-configs-changed`, `app-update-available`

### 3. Background tasks (spawn trong `lib.rs::run().setup`)
| Task | Interval | Mục đích |
|---|---|---|
| Version updater | background | Kiểm tra browser version updates |
| Auto-update check | 3h | Kiểm tra app update từ GitHub releases |
| Browser status broadcast | 5s (active) / 30s (idle) | Quét sysinfo, emit running state |
| Proxy cleanup | 30s | Cleanup proxy worker cho dead browser PID |
| Binary cleanup | 12h | Xóa browser binary không dùng |
| DNS blocklist refresh | 12h | Refresh Hagezi blocklists stale |
| GeoIP download | startup + 2s delay | Download GeoIP DB nếu thiếu |
| Sync scheduler | event-driven | Đồng bộ profile/proxy/vpn/group/extension |
| Cloud auth refresh | background | Refresh sync token + wayfern token |

## Kiến trúc sync (chi tiết ở `03-sync.md`)

```
Profile dir (Chromium)  ──manifest (hash+size diff)──>  S3-compatible
                        ──content-hashManifest──────>  (donut-sync hoặc Donut cloud)
Config JSON entities   ──whole JSON blob, 1 entity/obj─> updated_at: last-write-wins
```

## Kiến trúc proxy (chi tiết ở `05-proxy-vpn.md`)

```
Browser → donut-proxy (local, per-profile worker) → upstream proxy (HTTP/SOCKS5/SS)
                                                     ↓
                                               VPN worker (WireGuard) → SOCKS5 local
```

## Singleton pattern
- Nhiều manager dùng `lazy_static!` hoặc `OnceLock` global singleton
- Ví dụ: `PROXY_MANAGER`, `ProfileManager::instance()`, `WayfernManager::instance()`, `McpServer::instance()`, `SyncScheduler`
- Phải init/set global emitter trước khi dùng các manager emit events

## App data directory
- Release: `DonutBrowser` / Debug: `DonutBrowserDev` (`app_dirs.rs::app_name()`)
- macOS: `~/Library/Application Support/DonutBrowser/`
- Linux: `~/.local/share/DonutBrowser/`
- Windows: `%LOCALAPPDATA%\DonutBrowser\`
- Override qua `DONUTBROWSER_DATA_ROOT` env var

## Logs
- **GUI/Tauri**: `~/Library/Logs/com.donutbrowser/DonutBrowser.log` (rotated, 5MB/file)
- **donut-proxy worker**: `$TMPDIR/donut-proxy-<config_id>.log` (1 file per worker)
- Dev builds → `DonutBrowserDev.log`
