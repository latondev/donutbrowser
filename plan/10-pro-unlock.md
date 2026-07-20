# Pro Feature Unlock (Local Dev/Test Build)

> **Lưu ý pháp lý**: Donut Browser licensed AGPL-3.0. Patch này cho mục đích **học tập/thử nghiệm cá nhân, phi thương mại**. Không distribute binary, không rebrand, không dùng cho sản phẩm thương mại nếu không có written permission từ copyright holder (contact@donutbrowser.com). Xem `AGENTS.md` → "Proprietary Changes".

## Tổng quan

Pro features được gate qua `Entitlements` struct (Rust) và `getEntitlements()` (TS). Toàn bộ gate điểm đọc qua 2 hàm này, nên chỉ cần patch 2 file:

| File | Thay đổi |
|---|---|
| `src-tauri/src/cloud_auth.rs` | `entitlements()`, `has_active_paid_subscription()`, `can_use_browser_automation()`, `can_use_cross_os_fingerprints()`, `can_use_cloud_backup()`, `can_use_cloud_backup_sync()`, `requests_per_hour()` → luôn trả `true`/max |
| `src/lib/entitlements.ts` | `getEntitlements()` → luôn trả Pro-level entitlements |
| `src-tauri/src/wayfern_manager.rs` | Skip 3s wait loop cho wayfern token (không có cloud account → token không bao giờ đến) |

Mọi đoạn code gốc được giữ lại dưới dạng comment `/* ... */` hoặc trong block comment, đánh dấu `ponytail:` để dễ revert.

## Features được unlock

| Feature | Gate | Trạng thái |
|---|---|---|
| **Browser Automation API & MCP** | `can_use_browser_automation()` | ✅ unlocked |
| **Cross-OS fingerprints** | `can_use_cross_os_fingerprints()` | ✅ unlocked |
| **Profile Synchronizer (Wayfern)** | sync engine + scheduler | ✅ hoạt động (self-hosted sync server) |
| **Cloud backup (20 profiles)** | `can_use_cloud_backup()` | ✅ unlocked (cần self-hosted sync server) |
| **Priority support** | N/A (non-technical) | N/A |
| **Commercial use** | 14-day trial modal | ✅ không hiện (trial modal bị skip vì `crossOsUnlocked = true`) |
| **Unlimited local profiles** | `profile_limit` | ✅ 1,000,000 |
| **Wayfern browser engine** | wayfern token | ⚠️ launch được, không có token → cross-OS fingerprint spoofing yếu hơn |
| **Proxy/VPN support** | không gate | ✅ vốn free |
| **Profile Management API & MCP** | `can_use_browser_automation()` | ✅ unlocked |
| **Cookie & Extension Management** | không gate | ✅ vốn free |
| **Set as default browser** | không gate | ✅ vốn free |

## Cách build & chạy

### Yêu cầu
- Rust toolchain (rustup)
- Node.js + pnpm@11.10.0
- Wayfern browser binary (download trong app hoặc copy thủ công)

### Build dev
```bash
pnpm install
pnpm copy-proxy-binary    # copy donut-proxy binary
pnpm tauri dev            # chạy Tauri + Next.js dev
```

### Build release
```bash
pnpm tauri build
```

Release build ghi data vào `DonutBrowser` dir (debug → `DonutBrowserDev`).

## Cách dùng API (sau khi build)

### 1. Bật API server
Settings → Integrations → Local API → Enable
Hoặc set trong settings JSON:
```json
{ "api_enabled": true, "api_port": 8765 }
```

### 2. Lấy API token
Settings → Integrations → copy bearer token

### 3. Gọi API

#### Tạo profile
```bash
curl -X POST http://127.0.0.1:8765/v1/profiles \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Profile 1",
    "browser": "wayfern",
    "version": "<wayfern-version>",
    "proxy_id": null,
    "vpn_id": null
  }'
```

#### Launch profile (automation)
```bash
curl -X POST http://127.0.0.1:8765/v1/profiles/<id>/run \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{ "url": "https://example.com", "headless": false }'
```
→ Trả `remote_debugging_port` — dùng CDP để điều khiển browser.

#### Open URL trong profile đang chạy
```bash
curl -X POST http://127.0.0.1:8765/v1/profiles/<id>/open-url \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{ "url": "https://google.com" }'
```

#### Kill profile
```bash
curl -X POST http://127.0.0.1:8765/v1/profiles/<id>/kill \
  -H "Authorization: Bearer <token>"
```

#### Batch run/stop
```bash
curl -X POST http://127.0.0.1:8765/v1/profiles/batch/run \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{ "profile_ids": ["id1","id2"], "url": "https://example.com" }'
```

### 4. OpenAPI spec
Truy cập `http://127.0.0.1:8765/openapi.json` để xem đầy đủ endpoints.

## Cách dùng MCP (Claude Desktop)

### 1. Bật MCP server
Settings → Integrations → MCP Server → Enable

### 2. Add to Claude Desktop
Settings → Integrations → "Install for Claude Desktop"
(hoặc dùng nút "Copy MCP URL" và config thủ công)

### 3. Dùng trong Claude
Claude sẽ thấy tools: `create_profile`, `run_profile`, `stop_profile`, `open_url`, `list_profiles`, `delete_profile`, `import_profile`, etc.

## Điểm cần lưu ý

### Wayfern token
- Wayfern engine cần token từ `api.donutbrowser.com` để unlock **cross-OS fingerprint spoofing** đầy đủ
- Không có paid account → không fetch được token → browser vẫn launch nhưng fingerprint spoofing yếu hơn
- Patch đã skip 3s wait loop để launch nhanh
- **Không thể bypass hoàn toàn** vì token gen server-side (cần cloud API)

### Cloud sync
- `can_use_cloud_backup()` = true, nhưng sync cần sync server
- Dùng **self-hosted sync server** (`donut-sync/`, Docker) — miễn phí, không cần paid plan
- Hoặc dùng Donut cloud (paid) — sync tự động

### Commercial trial modal
- 14-day trial cho commercial use
- Modal bị skip tự động vì `crossOsUnlocked = true` (patch frontend)
- Không hiện nữa → không acknknowledge cần thiết

### Rate limit
- `requests_per_hour = i64::MAX` (Rust) / `100` (TS)
- Chokepoints trong `api_server.rs` / `mcp_server.rs` là **inert** (chưa enforce)
- Sẽ không bị rate limit

## Revert (khi muốn restore gốc)

3 file đã patch, tìm comment `ponytail:` để revert:

1. **`src-tauri/src/cloud_auth.rs`**:
   - `entitlements()` → restore body gốc (state.as_ref().map(...))
   - `has_active_paid_subscription*()`, `can_use_*()`, `requests_per_hour()` → restore body gốc
   - `derive_entitlements()` → xóa dòng `return fully_unlocked_entitlements();` ở nhánh `!active`
   - Xóa hàm `fully_unlocked_entitlements()`

2. **`src/lib/entitlements.ts`**:
   - Đổi tên `_getEntitlementsOriginal` → `getEntitlements`, xóa bản override

3. **`src-tauri/src/wayfern_manager.rs`**:
   - Uncomment block `/* ... */` (3s wait loop)
   - Đổi `let wayfern_token = ...` thành `let mut wayfern_token = ...`

## Files đã patch (tóm tắt diff)

```
src-tauri/src/cloud_auth.rs       # Entitlements luôn trả Pro-level
src/lib/entitlements.ts            # getEntitlements() luôn trả Pro-level
src-tauri/src/wayfern_manager.rs  # Skip 3s wayfern token wait loop
```

Tất cả thay đổi được đánh dấu `ponytail:` comment để dễ tìm khi revert.
