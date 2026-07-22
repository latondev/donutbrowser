# Extensions

Quản lý browser extension: liệt kê, xóa, tạo extension group.

---

## GET /v1/extensions

Liệt kê tất cả extension đã cài.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/extensions
```

### Response

```json
{
  "value": [],
  "Count": 0
}
```

Khi có extension:

```json
{
  "value": [
    {
      "id": "uuid-here",
      "name": "uBlock Origin",
      "version": "1.62.0",
      "path": "/path/to/extension",
      "enabled": true
    }
  ],
  "Count": 1
}
```

### Fields

| Field | Type | Mô tả |
|-------|------|-------|
| `id` | string (UUID) | ID extension |
| `name` | string | Tên extension |
| `version` | string | Version |
| `path` | string | Đường dẫn thư mục extension |
| `enabled` | boolean | Đang bật hay không |

---

## DELETE /v1/extensions/{id}

Xóa extension.

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/extensions/<EXTENSION_ID>
```

### Response

```
HTTP 204 No Content
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Extension không tồn tại |

---

## Extension Groups

### GET /v1/extension-groups

Liệt kê tất cả extension group.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/extension-groups
```

### Response

```json
{
  "value": [],
  "Count": 0
}
```

---

### POST /v1/extension-groups

Tạo extension group mới.

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-extension-group"}' \
  http://127.0.0.1:10108/v1/extension-groups
```

### Request Body

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `name` | Có | Tên group (không rỗng) |

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `name` rỗng |
| 409 | Group đã tồn tại |

---

### GET /v1/extension-groups/{id}

Xem chi tiết extension group.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/extension-groups/<GROUP_ID>
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Group không tồn tại |

---

### DELETE /v1/extension-groups/{id}

Xóa extension group.

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/extension-groups/<GROUP_ID>
```

### Response

```
HTTP 204 No Content
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Group không tồn tại |
