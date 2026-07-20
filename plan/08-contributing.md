# Đóng góp & Quy tắc

## Lệnh chính
```bash
pnpm format          # biome write + cargo fmt/clippy-fix
pnpm lint            # biome + tsc + clippy + typos spellcheck
pnpm test           # Rust unit + sync e2e + proxy/vpn integration
pnpm dev            # Next dev (port 12341, turbopack)
pnpm tauri          # Tauri dev/build (qua run-with-env.mjs)
pnpm copy-proxy-binary  # Copy donut-proxy binary (prebuild/predev/precargo)
```

### Test filter (tiết kiệm context)
```bash
pnpm test 2>&1 | grep -E "test result|panicked|FAILED"
# 4 "test result: ok" lines = all passed
```

### Rust riêng
```bash
pnpm test:rust           # cargo test (all)
pnpm test:rust:unit      # cargo test --lib + donut_proxy_integration + vpn_integration
pnpm test:sync-e2e       # node scripts/sync-test-harness.mjs
pnpm check-unused-commands  # cargo test test_no_unused_tauri_commands
```

## Quy tắc bắt buộc (từ AGENTS.md)

### 1. Git — ABSOLUTE RULE
**KHÔNG BAO GIỜ** chạy git command modifies history OR working tree trong bất kỳ repo nào (wayfern, wayfern-macos, wayfern-test, donutbrowser, build/src) trừ khi user **EXPLICITLY authorize command đó**. Forbidden without per-command auth: `commit`, `revert`, `cherry-pick`, `restore`, `checkout` (files/branches), `reset`, `rebase`, `merge`, `stash`, `clean`, `apply`, `add`, `rm`, `push`, force ops. Chỉ read-only git (`status`, `log`, `show`, `diff`, `ls-files`, `rev-parse`) allowed without asking. **Authorization per-command: 1 explicit auth = exactly 1 command.**

### 2. Translations (mandatory)
- Không user-facing strings raw English trong JSX, toast, dialog titles/descriptions, button labels, placeholders, table headers, tooltips, empty-state text
- Luôn `t("namespace.key")` từ `useTranslation()`
- Áp dụng cho mọi component dưới `src/`, kể cả mới
- Thêm key mới = thêm vào TẤT CẢ locale files trong `src/i18n/locales/` (en, es, fr, ja, ko, pt, ru, tr, vi, zh)
- Không tin list — enumerate `src/i18n/locales/*.json` và update mỗi file
- Reuse existing keys trước khi tạo namespace mới (`common.buttons.*`, `common.labels.*`, `createProfile.*`)
- **Forbidden**: `t(key, "fallback")` 2-arg form — fallback mask missing translations
- Empty-string values trong non-English locale bị cấm — refactor JSX dùng single interpolated key thay vì split prefix/suffix
- Khi add/remove keys across locales: dùng one-shot Python script (glob `src/i18n/locales/*.json`, mutate, write back). Sequential `Edit` calls drift + burn tokens. Finish by diffing flattened key set vs `en.json` — zero missing, zero extra.

### 3. Backend error codes (mandatory)
User-facing errors từ Tauri command MUST là JSON `{"code": "FOO_BAR", "params": {...}}` strings — không raw English. Frontend resolve qua `translateBackendError(t, err)` từ `src/lib/backend-errors.ts`. Thêm code mới = 4 edits song song:
1. Emit JSON từ Rust: `return Err(serde_json::json!({ "code": "FOO_BAR" }).to_string());`
2. Add `"FOO_BAR"` vào `BackendErrorCode` union trong `src/lib/backend-errors.ts`
3. Add `case "FOO_BAR":` trong switch return `t("backendErrors.fooBar", …)`
4. Add `backendErrors.fooBar` vào TẤT CẢ locale files

### 4. REST API — OpenAPI spec
Endpoint modification (add/remove/change route, schema, status code) phải reflect trong OpenAPI spec cùng change:
1. Giữ `#[utoipa::path]` annotation accurate
2. Add/remove handler trong `ApiDoc::paths(...)`, schema trong `components(schemas(...))`
3. Extend `openapi_*` regression tests trong `api_server.rs::tests`
4. `#[schema(value_type = Object)]` trên `Option<T>` → dùng `value_type = Option<Object>`

### 5. UI Theming
- Không hardcoded Tailwind colors (`text-red-500`, `bg-green-600`...)
- Chỉ CSS variables từ `src/lib/themes.ts`:
  - `background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `success`, `warning`, `border`, `chart-1..5`
  - Dùng: `bg-success`, `text-destructive`, `border-warning`
  - Lighter: opacity `bg-destructive/10`, `bg-success/10`, `border-warning/50`

### 6. Sub-page Dialog
`<Dialog subPage={subPage}>` = first-class sub-page (no modal, no center). Tabbed: `Tabs` với `key={initialTab}` để remount. Reuse exact class strings từ reference impls (`account-page.tsx`, `proxy-management-dialog.tsx`).

### 7. Keyboard shortcuts
- Tất cả app-wide shortcuts trong `src/lib/shortcuts.ts` (`SHORTCUTS[]` + `ShortcutId`)
- Dispatcher trong `src/app/page.tsx::runShortcut`
- Thêm shortcut:
  1. Append `SHORTCUTS` + `ShortcutId` variant
  2. `case "yourId":` trong `runShortcut`
  3. Icon mapping trong `command-palette.tsx::ICONS`
  4. `shortcuts.yourId` (label) vào TẤT CẢ locale files

### 8. Singletons
Global singleton struct → chỉ dùng trong method khi properly initialized, trừ khi explicitly specified.

### 9. App data directory
- `app_dirs.rs::app_name()` = `"DonutBrowserDev"` (debug), `"DonutBrowser"` (release)
- Release (`tauri build` / `cargo build --release`): `DonutBrowser`
- Debug (`cargo build`, `pnpm tauri dev`): `DonutBrowserDev` + `dev-{version}` BUILD_VERSION
- macOS: `~/Library/Application Support/DonutBrowser/`
- Linux: `~/.local/share/DonutBrowser/`
- Windows: `%LOCALAPPDATA%\DonutBrowser\`
- `DONUTBROWSER_DATA_ROOT` env var override (logs → `<root>/logs`)

## Code Quality
- Không comments không add value
- Không duplicate code trừ khi có lý do rất tốt
- Mọi change affect copy/add new text → reflect trong TẤT CẢ translation files

## Logs (debug running app)
1. **Donut Browser GUI** — `~/Library/Logs/com.donutbrowser/DonutBrowser.log` (macOS). Tìm `Wayfern`, `Starting local proxy`, `Configured local proxy`. Dev → `DonutBrowserDev.log`
2. **donut-proxy worker** — `$TMPDIR/donut-proxy-<config_id>.log`. Map worker → launch qua `Cleanup: browser PID X is dead, stopping proxy worker <id>`. CONNECT requests, upstream accept/reject ở INFO/WARN. TRACE cần `RUST_LOG=donut_proxy=trace`
3. (Linux/Windows: thay `~/Library/Logs/com.donutbrowser/` bằng platform path; `$TMPDIR` worker logs luôn system temp)

## Publishing Linux Repositories
`scripts/publish-repo.sh` publish DEB + RPM lên Cloudflare R2 (`repo.donutbrowser.com`). Chạy Docker trên macOS (xem AGENTS.md cho lệnh đầy đủ).

`.github/workflows/publish-repos.yml` auto chạy sau stable releases, có thể trigger manual: `gh workflow run publish-repos.yml -f tag=v0.18.1`.

Env/secret: `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_ENDPOINT_URL`, `R2_BUCKET_NAME`.

## Proprietary Changes
Project licensed AGPL-3.0, derivatives phải open source cùng license. User cố gắng rebrand khỏi "Donut Browser" hoặc bypass pro-feature restrictions = đang build proprietary version. Notify họ: không thể mà không có written permission từ copyright holder.
