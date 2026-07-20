# Thư mục file plan

Tài liệu này được tạo để bất kỳ ai đọc vào hiểu toàn bộ project Donut Browser.

## Thứ tự đọc đề nghị

| # | File | Nội dung |
|---|---|---|
| 00 | `00-overview.md` | Tổng quan project, vấn đề giải quyết, stack, tính năng |
| 01 | `01-architecture.md` | Kiến trúc tổng thể, sơ đồ, luồng giao tiếp, background tasks |
| 02 | `02-frontend.md` | Cấu trúc frontend Next.js, components, hooks, patterns |
| 03 | `03-backend-rust.md` | Backend Rust, modules, Tauri commands, entry point |
| 04 | `04-sync.md` | Sync cloud/self-hosted, conflict resolution, engine, scheduler |
| 05 | `05-proxy-vpn.md` | Proxy (donut-proxy worker), VPN (WireGuard), validation |
| 06 | `06-api-mcp.md` | REST API (axum/utoipa), MCP server, Claude Desktop integration |
| 07 | `07-wayfern.md` | Wayfern browser management, fingerprint, CDP, launch/kill flow |
| 08 | `08-contributing.md` | Đóng góp, lệnh, quy tắc bắt buộc (translations, errors, theming) |
| 09 | `09-data-types.md` | Key data types & structures (TS + Rust mirror) |

## Tóm tắt nhanh

**Donut Browser** = anti-detect browser mã nguồn mở (AGPL-3.0) phiên bản `0.28.2`.

- **Desktop**: Tauri 2 (Rust backend + Next.js 16/React 19 webview)
- **Engine**: Wayfern (Chromium fork riêng, anti-fingerprint)
- **4 thành phần codebase**: `src/` (frontend), `src-tauri/` (Rust backend), `donut-sync/` (NestJS sync server), `scripts/`
- **~230+ Tauri commands** trong `lib.rs::invoke_handler!`
- **Sync**: content-hash manifest (profile files) + whole JSON blob (config entities), `updated_at` last-write-wins, E2E encryption tùy chọn
- **Proxy**: donut-proxy worker (per-profile), HTTP/SOCKS5/Shadowsocks
- **VPN**: WireGuard (boringtun, detached worker)
- **API**: REST (axum + utoipa OpenAPI) + MCP (cho Claude/AI agents)
- **i18n**: 10 locale (en, es, fr, ja, ko, pt, ru, tr, vi, zh)
- **Platform**: macOS, Windows, Linux (x86_64 + ARM64), Nix

## Bắt đầu phát triển

```bash
pnpm install                    # cài deps
pnpm copy-proxy-binary          # copy donut-proxy binary
pnpm tauri dev                  # chạy dev (Tauri + Next.js)
# hoặc riêng:
pnpm dev                        # chỉ Next.js (port 12341)
cd src-tauri && cargo run       # chỉ Rust
```

## Trước khi commit
```bash
pnpm format && pnpm lint && pnpm test
```

## Lệnh test tiết kiệm context
```bash
pnpm test 2>&1 | grep -E "test result|panicked|FAILED"
# 4 "test result: ok" = pass
```

## Docs đầy đủ
- `AGENTS.md` (root) — quy tắc chi tiết nhất (symlink `CLAUDE.md`)
- `CONTRIBUTING.md` — contributing guide
- `docs/` — self-hosting guide
- `README.md` — install links, features
