# Profiles

Quản lý profile trình duyệt: tạo, sửa, xóa, launch, kill, mở URL, import cookies, import profile, batch operation.

---

## GET /v1/profiles

Liệt kê tất cả profile.

### Request

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/profiles
```

### Response

```json
{
  "profiles": [
    {
      "id": "5794369f-326e-4f09-81e9-92dea92b21a3",
      "name": "a2",
      "browser": "wayfern",
      "version": "149.0.7827.116",
      "proxy_id": null,
      "launch_hook": null,
      "process_id": null,
      "last_launch": 1784715101,
      "release_type": "stable",
      "group_id": null,
      "tags": [],
      "is_running": false,
      "proxy_bypass_rules": [],
      "vpn_id": null,
      "clear_on_close": false
    }
  ],
  "total": 1
}
```

### Fields

| Field | Type | Mô tả |
|-------|------|-------|
| `id` | string (UUID) | ID duy nhất của profile |
| `name` | string | Tên hiển thị |
| `browser` | string | Loại browser (`wayfern`) |
| `version` | string | Version browser |
| `proxy_id` | string\|null | ID proxy gắn vào profile |
| `launch_hook` | string\|null | URL sẽ mở tự động khi launch |
| `process_id` | number\|null | PID nếu đang chạy |
| `last_launch` | number\|null | Unix timestamp lần launch cuối |
| `release_type` | string | `stable` hoặc `nightly` |
| `group_id` | string\|null | ID group |
| `tags` | string[] | Danh sách tag |
| `is_running` | boolean | Browser đang chạy hay không |
| `proxy_bypass_rules` | string[] | Rule bypass proxy |
| `vpn_id` | string\|null | ID VPN |
| `clear_on_close` | boolean | Xóa data khi đóng |

---

## POST /v1/profiles

Tạo profile mới.

### Request Body

```json
{
  "name": "my-profile",
  "browser": "wayfern",
  "version": "149.0.7827.116",
  "proxy_id": null,
  "vpn_id": null,
  "group_id": null,
  "tags": ["work", "personal"],
  "release_type": "stable",
  "launch_hook": "https://example.com",
  "wayfern_config": {}
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `name` | Có | Tên profile (không rỗng) |
| `browser` | Có | `wayfern` |
| `version` | Không | Version browser, mặc định lấy version mới nhất |
| `proxy_id` | Không | ID proxy muốn gắn |
| `vpn_id` | Không | ID VPN muốn gắn |
| `group_id` | Không | ID group |
| `tags` | Không | Mảng tag |
| `release_type` | Không | `stable` (mặc định) hoặc `nightly` |
| `launch_hook` | Không | URL mở khi launch (http/https) |
| `wayfern_config` | Không | Config Wayfern fingerprint |

### Ví dụ

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-profile","browser":"wayfern","version":"149.0.7827.116"}' \
  http://127.0.0.1:10108/v1/profiles
```

### Response

```json
{
  "profile": {
    "id": "bae7503a-b94c-4d4c-ba09-fe81c3b31d1a",
    "name": "my-profile",
    "browser": "wayfern",
    "version": "149.0.7827.116",
    "proxy_id": null,
    "launch_hook": null,
    "process_id": null,
    "last_launch": null,
    "release_type": "stable",
    "group_id": null,
    "tags": [],
    "is_running": false,
    "proxy_bypass_rules": [],
    "vpn_id": null,
    "clear_on_close": false
  }
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `name` rỗng |
| 400 | `browser` không supported |

---

## GET /v1/profiles/{id}

Xem chi tiết một profile.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3
```

### Response

```json
{
  "profile": {
    "id": "5794369f-326e-4f09-81e9-92dea92b21a3",
    "name": "a2",
    "browser": "wayfern",
    "version": "149.0.7827.116",
    "proxy_id": null,
    "launch_hook": null,
    "process_id": null,
    "last_launch": 1784715101,
    "release_type": "stable",
    "group_id": null,
    "tags": [],
    "is_running": false,
    "proxy_bypass_rules": [],
    "vpn_id": null,
    "clear_on_close": false
  }
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |

---

## PUT /v1/profiles/{id}

Cập nhật profile.

### Request Body

```json
{
  "name": "renamed-profile",
  "proxy_id": "05308bcb-3f11-4f09-b6fd-fb17d0823b41",
  "vpn_id": null,
  "group_id": "227fae7d-a680-4d0b-95e1-14f444334b96",
  "tags": ["work", "production"],
  "release_type": "stable",
  "launch_hook": "https://example.com/start",
  "version": "149.0.7827.116",
  "proxy_bypass_rules": ["localhost", "127.0.0.1"],
  "clear_on_close": true,
  "extension_group_id": null,
  "sync_mode": "manual"
}
```

Tất cả field đều tùy chọn — chỉ gửi field muốn thay đổi.

| Field | Mô tả |
|-------|-------|
| `name` | Tên mới (không rỗng) |
| `proxy_id` | Gắn/hủy proxy |
| `vpn_id` | Gắn/hủy VPN |
| `group_id` | Gắn vào group |
| `tags` | Thay thế danh sách tag |
| `release_type` | `stable` hoặc `nightly` |
| `launch_hook` | URL mở khi launch (http/https, hoặc null) |
| `version` | Đổi version browser |
| `proxy_bypass_rules` | Rule bypass proxy |
| `clear_on_close` | Xóa data khi đóng |
| `extension_group_id` | ID extension group |
| `sync_mode` | `manual` hoặc `auto` |

```bash
curl -X PUT -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"renamed","proxy_id":"05308bcb-3f11-4f09-b6fd-fb17d0823b41"}' \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 400 | `name` rỗng |

---

## DELETE /v1/profiles/{id}

Xóa profile. Browser phải đang **không chạy**.

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3
```

### Response

```
HTTP 204 No Content
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 400 | Browser đang chạy |
| 409 | Profile bị lock bởi team member |

---

## POST /v1/profiles/{id}/run

Launch browser cho profile. Trả về CDP port để debug.

### Request Body

```json
{
  "url": "https://example.com",
  "headless": false
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `url` | Không | URL mở khi launch |
| `headless` | Không | `true` = headless mode |

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"headless":false}' \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3/run
```

### Response

```json
{
  "profile_id": "5794369f-326e-4f09-81e9-92dea92b21a3",
  "remote_debugging_port": 56954,
  "headless": false
}
```

### CDP Debugging

Dùng `remote_debugging_port` để connect tới Chrome DevTools Protocol:

```bash
# Truy cập DevTools
http://127.0.0.1:56954

# WebSocket endpoint
ws://127.0.0.1:56954/devtools/browser/<SESSION_ID>
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 402 | Không có plan trả phí (automation endpoint) |
| 409 | Browser đang chạy / profile bị lock |

---

## POST /v1/profiles/{id}/kill

Tắt browser của profile.

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3/kill
```

### Response

```json
""
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 402 | Không có plan trả phí |
| 400 | Browser không chạy |

---

## POST /v1/profiles/{id}/open-url

Mở URL trong browser **đang chạy**.

### Request Body

```json
{
  "url": "https://example.com"
}
```

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}' \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3/open-url
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 402 | Không có plan trả phí |
| 400 | Browser không chạy |

---

## POST /v1/profiles/{id}/cookies/import

Import cookies vào profile. Browser phải đang **không chạy**.

### Request Body

```json
{
  "content": "# Netscape HTTP Cookie File\n.example.com\tTRUE\t/\tFALSE\t0\tname\tvalue\n"
}
```

Hỗ trợ 2 format:
- **Netscape** (`.txt`): `domain\tflag\tpath\tsecure\texpiration\tname\tvalue`
- **JSON**: Mảng object `[{"name":"...","value":"...","domain":"..."}]]`

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"content":"# Netscape HTTP Cookie File\n.example.com\tTRUE\t/\tFALSE\t0\tsession\tabc123"}' \
  http://127.0.0.1:10108/v1/profiles/5794369f-326e-4f09-81e9-92dea92b21a3/cookies/import
```

### Response

```json
{
  "cookies_imported": 1,
  "cookies_replaced": 0,
  "errors": []
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Profile không tồn tại |
| 409 | Browser đang chạy |

---

## POST /v1/profiles/import

Import profile từ thư mục ngoài (Chrome, Edge, Firefox, v.v.).

### Request Body

```json
{
  "duplicate_strategy": "rename",
  "group_id": null,
  "items": [
    {
      "source_path": "C:\\Users\\ADMIN\\AppData\\Local\\Google\\Chrome\\User Data",
      "browser_type": "chrome",
      "new_profile_name": "imported-chrome",
      "proxy_id": null,
      "vpn_id": null
    }
  ],
  "wayfern_config": {}
}
```

| Field | Mô tả |
|-------|-------|
| `duplicate_strategy` | `rename` (mặc định), `skip`, hoặc `overwrite` |
| `items` | Mảng profile cần import |
| `items[].source_path` | Đường dẫn User Data dir |
| `items[].browser_type` | `chrome`, `edge`, `brave`, `firefox`, v.v. |
| `items[].new_profile_name` | Tên profile mới (tùy chọn) |
| `wayfern_config` | Fingerprint config (tùy chọn) |

### Response

```json
{
  "imported_count": 1,
  "skipped_count": 0,
  "failed_count": 0,
  "results": [
    {
      "source_path": "C:\\Users\\...\\User Data",
      "name": "imported-chrome",
      "profile_id": "abc123",
      "status": "imported"
    }
  ]
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `items` rỗng hoặc path không hợp lệ |

---

## GET /v1/profiles/import/detect

Tự động detect profile có sẵn trên máy (Chrome, Edge, Brave, Firefox, v.v.).

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/profiles/import/detect
```

### Response

```json
{
  "profiles": [
    {
      "browser": "chrome",
      "description": "C:\\Users\\ADMIN\\AppData\\Local\\Google\\Chrome\\User Data",
      "mapped_browser": "wayfern",
      "name": "Default",
      "path": "C:\\Users\\ADMIN\\AppData\\Local\\Google\\Chrome\\User Data\\Default"
    }
  ],
  "total": 1
}
```

---

## POST /v1/profiles/batch/run

Launch nhiều profile cùng lúc.

### Request Body

```json
{
  "profile_ids": [
    "5794369f-326e-4f09-81e9-92dea92b21a3",
    "a42743ae-06ed-4e9d-b7ed-3892519fb28f"
  ],
  "url": "https://example.com",
  "headless": false
}
```

### Response

```json
{
  "results": [
    {
      "profile_id": "5794369f-326e-4f09-81e9-92dea92b21a3",
      "ok": true,
      "remote_debugging_port": 56954,
      "error": null
    },
    {
      "profile_id": "a42743ae-06ed-4e9d-b7ed-3892519fb28f",
      "ok": false,
      "remote_debugging_port": null,
      "error": "Browser already running"
    }
  ]
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 402 | Không có plan trả phí |

---

## POST /v1/profiles/batch/stop

Tắt nhiều profile cùng lúc.

### Request Body

```json
{
  "profile_ids": [
    "5794369f-326e-4f09-81e9-92dea92b21a3",
    "a42743ae-06ed-4e9d-b7ed-3892519fb28f"
  ]
}
```

### Response

```json
{
  "results": [
    {
      "profile_id": "5794369f-326e-4f09-81e9-92dea92b21a3",
      "ok": true,
      "error": null
    },
    {
      "profile_id": "a42743ae-06ed-4e9d-b7ed-3892519fb28f",
      "ok": false,
      "error": "Browser not running"
    }
  ]
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 402 | Không có plan trả phí |
