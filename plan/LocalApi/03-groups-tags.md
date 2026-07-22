# Groups & Tags

Quản lý group (nhóm profile) và tag.

---

## Groups

### GET /v1/groups

Liệt kê tất cả group.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/groups
```

### Response

```json
{
  "value": [
    {
      "id": "227fae7d-a680-4d0b-95e1-14f444334b96",
      "name": "OtherGroup",
      "profile_count": 0
    }
  ],
  "Count": 1
}
```

### Fields

| Field | Type | Mô tả |
|-------|------|-------|
| `id` | string (UUID) | ID group |
| `name` | string | Tên group |
| `profile_count` | number | Số profile trong group |

---

### POST /v1/groups

Tạo group mới.

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-group"}' \
  http://127.0.0.1:10108/v1/groups
```

### Request Body

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `name` | Có | Tên group (không rỗng) |

### Response

```json
{
  "id": "227fae7d-a680-4d0b-95e1-14f444334b96",
  "name": "my-group",
  "profile_count": 0
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `name` rỗng |
| 409 | Group đã tồn tại (`GROUP_ALREADY_EXISTS`) |

---

### GET /v1/groups/{id}

Xem chi tiết group.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/groups/227fae7d-a680-4d0b-95e1-14f444334b96
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Group không tồn tại |

---

### PUT /v1/groups/{id}

Đổi tên group.

```bash
curl -X PUT -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"renamed-group"}' \
  http://127.0.0.1:10108/v1/groups/227fae7d-a680-4d0b-95e1-14f444334b96
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Group không tồn tại |
| 400 | `name` rỗng |
| 409 | Tên đã tồn tại |

---

### DELETE /v1/groups/{id}

Xóa group. Profile trong group sẽ bị hủy gắn (không bị xóa).

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/groups/227fae7d-a680-4d0b-95e1-14f444334b96
```

### Response

```
HTTP 204 No Content
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Group không tồn tại |

---

## Tags

### GET /v1/tags

Liệt kê tất cả tag đang dùng.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/tags
```

### Response

```json
{
  "value": [],
  "Count": 0
}
```

Tag được tạo ngầm khi gắn vào profile (qua `PUT /v1/profiles/{id}` với field `tags`). Không có endpoint tạo/sửa/xóa tag trực tiếp — tag tự động xóa khi không còn profile nào dùng.
