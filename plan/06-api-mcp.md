# API & MCP

## REST API Server (`src-tauri/src/api_server.rs`)

### Tổng quan
- **axum** 0.8 + **utoipa** 5 (OpenAPI) + **utoipa-axum**
- Chạy local, opt-in qua Settings (`api_enabled`, `api_port`)
- Auto-start ở startup nếu enabled
- OpenAPI spec được serve tại `/openapi.json`

### OpenAPI spec — QUAN TRỌNG
Spec đến từ `ApiDoc` derive (`#[derive(OpenApi)]` với `paths(...)`, `components(schemas(...))`, `tags(...)`), **KHÔNG** từ router. `OpenApiRouter`-generated spec bị discard (`let (v1_routes, _) = ...`).

→ Handler register trên router nhưng thiếu trong `ApiDoc` sẽ **biến mất khỏi spec** (đã xảy ra với extension + VPN-export endpoints).

**Mọi endpoint modification phải update spec cùng lúc:**
1. Giữ `#[utoipa::path]` annotation accurate (path, request body, mọi reachable response status)
2. Add/remove handler trong `ApiDoc::paths(...)` + schema types trong `components(schemas(...))`
3. Extend `openapi_*` regression tests trong `api_server.rs::tests`
4. `#[schema(value_type = Object)]` trên `Option<T>` erase optionality → dùng `value_type = Option<Object>` (hoặc drop attribute)

### Error status conventions
Manager errors qua `manager_error_response` — map message content → status, pass-through text as body:

| Status | Trường hợp |
|---|---|
| 401 | Missing/invalid bearer token (auth middleware; empty body) |
| 402 | 5 automation endpoints (`run`, `open-url`, `kill`, `batch/run`, `batch/stop`) không paid plan; expired-proxy (`PROXY_PAYMENT_REQUIRED`) |
| 404 | Entity not found (`… not found` / `*_NOT_FOUND`) |
| 400 | Validation, duplicates, empty names, invalid/unsupported/unavailable input |
| 409 | Conflicts: browser version đang download, profile locked bởi team member (run), browser running during cookie import |
| 500 | Internal failures (IO, network, poisoned locks) |

Error bodies: plain-text diagnostics; một số là JSON `{"code": ...}` strings (shared với Tauri commands, vd `NAME_CANNOT_BE_EMPTY`, `GROUP_ALREADY_EXISTS`).

### Endpoints chính (automation)
- `POST /v1/profiles/{id}/run` — launch browser (402 nếu không paid)
- `POST /v1/profiles/{id}/open-url` — open URL trong running profile
- `POST /v1/profiles/{id}/kill` — kill browser
- `POST /v1/profiles/batch/run` — batch launch
- `POST /v1/profiles/batch/stop` — batch stop

### Auth
- Bearer token (auth middleware)
- Token gen qua Settings, lưu trong app settings

## MCP Server (`src-tauri/src/mcp_server.rs`)

### Tổng quan
- **Model Context Protocol** server — cho AI agents (Claude Desktop, etc.) điều khiển Donut Browser
- HTTP-based, chạy local
- Opt-in qua Settings (`mcp_enabled`)
- Auto-start ở startup nếu enabled
- URL: `http://127.0.0.1:{port}/mcp/{token}`

### MCP integrations (`mcp_integrations.rs`)
- `list_mcp_agents()` — list agents với connection status
- `add_mcp_to_agent(agent_id)` — install MCP cho agent
- `remove_mcp_from_agent(agent_id)` — uninstall

### Claude Desktop integration
`add_mcp_to_claude_desktop_internal`:
1. Resolve MCP URL + token
2. Tạo extension dir: `Claude/Claude Extensions/local.mcpb.donut-browser.donut-browser/`
3. Write `manifest.json` (manifest_version 0.3, server type node)
4. Write `server/index.js` — Node bridge script (HTTP ↔ stdin/stdout JSON-RPC)
5. Update `extensions-installations.json` registry
- Platform paths:
  - macOS: `~/Library/Application Support/Claude/Claude Extensions/...`
  - Windows: `%APPDATA%/Claude/Claude Extensions/...`
  - Linux: `~/.config/Claude/Claude Extensions/...`

### MCP-only Tauri commands
Một số commands chỉ dùng qua MCP, không qua frontend (allowlisted trong `test_no_unused_tauri_commands`):
- `connect_vpn`, `disconnect_vpn`, `get_vpn_status`, `get_vpn_config`, `list_active_vpn_connections`
- `export_profile_cookies`, `update_extension`, `set_extension_sync_enabled`, `set_extension_group_sync_enabled`
- `get_team_lock_status`, `generate_sample_fingerprint`, `cloud_get_wayfern_token`, `cloud_refresh_wayfern_token`, `lock_profile`

### McpConfig
```rust
struct McpConfig {
  port: u16,
  token: String,
}
```
- Token gen qua `SettingsManager::get_mcp_token(app_handle)`
- `get_mcp_config(app_handle)` — return port + token (cho frontend hiển thị)

## Settings liên quan
```rust
// settings_manager.rs
struct AppSettings {
  api_enabled: bool,
  api_port: u16,
  mcp_enabled: bool,
  // ... nhiều field khác
}
```
- `get_app_settings` / `save_app_settings` — Tauri commands
- `get_sync_settings` / `save_sync_settings` — sync server URL + token
