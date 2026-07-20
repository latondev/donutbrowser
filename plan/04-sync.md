# Sync (Cloud / Self-hosted)

## Tổng quan
Sync mirror local state sang S3-compatible storage. Hai cơ chế khác nhau:

| Loại | Cơ chế | File |
|---|---|---|
| **Profile browser files** (Chromium/Firefox dir) | Content-hash manifest — per-file hash+size diff, chỉ transfer file thay đổi | `sync/manifest.rs::generate_manifest`/`compute_diff`, `sync/engine.rs::sync_profile` |
| **Config entities** (proxies, VPNs, groups, extensions, extension groups, profile *metadata*) | Một JSON blob nhỏ, sync whole | `sync/engine.rs::sync_X`/`upload_X`/`download_X` |

## Conflict resolution — một quy tắc duy nhất: `updated_at` last-write-wins

Mọi config entity có `updated_at: Option<u64>` (unix seconds; `extension_manager` dùng non-Optional u64). Đây là **source of truth** cho bên nào thắng, được bump lên `now()` CHỈ khi user edit (trong manager mutators), KHÔNG bởi sync bookkeeping.

Dùng `crate::proxy_manager::now_secs()`.

`last_sync` là **display/bookkeeping only** ("last synced at") — ghi trên mỗi upload/download, KHÔNG quyết định hướng sync.

### Quy trình reconcile (`engine.rs::remote_updated_at` + mỗi `sync_X`)
1. `stat` (HEAD) remote object. `updated_at` đọc từ S3 object metadata (`x-amz-meta-updated-at`) — **không download body** khi không thay đổi.
2. Compare local `updated_at` vs remote:
   - Local newer → upload
   - Remote newer → download
   - Equal → no transfer
   - Legacy objects (no timestamp) → resolve thành 0, edit thật luôn thắng
3. **Fallback** cho server cũ không trả metadata: GET JSON body nhỏ, đọc `updated_at` embedded. Correctness preserved; HEAD path chỉ là optimization class-B-op.

### Upload path (`engine.rs::upload_config_json`)
Ghi `updated_at` vào CẢ body JSON lẫn S3 object metadata → sau download, 2 bên đồng thuận (no ping-pong).

**Thêm synced config field mới?**
1. Thêm `updated_at` vào struct (`#[serde(default)]`)
2. Bump nó trong mọi edit path thật
3. Route reconcile qua `remote_updated_at` + `upload_config_json`

## Sync server (`donut-sync/`)
NestJS app, self-hostable, S3-compatible.

```
donut-sync/src/
├── main.ts                  # NestJS bootstrap
├── app.module.ts            # Root module
├── app.controller.ts        # Health check
├── app.service.ts
├── auth/
│   ├── auth.guard.ts        # Bearer token auth
│   └── user-context.interface.ts
└── sync/
    ├── sync.module.ts
    ├── sync.controller.ts    # presignUpload, presignDownload, stat, list...
    ├── sync.service.ts       # Logic: S3 presign, metadata passthrough
    ├── internal.controller.ts # Internal endpoints
    └── dto/sync.dto.ts       # DTOs
```

### Server metadata passthrough
- `presignUpload` ký request `metadata` vào PUT dưới dạng `x-amz-meta-*`, echo lại chính xác những gì ký
- Rust client phải gửi đúng headers đó trên PUT, không thì S3 reject
- `stat` trả `response.Metadata`
- Server cũ omit `metadata` → client fallback qua body-GET path
- DTOs: `donut-sync/src/sync/dto/sync.dto.ts`; logic: `sync.service.ts`

## Sync engine chi tiết (`sync/engine.rs`, 4346 dòng)
- `SyncEngine::create_from_settings(app_handle)` — tạo engine từ SyncSettings
- `SyncEngine::sync_profile` — upload/download profile browser files qua manifest
- `SyncEngine::upload_X`/`download_X` — sync config entity (proxy, vpn, group, extension, profile metadata)
- `SyncEngine::delete_X` — xóa remote khi local delete
- `check_for_missing_synced_profiles` / `check_for_missing_synced_entities` — pull remote về local nếu thiếu
- `remote_updated_at` — HEAD request lấy metadata
- `upload_config_json` — upload JSON + set metadata

### Concurrency
- `SYNC_CONCURRENCY = 32` — upload/download parallel limit
- `MAX_FILE_RETRIES = 3` — retry per file
- Semaphore-based

### Critical files
`CRITICAL_FILE_PATTERNS` — nếu upload/download fail → abort sync:
- Chromium: `Cookies`, `Login Data`, `Local Storage`, `Local State`, `Preferences`, `Secure Preferences`, `Web Data`, `Extension Cookies`
- Firefox: `cookies.sqlite`, `key4.db`, `logins.json`, `cert9.db`, `places.sqlite`, `formhistory.sqlite`, `permissions.sqlite`, `prefs.js`, `storage.sqlite`

### Security: path validation
`is_safe_manifest_path(path)` — reject absolute paths, `..`, root/prefix. Manifest là remote-controlled (self-hosted server compromised, MITM plaintext Regular-mode, malicious team member share E2E key) → unvalidated path = arbitrary file write/delete.

### SQLite WAL checkpoint
`checkpoint_sqlite_wal_files(profile_dir)` — trước khi generate manifest, checkpoint WAL files vào main DB để tránh data loss (WAL excluded from sync).

### Resume state
`SyncResumeState` — persisted to `.donut-sync/resume-state.json`, cho phép resume interrupted sync. Discard nếu older 12h (presigned URLs expire 1h nhưng file có thể còn).

## SyncScheduler (`sync/scheduler.rs`)
- Event-driven: queue profile sync khi profile stop, queue config entity sync khi edit
- Debounce: gộp nhiều queue trong khoảng thời gian ngắn
- `mark_profile_running` / `mark_profile_stopped` — sync queued ở launch, trigger ở stop
- `is_sync_in_progress()` — check trước khi cleanup binaries
- `sync_all_enabled_profiles` — initial sync ở startup

## SubscriptionManager (`sync/subscription.rs`)
- Listen server events (server push)
- Take work receiver, pass cho scheduler

## E2E encryption (`sync/encryption.rs`)
- Optional, per-profile password
- AES-GCM + argon2 (KDF) + salt
- `set_e2e_password`, `verify_e2e_password`, `delete_e2e_password`, `rollover_encryption_for_all_entities`
- Password-protected profile: decrypt to RAM-backed ephemeral dir, never disk

## SyncSettings (`src/types.ts`)
```typescript
interface SyncSettings {
  sync_server_url?: string;  // donut-sync server URL
  sync_token?: string;       // auth token
}
```

## SyncMode (`profile/types.rs`)
```rust
enum SyncMode {
  Disabled,    // no sync
  Regular,     // sync, no E2E
  Encrypted,   // sync + E2E encryption
}
```

## Self-hosting
- Docker-based setup (xem `docs/` self-hosting guide)
- Donut cloud = hosted version (paid plans)
- Cả 2 dùng cùng protocol, server code ở `donut-sync/`
