# Backend Rust (`src-tauri/`)

## Tổng quan
- Tauri 2 desktop app, Rust backend
- 2 binary targets:
  - `donutbrowser` (main app) — `src/main.rs` → `lib.rs::run()`
  - `donut-proxy` (local proxy worker) — `src/bin/proxy_server.rs`

## Cấu trúc modules
```
src-tauri/src/
├── main.rs                    # Entry point, gọi donutbrowser_lib::run()
├── lib.rs                     # Tauri Builder, setup, invoke_handler (230+ commands)
├── bin/
│   └── proxy_server.rs        # donut-proxy standalone binary
├── events/
│   └── mod.rs                 # EventEmitter trait + global emitter (OnceLock)
├── profile/                   # Profile CRUD + metadata
│   ├── mod.rs                 # ProfileManager (singleton), re-exports
│   ├── types.rs               # BrowserProfile struct, SyncMode, SyncStatus
│   ├── manager.rs             # CRUD: create/list/save/delete/clone/rename
│   ├── password.rs            # Per-profile password (encrypt/decrypt on disk)
│   ├── encryption.rs          # AES-GCM encryption helpers
│   └── clear_on_close.rs      # Wipe browsing data on browser exit
├── browser.rs                 # Browser trait + ProxySettings struct
├── browser_runner.rs           # Launch/kill orchestration cho profiles
├── browser_version_manager.rs # Fetch/list browser versions (Wayfern)
├── downloaded_browsers_registry.rs # Track downloaded binaries, cleanup unused
├── platform_browser.rs        # Platform-specific browser detection
├── default_browser.rs         # Set/get default OS browser
├── wayfern_manager.rs         # Wayfern (Chromium fork) management + CDP (WebSocket)
├── wayfern_terms.rs           # Wayfern terms of use acceptance
├── downloader.rs              # Browser binary downloader (HTTP, progress)
├── extraction.rs              # Archive extraction (zip, tar, bzip2, lzma, msi)
├── proxy_manager.rs           # StoredProxy CRUD + check validity (PROXY_MANAGER singleton)
├── proxy_storage.rs           # Proxy config persistence (JSON files)
├── proxy_server.rs             # donut-proxy worker lifecycle
├── proxy_runner.rs            # Start/stop donut-proxy process
├── proxy_server_tests.rs     # Tests cho proxy server
├── socks5_local.rs            # Local SOCKS5 server
├── vpn/                       # VPN (WireGuard) module
│   ├── mod.rs                 # VpnConfig, VpnImportResult, VpnStatus
│   ├── config.rs              # WireGuard config parsing
│   ├── wireguard.rs           # WireGuard tunnel (boringtun)
│   ├── tunnel.rs              # Tunnel management
│   ├── socks5_server.rs       # SOCKS5 server for VPN
│   └── storage.rs             # VPN config persistence
├── vpn_worker_runner.rs      # Start/stop VPN worker process (detached)
├── vpn_worker_storage.rs     # VPN worker state persistence
├── sync/                      # Cloud sync module (xem 03-sync.md)
│   ├── mod.rs                 # Exports + set_global_scheduler
│   ├── engine.rs              # SyncEngine — upload/download profile files + config entities
│   ├── scheduler.rs           # SyncScheduler — queue + debounce + run-on-stop
│   ├── client.rs              # SyncClient — HTTP client cho donut-sync server
│   ├── encryption.rs          # E2E encryption (AES-GCM, argon2, salt)
│   ├── manifest.rs            # Content-hash manifest (per-file hash+size diff)
│   ├── subscription.rs        # SubscriptionManager — listen server events
│   ├── types.rs               # SyncRequest, SyncResponse, etc.
│   └── scheduler.rs
├── api_server.rs              # REST API server (axum + utoipa OpenAPI)
├── api_client.rs              # HTTP client cho external APIs
├── mcp_server.rs              # MCP (Model Context Protocol) server
├── mcp_integrations.rs        # MCP agent integration (Claude Desktop, generic)
├── settings_manager.rs        # App settings persistence (SettingsManager singleton)
├── cookie_manager.rs          # Cookie read/copy/import/export (Chromium + Firefox)
├── extension_manager.rs       # Browser extension management
├── group_manager.rs           # Profile group CRUD
├── tag_manager.rs             # Tag management
├── profile_importer.rs       # Bulk profile import (Chromium detection, ZIP, batch)
├── synchronizer.rs            # Real-time profile sync (leader/follower, CDP)
├── cloud_auth.rs              # Cloud authentication (device code, token refresh)
├── commercial_license.rs      # Commercial trial/license management
├── team_lock.rs               # Team profile locking (cloud collaboration)
├── traffic_stats.rs           # Per-profile traffic stats + secure history erase
├── dns_blocklist.rs           # Hagezi DNS blocklists + custom lists/allowlist
├── fingerprint_consistency.rs # Launch-time proxy exit vs fingerprint timezone/lang check
├── geolocation.rs             # Geolocation resolution
├── geoip_downloader.rs        # MaxMind GeoIP DB download
├── ip_utils.rs                # fetch_public_ip, IP utilities
├── human_typing.rs            # Human typing simulation (automation)
├── auto_updater.rs            # Browser auto-update logic
├── app_auto_updater.rs        # App auto-update (download, prepare, restart)
├── version_updater.rs         # Version cache + background update
├── app_dirs.rs                # App data/log directory resolution
├── ephemeral_dirs.rs          # RAM-backed ephemeral profile dirs (tmpfs/ramdisk)
└── territory_info.xml         # Territory metadata
```

## Entry point (`lib.rs::run()`)
Thứ tự startup:
1. Parse args, detect startup URL (deep link)
2. Tauri Builder + plugins (log, single-instance, deep-link, fs, opener, shell, dialog, macos-permissions, clipboard, window-state)
3. `setup` closure:
   - Recover ephemeral dirs
   - Extract extension icons
   - Build main window (880×500, min 640×400)
   - Setup system tray (best-effort, Linux có thể fail)
   - Intercept close → emit `close-confirm-requested`
   - macOS: transparent titlebar + disable native fullscreen
   - Deep link handler
   - Set global event emitter
   - Spawn background tasks (xem 01-architecture.md)
   - Clear stale PIDs, kill orphan proxy/VPN workers
   - Bump non-running profiles to latest installed version
   - Start sync scheduler + subscription
   - Start cloud auth refresh loop
4. `invoke_handler!` — đăng ký 230+ commands

## Tauri commands (categorized)
| Nhóm | Commands (ví dụ) |
|---|---|
| Profile | `create_browser_profile_new`, `list_browser_profiles`, `delete_profile`, `clone_profile`, `rename_profile`, `update_wayfern_config`, `update_profile_*` |
| Browser | `launch_browser_profile`, `kill_browser_profile`, `check_browser_status`, `open_url_with_profile`, `download_browser`, `cancel_download`, `fetch_browser_versions_*` |
| Proxy | `create_stored_proxy`, `get_stored_proxies`, `update_stored_proxy`, `delete_stored_proxy`, `check_proxy_validity`, `export_proxies`, `import_proxies_*`, `parse_txt_proxies` |
| VPN | `import_vpn_config`, `list_vpn_configs`, `connect_vpn`, `disconnect_vpn`, `get_vpn_status`, `check_vpn_validity` |
| Sync | `set_profile_sync_mode`, `cancel_profile_sync`, `request_profile_sync`, `set_*_sync_enabled`, `set_e2e_password`, `verify_e2e_password`, `enable_sync_for_all_entities` |
| Extension | `list_extensions`, `add_extension`, `delete_extension`, `create_extension_group`, `assign_extension_group_to_profile` |
| Group | `create_profile_group`, `update_profile_group`, `delete_profile_group`, `assign_profiles_to_group` |
| Cookie | `read_profile_cookies`, `copy_profile_cookies`, `import_cookies_from_file`, `export_profile_cookies` |
| Settings | `get_app_settings`, `save_app_settings`, `get_sync_settings`, `save_sync_settings`, `read_log_files` |
| Cloud auth | `cloud_exchange_device_code`, `cloud_get_user`, `cloud_logout`, `cloud_get_proxy_usage`, `cloud_get_countries/regions/cities/isps`, `create_cloud_location_proxy` |
| API/MCP | `start_api_server`, `stop_api_server`, `start_mcp_server`, `stop_mcp_server`, `get_mcp_config`, `list_mcp_agents`, `add_mcp_to_agent` |
| DNS | `get_dns_blocklist_cache_status`, `refresh_dns_blocklists`, `get/set_custom_dns_config`, `import/export_custom_dns_rules` |
| Update | `check_for_app_updates`, `download_and_prepare_app_update`, `restart_application`, `check_for_browser_updates` |
| Password | `set_profile_password`, `change_profile_password`, `verify_profile_password`, `unlock_profile`, `lock_profile`, `is_profile_locked` |
| Team | `get_team_locks`, `get_team_lock_status` |
| Tray/Window | `confirm_quit`, `hide_to_tray`, `update_tray_menu` |
| Wayfern | `check_wayfern_terms_accepted`, `accept_wayfern_terms`, `generate_sample_fingerprint` |

## Error handling (bắt buộc)
- Tauri command trả `Result<T, String>`
- Error PHẢI là JSON string: `{"code": "FOO_BAR", "params": {...}}`
- Frontend resolve qua `translateBackendError(t, err)` → `t("backendErrors.fooBar")`
- Helper: `wrap_backend_error(e, context)` — pass-through JSON errors, wrap others
- Thêm code mới = 4 edits song song (xem AGENTS.md "Backend error codes")

## Platform-specific code
- `#[cfg(target_os = "macos")]` — transparent titlebar, NSWindow, objc2
- `#[cfg(target_os = "windows")]` — winreg, windows crate (Win32 APIs)
- `#[cfg(unix)]` — nix (signal, process)
- Linux tray best-effort (libayatana-appindicator runtime)

## Testing
- `cargo test --lib` — unit tests
- `cargo test --test donut_proxy_integration` — proxy integration
- `cargo test --test vpn_integration` — VPN integration
- `cargo test --test sync_e2e` — sync end-to-end
- `cargo test test_no_unused_tauri_commands` — check unused commands
- Clippy: `cargo clippy --all-targets --all-features -- -D warnings -D clippy::all`
- Fmt: `cargo fmt --all`
