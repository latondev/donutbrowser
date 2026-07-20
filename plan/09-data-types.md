# Key Data Types & Structures

## TypeScript types (`src/types.ts`)

### Core: BrowserProfile
```typescript
interface BrowserProfile {
  id: string;                    // UUID
  name: string;
  browser: string;               // "wayfern"
  version: string;               // Browser version
  proxy_id?: string;             // Ref to StoredProxy
  vpn_id?: string;               // Ref to VpnConfig
  launch_hook?: string;          // Custom launch hook
  process_id?: number;           // Running browser PID (None = not running)
  last_launch?: number;          // Epoch seconds
  release_type: string;          // "stable"
  wayfern_config?: WayfernConfig;
  group_id?: string;             // Profile group
  tags?: string[];
  note?: string;
  window_color?: string;         // "#RRGGBB" — auto-derived from id when unset
  sync_mode?: SyncMode;          // "Disabled"|"Regular"|"Encrypted"
  encryption_salt?: string;
  last_sync?: number;            // Epoch seconds — display only, NOT sync direction
  host_os?: string;              // "macos"|"windows"|"linux"
  ephemeral?: boolean;
  clear_on_close?: boolean;
  extension_group_id?: string;
  proxy_bypass_rules?: string[];
  created_by_id?: string;
  created_by_email?: string;
  created_at?: number;           // Epoch seconds UTC
  dns_blocklist?: string;
  password_protected?: boolean;
}
```

### Rust mirror (`src-tauri/src/profile/types.rs`)
```rust
pub struct BrowserProfile {
  pub id: uuid::Uuid,
  pub name: String,
  pub browser: String,
  pub version: String,
  pub proxy_id: Option<String>,
  pub vpn_id: Option<String>,
  pub launch_hook: Option<String>,
  pub process_id: Option<u32>,
  pub last_launch: Option<u64>,
  pub release_type: String,             // default "stable"
  pub wayfern_config: Option<WayfernConfig>,
  pub group_id: Option<String>,
  pub tags: Vec<String>,
  pub note: Option<String>,
  pub window_color: Option<String>,
  pub sync_mode: SyncMode,              // default Disabled
  pub encryption_salt: Option<String>,
  pub last_sync: Option<u64>,
  pub host_os: Option<String>,
  pub ephemeral: bool,
  pub extension_group_id: Option<String>,
  pub proxy_bypass_rules: Vec<String>,
  pub created_by_id: Option<String>,
  pub created_by_email: Option<String>,
  pub dns_blocklist: Option<String>,
  pub password_protected: bool,
  pub clear_on_close: bool,
  pub created_at: Option<u64>,
  pub updated_at: Option<u64>,          // last meaningful metadata edit (LWW source of truth)
}

pub enum SyncMode { Disabled, Regular, Encrypted }
pub enum SyncStatus { Disabled, Syncing, Synced, Error }
```

### SyncMode / SyncStatus
```typescript
type SyncMode = "Disabled" | "Regular" | "Encrypted";
type SyncStatus = "Disabled" | "Syncing" | "Synced" | "Error";

function isSyncEnabled(profile): boolean {
  return profile.sync_mode != null && profile.sync_mode !== "Disabled";
}
```

## Proxy
```typescript
interface ProxySettings {
  proxy_type: string;   // "http"|"https"|"socks4"|"socks5"|"ss"
  host: string;
  port: number;
  username?: string;
  password?: string;
}

interface StoredProxy {
  id: string;
  name: string;
  proxy_settings: ProxySettings;
  sync_enabled?: boolean;
  last_sync?: number;
  is_cloud_managed?: boolean;
  is_cloud_derived?: boolean;
  geo_country?: string;
  geo_state?: string;
  geo_region?: string;
  geo_city?: string;
  geo_isp?: string;
}

interface ProxyCheckResult {
  ip: string;
  city?: string;
  country?: string;
  country_code?: string;
  timestamp: number;
  is_valid: boolean;
}

const CLOUD_PROXY_ID = "cloud-included-proxy";
```

## VPN
```typescript
type VpnType = "WireGuard";

interface VpnConfig {
  id: string;
  name: string;
  vpn_type: VpnType;
  config_data: string;
  created_at: number;
  last_used?: number;
  sync_enabled?: boolean;
  last_sync?: number;
}

interface VpnStatus {
  connected: boolean;
  vpn_id: string;
  connected_at?: number;
  bytes_sent?: number;
  bytes_received?: number;
  last_handshake?: number;
}

interface VpnImportResult {
  success: boolean;
  vpn_id?: string;
  vpn_type?: VpnType;
  name: string;
  error?: string;
}
```

## Wayfern
```typescript
type WayfernOS = "windows" | "macos" | "linux" | "android" | "ios";

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
  fingerprint?: string;                      // JSON string
  randomize_fingerprint_on_launch?: boolean;
  os?: WayfernOS;
  geo_proxy_signature?: string;
}

interface WayfernLaunchResult {
  id: string;
  processId?: number;
  profilePath?: string;
  url?: string;
  cdp_port?: number;
}

// WayfernFingerprintConfig: ~70 fields (xem 07-wayfern.md hoặc types.ts:286-393)
```

## Extension
```typescript
interface Extension {
  id: string;
  name: string;
  file_name: string;
  file_type: string;
  browser_compatibility: string[];
  created_at: number;
  updated_at: number;
  sync_enabled?: boolean;
  last_sync?: number;
  version?: string;
  description?: string;
  author?: string;
  homepage_url?: string;
}

interface ExtensionGroup {
  id: string;
  name: string;
  extension_ids: string[];
  created_at: number;
  updated_at: number;
  sync_enabled?: boolean;
  last_sync?: number;
}
```

## Group
```typescript
interface ProfileGroup {
  id: string;
  name: string;
  sync_enabled?: boolean;
  last_sync?: number;
}

interface GroupWithCount {
  id: string;
  name: string;
  count: number;
  sync_enabled?: boolean;
  last_sync?: number;
}
```

## Cloud / Auth
```typescript
interface Entitlements {
  active: boolean;
  browserAutomation: boolean;
  crossOsFingerprints: boolean;
  cloudBackup: boolean;
  teamCollaboration: boolean;
  profileLimit: number;
  requestsPerHour: number;
}

interface CloudUser {
  id: string;
  email: string;
  plan: string;
  planPeriod: string | null;
  subscriptionStatus: string;
  profileLimit: number;
  cloudProfilesUsed: number;
  proxyBandwidthLimitMb: number;
  proxyBandwidthUsedMb: number;
  proxyBandwidthExtraMb: number;
  teamId?: string;
  teamName?: string;
  teamRole?: string;
  deviceOrdinal?: number | null;     // 1 = oldest = primary
  deviceCount?: number | null;
  isPrimaryDevice?: boolean | null;   // only primary can run automation
  entitlements?: Entitlements;
}

interface CloudAuthState {
  user: CloudUser;
  logged_in_at: string;
}

interface ProfileLockInfo {
  profileId: string;
  lockedBy: string;
  lockedByEmail: string;
  lockedAt: string;
  expiresAt?: string;
}

interface SyncSettings {
  sync_server_url?: string;
  sync_token?: string;
}
```

## Cookies
```typescript
interface UnifiedCookie {
  name: string;
  value: string;
  domain: string;
  path: string;
  expires: number;
  is_secure: boolean;
  is_http_only: boolean;
  same_site: number;
  creation_time: number;
  last_accessed: number;
}

interface DomainCookies {
  domain: string;
  cookies: UnifiedCookie[];
  cookie_count: number;
}

interface CookieReadResult {
  profile_id: string;
  browser_type: string;
  domains: DomainCookies[];
  total_count: number;
}

interface CookieCopyRequest {
  source_profile_id: string;
  target_profile_ids: string[];
  selected_cookies: SelectedCookie[];
}

interface CookieCopyResult {
  target_profile_id: string;
  cookies_copied: number;
  cookies_replaced: number;
  errors: string[];
}
```

## Traffic stats
```typescript
interface BandwidthDataPoint {
  timestamp: number;
  bytes_sent: number;
  bytes_received: number;
}

interface DomainAccess {
  domain: string;
  request_count: number;
  bytes_sent: number;
  bytes_received: number;
  first_access: number;
  last_access: number;
}

interface TrafficStats {
  proxy_id: string;
  profile_id?: string;
  session_start: number;
  last_update: number;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_requests: number;
  bandwidth_history: BandwidthDataPoint[];
  domains: Record<string, DomainAccess>;
  unique_ips: string[];
}

interface TrafficSnapshot {
  profile_id?: string;
  session_start: number;
  last_update: number;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_requests: number;
  current_bytes_sent: number;
  current_bytes_received: number;
  recent_bandwidth: BandwidthDataPoint[];
}
```

## Profile import
```typescript
interface DetectedProfile {
  browser: string;
  name: string;
  path: string;
  description: string;
  mapped_browser: string;
}

interface ImportProfileItem {
  source_path: string;
  browser_type?: string;
  new_profile_name: string;
  proxy_id?: string | null;   // mutually exclusive with vpn_id
  vpn_id?: string | null;
}

interface ProfileImportBatchResult {
  imported_count: number;
  skipped_count: number;
  failed_count: number;
  results: ProfileImportItemResult[];
}

interface ArchiveScanResult {
  extracted_dir: string;
  profiles: DetectedProfile[];
}
```

## App update
```typescript
interface AppUpdateInfo {
  current_version: string;
  new_version: string;
  release_notes: string;
  download_url: string;
  is_nightly: boolean;
  published_at: string;
  manual_update_required: boolean;
  release_page_url?: string;
  repo_update: boolean;
  checksums_url?: string | null;
  asset_digest?: string | null;
}

interface AppUpdateProgress {
  stage: string;        // "downloading"|"extracting"|"installing"|"completed"
  percentage?: number;
  speed?: string;      // MB/s
  eta?: string;
  message: string;
}
```

## Synchronizer (real-time CDP sync)
```typescript
interface SyncFollowerState {
  profile_id: string;
  profile_name: string;
  failed_at_url: string | null;
}

interface SyncSessionInfo {
  id: string;
  leader_profile_id: string;
  leader_profile_name: string;
  followers: SyncFollowerState[];
}
```
