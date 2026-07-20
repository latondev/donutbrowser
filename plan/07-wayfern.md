# Wayfern Browser Management

## Tổng quan
**Wayfern** là Chromium fork riêng (https://wayfern.com) — engine anti-detect của Donut Browser.
- Privacy-focused, spoof fingerprint vượt Cloudflare, reCaptcha v3, anti-bot services
- Tích hợp qua **CDP (Chrome DevTools Protocol)** qua WebSocket
- Terms of use acceptance bắt buộc trước khi dùng

## Module chính: `wayfern_manager.rs`
- `WayfernManager::instance()` — singleton
- `generate_fingerprint_config(app_handle, profile, config)` — gen fingerprint config, return (fingerprint_json, geolocation_applied)
- `launch_wayfern(...)` — launch browser, return `WayfernLaunchResult { id, processId, profilePath, url, cdp_port }`
- CDP integration qua `tokio-tungstenite` (WebSocket + native-tls)

## WayfernConfig (`src/types.ts`)
```typescript
interface WayfernConfig {
  proxy?: string;
  screen_max_width?: number;
  screen_max_height?: number;
  screen_min_width?: number;
  screen_min_height?: number;
  geoip?: string | boolean;
  block_images?: boolean;
  block_webrtc?: boolean;
  block_webgl?: boolean;
  executable_path?: string;
  fingerprint?: string;                        // JSON string of full fingerprint config
  randomize_fingerprint_on_launch?: boolean;    // Gen new fingerprint mỗi launch
  os?: WayfernOS;                              // "windows"|"macos"|"linux"|"android"|"ios"
  geo_proxy_signature?: string;                 // Internal: routing location
}

type WayfernOS = "windows" | "macos" | "linux" | "android" | "ios";
```

## WayfernFingerprintConfig
Match C++ `FingerprintData` structure. Các nhóm field:
- **User agent/platform**: userAgent, platform, platformVersion, brand, brandVersion
- **Hardware**: hardwareConcurrency, maxTouchPoints, deviceMemory
- **Screen**: width, height, availWidth, availHeight, colorDepth, pixelDepth, devicePixelRatio
- **Window**: outerWidth/Height, innerWidth/Height, screenX/Y
- **Language**: language, languages[]
- **Browser features**: doNotTrack, cookieEnabled, webdriver, pdfViewerEnabled
- **WebGL**: vendor, renderer, version, shadingLanguageVersion, parameters (JSON), shader precision formats
- **Timezone/geo**: timezone, timezoneOffset, latitude, longitude, accuracy
- **Media queries**: prefersReducedMotion, prefersDarkMode, prefersContrast, prefersReducedData
- **Color/HDR**: colorGamutSrgb, P3, Rec2020, hdrSupport
- **Audio**: sampleRate, maxChannelCount
- **Storage**: localStorage, sessionStorage, indexedDb
- **Canvas**: canvasNoiseSeed
- **Fonts/plugins/mime**: JSON strings
- **Battery**: charging, chargingTime, dischargingTime, level
- **Voices**: JSON string
- **Vendor**: vendor, vendorSub, productSub
- **Network**: connectionEffectiveType, downlink, rtt
- **Performance**: performanceMemory

## WayfernLaunchResult
```typescript
interface WayfernLaunchResult {
  id: string;
  processId?: number;
  profilePath?: string;
  url?: string;
  cdp_port?: number;
}
```

## Luồng launch (từ `browser_runner.rs`)
1. `launch_browser_profile(profile_id)` — entry
2. Check Wayfern terms accepted (`wayfern_terms.rs`)
3. Check browser binary downloaded (`downloaded_browsers_registry.rs`)
4. Validate fingerprint consistency (`fingerprint_consistency.rs`) — proxy exit vs fingerprint timezone/lang
5. Start proxy worker nếu có (`proxy_runner.rs`)
6. Start VPN worker nếu có (`vpn_worker_runner.rs`)
7. Gen fingerprint config (`WayfernManager::generate_fingerprint_config`)
   - Nếu `randomize_fingerprint_on_launch` = gen mới mỗi lần
   - Apply geolocation từ proxy exit IP (nếu `geoip` enable)
8. Launch Wayfern process, get CDP port
9. Update profile `process_id`, `last_launch`
10. Emit `profile-running-changed` event
11. Queue sync (scheduler `mark_profile_running`)

## Luồng kill
1. `kill_browser_profile(profile_id)`
2. Kill Wayfern process (via PID hoặc CDP)
3. Stop proxy worker (nếu không có profile khác dùng)
4. Stop VPN worker (nếu không có profile khác dùng)
5. Re-encrypt password-protected profile (`profile::password::complete_after_quit_and_wait`)
6. Clear-on-close (`profile::clear_on_close::clear_profile_browsing_data`)
7. Release team lock (`team_lock::release_team_lock_if_needed`)
8. Trigger sync (`scheduler.mark_profile_stopped`)
9. Emit `profile-running-changed` event

## Wayfern terms (`wayfern_terms.rs`)
- `WayfernTermsManager::instance()` — singleton
- `is_terms_accepted()` — check acceptance
- `accept_terms()` — mark accepted
- `is_wayfern_downloaded()` — check binary downloaded
- UI: `WayfernTermsDialog` component

## Browser version management (`browser_version_manager.rs`)
- `get_supported_browsers()` — list supported browsers
- `is_browser_supported_on_platform(browser)` — platform check
- `fetch_browser_versions_with_count(...)` — fetch versions (cached first option)
- `get_browser_release_types()` — stable/nightly

## Downloader (`downloader.rs`)
- `download_browser(...)` — download browser binary với progress
- `cancel_download(...)` — cancel ongoing download
- Progress emit qua events

## Extraction (`extraction.rs`)
- zip (flate2), tar, bzip2, lzma-rs, msi-extract
- Platform-specific: dmg (macOS), msi (Windows)

## GeoIP (`geoip_downloader.rs`, `geolocation.rs`)
- MaxMind GeoIP database (maxminddb crate)
- `GeoIPDownloader::instance()` — singleton
- `check_missing_geoip_database()` — check + auto-download at startup
- `download_geoip_database(app_handle)` — download DB
- Dùng cho fingerprint geolocation (match proxy exit IP)

## Related UI components
- `wayfern-config-dialog.tsx` — Wayfern config dialog
- `wayfern-config-form.tsx` — form fields
- `wayfern-terms-dialog.tsx` — terms acceptance
- `create-profile-dialog.tsx` — profile creation (includes Wayfern config)
- `shared-fingerprint-config-form.tsx` — shared form fields
