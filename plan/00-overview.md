# Donut Browser — Tổng quan Project

## Một câu
Donut Browser là một **anti-detect browser** mã nguồn mở (AGPL-3.0), cho phép tạo số lượng profile không giới hạn — mỗi profile cô lập hoàn toàn với fingerprint, cookie, extension, proxy/VPN riêng — vận hành trên engine Chromium fork tên **Wayfern**.

## Vấn đề giải quyết
- Che giấu/dOptionPanelỗi fingerprint trình duyệt để vượt qua Cloudflare, reCaptcha v3, các dịch vụ anti-bot.
- Quản lý nhiều profile (mỗi profile = một "danh tính" trình duyệt độc lập).
- Tự động hóa qua REST API và MCP (Claude/AI agents).
- Đồng bộ profile/proxy/group giữa nhiều thiết bị, có mã hóa end-to-end.

## Stack công nghệ chính
| Lớp | Công nghệ |
|---|---|
| Desktop shell | **Tauri 2** (Rust ↔ webview) |
| Backend | **Rust** (axum, tokio, hyper, rusqlite) |
| Browser engine | **Wayfern** (Chromium fork, riêng) |
| Frontend | **Next.js 16** + React 19 + TypeScript |
| UI | Tailwind 4 + shadcn/ui (Radix primitives) |
| Sync server | **NestJS** (donut-sync, self-hostable) |
| Proxy local | `donut-proxy` binary (Rust, shadowsocks/socks5/http) |
| VPN | WireGuard (boringtun) + smoltcp |
| Storage | JSON files + SQLite (rusqlite) |
| i18n | i18next (10 ngôn ngữ) |

## Số phiên bản hiện tại
- App: `0.28.2`
- License: AGPL-3.0
- Package manager: `pnpm@11.10.0`

## Tính năng chính (từ README)
1. **Unlimited profiles** — cô lập fingerprint/cookies/extensions/data
2. **Anti-detect Wayfern** — Chromium fork spoof fingerprint vượt anti-bot
3. **DNS AdBlocker** — per-profile DNS blocking (Hagezi blocklists)
4. **Proxy support** — HTTP/HTTPS/SOCKS4/SOCKS5/Shadowsocks, per profile
5. **VPN support** — WireGuard configs per profile
6. **Local API & MCP** — REST + Model Context Protocol cho Claude/automation
7. **Profile groups** — tổ chức + bulk settings
8. **Import profiles** — từ Chrome/Edge/Brave/Chromium browsers
9. **Cookie & extension management** — import/export, quản lý per profile
10. **Default browser** — đặt Donut làm browser mặc định
11. **Cloud sync** — sync profile/proxy/group (self-hostable)
12. **E2E encryption** — mã hóa end-to-end tùy chọn với password
13. **Zero telemetry** — không tracking

## Nền tảng hỗ trợ
- macOS (Apple Silicon + Intel), Homebrew cask
- Windows (installer + portable)
- Linux (deb, rpm, AppImage) — x86_64 + ARM64
- Nix (flake.nix)

## Bốn thành phần codebase lớn
1. **`src/`** — Frontend Next.js (UI, hooks, i18n, lib utilities)
2. **`src-tauri/`** — Backend Rust (Tauri commands, managers, sync, proxy, vpn, browser)
3. **`donut-sync/`** — NestJS sync server (self-hostable, S3-compatible)
4. **`scripts/`** — Build/publish tooling

## Nguyên tắc thiết kế (từ AGENTS.md)
- **YAGNI extremist** — không code thứ không cần
- Deletion trước addition, boring over clever
- Mọi user-facing string phải đi qua `t("namespace.key")` và có ở TẤT CẢ 10 locale
- Backend errors phải là JSON `{"code": "FOO_BAR"}` strings, resolve qua `backend-errors.ts`
- UI colors chỉ dùng CSS variables (themes.ts), không hardcoded Tailwind colors
- Singletons phải được init đúng cách trước khi dùng
