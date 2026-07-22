# OpenAPI Spec

Đầy đủ schema được tự tạo từ code qua utoipa.

---

## GET /openapi.json

Trả về OpenAPI 3.0 spec đầy đủ.

```bash
curl http://127.0.0.1:10108/openapi.json
```

> Không cần token cho endpoint này.

### Sử dụng

Bạn có thể import file này vào:

- [Swagger UI](https://editor.swagger.io/)
- [Postman](https://www.postman.com/)
- [Insomnia](https://insomnia.rest/)
- [Bruno](https://www.usebruno.com/)

### Tạo client SDK

Từ OpenAPI spec, bạn có thể sinh code client tự động:

```bash
# Python
openapi-python-client generate --url http://127.0.0.1:10108/openapi.json

# TypeScript
npx openapi-typescript http://127.0.0.1:10108/openapi.json -o ./api-types.ts

# Go
oapi-codegen -package api -generate types,client http://127.0.0.1:10108/openapi.json > api.gen.go
```

### Schemas

Các schema chính được định nghĩa trong OpenAPI:

| Schema | Mô tả |
|--------|-------|
| `ApiProfile` | Profile response object |
| `ApiProxyResponse` | Proxy response object |
| `ApiVpnResponse` | VPN response object |
| `CreateProfileRequest` | Body tạo profile |
| `CreateProxyRequest` | Body tạo proxy |
| `CreateVpnRequest` | Body tạo VPN |
| `RunProfileRequest` | Body launch browser |
| `BatchRunRequest` | Body batch launch |
| `BatchStopRequest` | Body batch stop |
| `OpenUrlRequest` | Body mở URL |
| `ImportProfilesRequest` | Body import profile |
| `ImportProxiesRequest` | Body import proxy |
| `ImportCookiesRequest` | Body import cookies |
| `DownloadBrowserRequest` | Body download browser |
