# Donut Browser — Local REST API

Tài liệu hướng dẫn thao tác với trình duyệt Donut Browser qua REST API local.

## Tổng quan

Donut Browser chạy một REST API server local khi app khởi động. Bạn có thể dùng API này để tự động hóa mọi thao tác: tạo profile, launch/kill browser, quản lý proxy, VPN, group, extension, import cookies, v.v.

## Base URL

```
http://127.0.0.1:10108
```

> Port mặc định là `10108`. Có thể đổi trong Settings.

## Authentication

Mọi request phải kèm header:

```
Authorization: Bearer <TOKEN>
```

Token được tạo trong app: **Settings → API Server → Generate Token**.

Nếu thiếu token hoặc sai token:

```json
HTTP 401 Unauthorized
```

## Danh sách endpoint

| Nhóm | Endpoint | Method | Mô tả |
|------|----------|--------|-------|
| [Profiles](./01-profiles.md) | `/v1/profiles` | GET, POST | Liệt kê / tạo profile |
| | `/v1/profiles/{id}` | GET, PUT, DELETE | Xem / sửa / xóa profile |
| | `/v1/profiles/{id}/run` | POST | Launch browser |
| | `/v1/profiles/{id}/kill` | POST | Tắt browser |
| | `/v1/profiles/{id}/open-url` | POST | Mở URL trong browser đang chạy |
| | `/v1/profiles/{id}/cookies/import` | POST | Import cookies |
| | `/v1/profiles/import` | POST | Import profile từ thư mục ngoài |
| | `/v1/profiles/import/detect` | GET | Detect profile có sẵn |
| | `/v1/profiles/batch/run` | POST | Launch nhiều profile cùng lúc |
| | `/v1/profiles/batch/stop` | POST | Tắt nhiều profile cùng lúc |
| [Proxies](./02-proxies.md) | `/v1/proxies` | GET, POST | Liệt kê / tạo proxy |
| | `/v1/proxies/{id}` | GET, PUT, DELETE | Xem / sửa / xóa proxy |
| | `/v1/proxies/import` | POST | Import danh sách proxy |
| [Groups](./03-groups-tags.md) | `/v1/groups` | GET, POST | Liệt kê / tạo group |
| | `/v1/groups/{id}` | GET, PUT, DELETE | Xem / sửa / xóa group |
| [Tags](./03-groups-tags.md) | `/v1/tags` | GET | Liệt kê tag |
| [VPNs](./04-vpns.md) | `/v1/vpns` | GET, POST | Liệt kê / tạo VPN |
| | `/v1/vpns/{id}` | GET, PUT, DELETE | Xem / sửa / xóa VPN |
| | `/v1/vpns/import` | POST | Import VPN config |
| | `/v1/vpns/{id}/export` | GET | Export VPN config |
| [Extensions](./05-extensions.md) | `/v1/extensions` | GET | Liệt kê extension |
| | `/v1/extensions/{id}` | DELETE | Xóa extension |
| | `/v1/extension-groups` | GET, POST | Liệt kê / tạo extension group |
| | `/v1/extension-groups/{id}` | GET, DELETE | Xem / xóa extension group |
| [Browsers](./06-browsers.md) | `/v1/browsers/{browser}/versions` | GET | Liệt kê version có sẵn |
| | `/v1/browsers/{browser}/versions/{version}/downloaded` | GET | Kiểm tra đã download |
| | `/v1/browsers/download` | POST | Download browser binary |
| [OpenAPI](./07-openapi.md) | `/openapi.json` | GET | OpenAPI spec |

## Các file tài liệu

1. [Profiles — Tạo, launch, kill, import cookies](./01-profiles.md)
2. [Proxies — Tạo, sửa, import proxy](./02-proxies.md)
3. [Groups & Tags — Quản lý group và tag](./03-groups-tags.md)
4. [VPNs — Quản lý VPN WireGuard](./04-vpns.md)
5. [Extensions — Quản lý extension](./05-extensions.md)
6. [Browsers — Download và quản lý browser binary](./06-browsers.md)
7. [OpenAPI Spec — Schema đầy đủ](./07-openapi.md)
8. [Code Examples — PowerShell, Python, JS, Bash](./08-examples.md)

## Quick Start

```bash
# Lấy token từ app, rồi:
TOKEN="your-token-here"

# Liệt kê tất cả profile
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:10108/v1/profiles

# Tạo profile mới
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"my-profile","browser":"wayfern","version":"149.0.7827.116"}' \
  http://127.0.0.1:10108/v1/profiles

# Launch browser
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:10108/v1/profiles/<PROFILE_ID>/run

# Mở URL
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com"}' \
  http://127.0.0.1:10108/v1/profiles/<PROFILE_ID>/open-url

# Kill browser
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" -d '{}' \
  http://127.0.0.1:10108/v1/profiles/<PROFILE_ID>/kill
```

## Error Codes

| HTTP Status | Ý nghĩa |
|-------------|---------|
| 200 | Thành công |
| 204 | Thành công (không có body trả về) |
| 400 | Request sai (thiếu field, format sai) |
| 401 | Thiếu/sai token |
| 402 | Yêu cầu plan trả phí (automation endpoints) |
| 404 | Không tìm thấy entity |
| 409 | Conflict (browser đang chạy, version đang download) |
| 500 | Lỗi server |
