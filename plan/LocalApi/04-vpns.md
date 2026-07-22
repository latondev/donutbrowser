# VPNs

Quản lý VPN (WireGuard): tạo, sửa, xóa, import, export config.

---

## GET /v1/vpns

Liệt kê tất cả VPN.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/vpns
```

### Response

```json
{
  "value": [],
  "Count": 0
}
```

Khi có VPN:

```json
{
  "value": [
    {
      "id": "uuid-here",
      "name": "my-vpn",
      "config": "[Interface]\nPrivateKey = xxx\nAddress = 10.0.0.2/32\n\n[Peer]\nPublicKey = yyy\nEndpoint = vpn.example.com:51820\nAllowedIPs = 0.0.0.0/0\nPersistentKeepalive = 25"
    }
  ],
  "Count": 1
}
```

### Fields

| Field | Type | Mô tả |
|-------|------|-------|
| `id` | string (UUID) | ID VPN |
| `name` | string | Tên VPN |
| `config` | string | WireGuard config file content |

---

## POST /v1/vpns

Tạo VPN mới từ WireGuard config.

### Request Body

```json
{
  "name": "my-vpn",
  "config": "[Interface]\nPrivateKey = abc123\nAddress = 10.0.0.2/32\n\n[Peer]\nPublicKey = def456\nEndpoint = vpn.example.com:51820\nAllowedIPs = 0.0.0.0/0"
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `name` | Có | Tên VPN (không rỗng) |
| `config` | Có | WireGuard config content |

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-vpn","config":"[Interface]\nPrivateKey=abc\nAddress=10.0.0.2/32\n\n[Peer]\nPublicKey=def\nEndpoint=vpn.example.com:51820\nAllowedIPs=0.0.0.0/0"}' \
  http://127.0.0.1:10108/v1/vpns
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `name` rỗng, `config` rỗng |
| 400 | Config không hợp lệ |

---

## GET /v1/vpns/{id}

Xem chi tiết VPN.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/vpns/<VPN_ID>
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | VPN không tồn tại |

---

## PUT /v1/vpns/{id}

Cập nhật VPN.

```bash
curl -X PUT -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"renamed-vpn","config":"[Interface]\n..."}' \
  http://127.0.0.1:10108/v1/vpns/<VPN_ID>
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | VPN không tồn tại |
| 400 | `name` rỗng |

---

## DELETE /v1/vpns/{id}

Xóa VPN. Nếu VPN đang được profile sử dụng, VPN sẽ bị hủy gắn (profile không bị xóa).

```bash
curl -X DELETE -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/vpns/<VPN_ID>
```

### Response

```
HTTP 204 No Content
```

---

## POST /v1/vpns/import

Import VPN từ file config WireGuard.

### Request Body

```json
{
  "configs": [
    {
      "name": "vpn-1",
      "config": "[Interface]\nPrivateKey=abc\nAddress=10.0.0.2/32\n\n[Peer]\nPublicKey=def\nEndpoint=vpn.example.com:51820\nAllowedIPs=0.0.0.0/0"
    },
    {
      "name": "vpn-2",
      "config": "[Interface]\nPrivateKey=ghi\nAddress=10.0.0.3/32\n\n[Peer]\nPublicKey=jkl\nEndpoint=vpn2.example.com:51820\nAllowedIPs=0.0.0.0/0"
    }
  ]
}
```

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"configs":[{"name":"vpn-1","config":"[Interface]\nPrivateKey=abc\n..."}]}' \
  http://127.0.0.1:10108/v1/vpns/import
```

### Response

```json
{
  "imported_count": 2,
  "skipped_count": 0,
  "failed_count": 0,
  "results": [
    {
      "name": "vpn-1",
      "status": "imported",
      "vpn_id": "uuid-here"
    }
  ]
}
```

---

## GET /v1/vpns/{id}/export

Export WireGuard config của VPN.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/vpns/<VPN_ID>/export
```

### Response

```json
{
  "name": "my-vpn",
  "config": "[Interface]\nPrivateKey = abc\nAddress = 10.0.0.2/32\n\n[Peer]\nPublicKey = def\nEndpoint = vpn.example.com:51820\nAllowedIPs = 0.0.0.0/0"
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | VPN không tồn tại |
