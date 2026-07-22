# Browsers

Download và quản lý browser binary (Wayfern).

---

## GET /v1/browsers/{browser}/versions

Liệt kê các version có sẵn để download.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/browsers/wayfern/versions
```

### Response

```json
{
  "value": [
    "149.0.7827.116"
  ],
  "Count": 1
}
```

---

## GET /v1/browsers/{browser}/versions/{version}/downloaded

Kiểm tra một version đã được download chưa.

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:10108/v1/browsers/wayfern/versions/149.0.7827.116/downloaded
```

### Response

```json
{
  "downloaded": true,
  "version": "149.0.7827.116"
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 404 | Browser type không supported |

---

## POST /v1/browsers/download

Download browser binary.

### Request Body

```json
{
  "browser": "wayfern",
  "version": "149.0.7827.116",
  "release_type": "stable"
}
```

| Field | Bắt buộc | Mô tả |
|-------|----------|-------|
| `browser` | Có | `wayfern` |
| `version` | Không | Version muốn download (mặc định: mới nhất) |
| `release_type` | Không | `stable` (mặc định) hoặc `nightly` |

```bash
curl -X POST -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"browser":"wayfern","version":"149.0.7827.116","release_type":"stable"}' \
  http://127.0.0.1:10108/v1/browsers/download
```

### Response

```json
{
  "status": "downloading",
  "browser": "wayfern",
  "version": "149.0.7827.116"
}
```

### Errors

| Status | Khi nào |
|--------|---------|
| 400 | `browser` không supported |
| 409 | Version đang được download (`BROWSER_VERSION_DOWNLOADING`) |
