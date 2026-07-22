# Proxies

Quản lý proxy: tạo, sửa, xóa, import danh sách proxy.

---

## GET /v1/proxies

Liệt kê tất cả proxy.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/proxies
```

### Response

```json
{
  "value": [
    {
      "id": "05308bcb-3f11-4f09-b6fd-fb17d0823b41",
      "name": "a2-1 month",
      "proxy_settings": {
        "proxy_type": "http",
        "host": "proxy17023.aproxy.id.vn",
        "port": 51319,
        "username": "liam455",
        "password": "ndc1mjq0nza4nw=="
      }
    },
    {
      "id": "1d90767f-19c4-4e5e-b6c1-71af34ab262c",
      "name": "Imported- Proxy 2",
      "proxy_settings": {
        "proxy_type": "http",
        "host": "10.0.0.1",
        "port": 8080,
        "username": null,
        "password": null
      }
    }
  ],
  "Count": 2
}
```

### Fields

| Field | Type | Mô tả |
|-------|------|-------|
| `id` | string (UUID) | ID proxy |
| `name` | string | Tên proxy |
| `proxy_settings.proxy_type` | string | `http`, `socks5`, `socks4` |
| `proxy_settings.host` | string | Host/IP proxy |
| `proxy_settings.port` | number | Port |
| `proxy_settings.username` | string\|null | Username (nếu có) |
| `proxy_settings.password` | string\|null | Password (nếu có) |

---

## POST /v1/proxies

Tạo proxy mới.

### Request Body

```json
{
  "name": "my-proxy",
  "proxy_settings": {
    "proxy_type": "http",
    "host": "127.0.0.1",
    "port": 8080,
    "username": "user",
    "password": "pass"
  }
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `name` | Có | Tên proxy (không rỗng) |
| `proxy_settings.proxy_type` | Có | `http`, `socks5`, `socks4` |
| `proxy_settings.host` | Có | Host/IP |
| `proxy_settings.port` | Có | Port (1-65535) |
| `proxy_settings.username` | Không | Username |
| `proxy_settings.password` | Không | Password |

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-proxy","proxy_settings":{"proxy_type":"http","host":"127.0.0.1","port":8080,"username":"user","password":"pass"}}' \
  http://127.0.0.1:10108/v1/proxies
```

### Response

```json
{
  "id": "05308bcb-3f11-4f09-b6fd-fb17d0823b41",
  "name": "my-proxy",
  "proxy_settings": {
    "proxy_type": "http",
    "host": "127.0.0.1",
    "port": 8080,
    "username": "user",
    "password": "pass"
  }
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `name` rỗng, `host` rỗng, `port` không hợp lệ |
| 400 | `proxy_type` không supported |

---

## GET /v1/proxies/{id}

Xem chi tiết một proxy.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/proxies/05308bcb-3f11-4f09-b6fd-fb17d0823b41
```

### Response

```json
{
  "id": "05308bcb-3f11-4f09-b6fd-fb17d0823b41",
  "name": "a2-1 month",
  "proxy_settings": {
    "proxy_type": "http",
    "host": "proxy17023.aproxy.id.vn",
    "port": 51319,
    "username": "liam455",
    "password": "ndc1mjq0nza4nw=="
  }
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Proxy không tồn tại |

---

## PUT /v1/proxies/{id}

Cập nhật proxy. Gửi tất cả field (PUT thay thế toàn bộ).

```bash
curl -X PUT -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"renamed-proxy","proxy_settings":{"proxy_type":"socks5","host":"10.0.0.1","port":1080,"username":null,"password":null}}' \
  http://127.0.0.1:10108/v1/proxies/05308bcb-3f11-4f09-b6fd-fb17d0823b41
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Proxy không tồn tại |
| 400 | `name` rỗng, field không hợp lệ |

---

## DELETE /v1/proxies/{id}

Xóa proxy. Nếu proxy đang được profile sử dụng, proxy sẽ bị hủy gắn khỏi profile (profile không bị xóa).

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/proxies/05308bcb-3f11-4f09-b6fd-fb17d0823b41
```

### Response

```
HTTP 204 No Content
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Proxy không tồn tại |

---

## POST /v1/proxies/import

Import danh sách proxy hàng loạt. Hỗ trợ nhiều format.

### Request Body

```json
{
  "format": "txt",
  "content": "127.0.0.1:8080:user:pass\n10.0.0.1:1080\nsocks5://user:pass@172.16.0.1:9050",
  "duplicate_strategy": "rename"
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `format` | Có | `txt` hoặc `json` |
| `content` | Có | Nội dung import |
| `duplicate_strategy` | Không | `rename` (mặc định), `skip`, `overwrite` |

### Format TXT

Mỗi dòng một proxy. Các format được hỗ trợ:

```
host:port
host:port:username:password
username:password@host:port
proxy_type://username:password@host:port
```

`proxy_type` mặc định là `http` nếu không chỉ định.

### Format JSON

```json
[
  {
    "name": "proxy-1",
    "proxy_type": "http",
    "host": "127.0.0.1",
    "port": 8080,
    "username": "user",
    "password": "pass"
  }
]
```

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"format":"txt","content":"127.0.0.1:8080:user:pass\n10.0.0.1:1080"}' \
  http://127.0.0.1:10108/v1/proxies/import
```

### Response

```json
{
  "imported_count": 2,
  "skipped_count": 0,
  "failed_count": 0,
  "results": [
    {
      "name": "Imported- Proxy 1",
      "host": "127.0.0.1",
      "port": 8080,
      "status": "imported",
      "proxy_id": "a7003e76-511a-453c-b092-ac9322d69700"
    },
    {
      "name": "Imported- Proxy 2",
      "host": "10.0.0.1",
      "port": 1080,
      "status": "imported",
      "proxy_id": "1d90767f-19c4-4e5e-b6c1-71af34ab262c"
    }
  ]
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `content` rỗng, `format` không hợp lệ |
