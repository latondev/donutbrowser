# Proxy & VPN

## Proxy

### Loại hỗ trợ
- HTTP, HTTPS, SOCKS4, SOCKS5, Shadowsocks (`ss`)
- Per-profile, dynamic proxy URLs

### Kiến trúc
```
Browser (Wayfern/Chromium)
    │
    ▼
donut-proxy worker (Rust binary, per-profile process)
    │  - 1 worker per profile launch
    │  - $TMPDIR/donut-proxy-<config_id>.log
    │
    ▼
Upstream proxy (HTTP/SOCKS5/SS) hoặc VPN worker SOCKS5
```

### Modules
| File | Vai trò |
|---|---|
| `proxy_manager.rs` | `PROXY_MANAGER` singleton — StoredProxy CRUD, check validity, cleanup dead proxies, IP geolocation |
| `proxy_storage.rs` | Proxy config persistence (JSON files), `is_process_running`, `list_proxy_configs`, `delete_proxy_config` |
| `proxy_server.rs` | donut-proxy worker lifecycle (start/stop/log) |
| `proxy_runner.rs` | `start_proxy_process`/`stop_proxy_process` — spawn donut-proxy binary |
| `socks5_local.rs` | Local SOCKS5 server (cho VPN validation) |
| `bin/proxy_server.rs` | Standalone donut-proxy binary entry |

### StoredProxy (`src/types.ts`)
```typescript
interface StoredProxy {
  id: string;
  name: string;
  proxy_settings: ProxySettings;
  sync_enabled?: boolean;
  last_sync?: number;
  is_cloud_managed?: boolean;   // Donut cloud proxy
  is_cloud_derived?: boolean;
  geo_country?: string;          // Geo data từ IP
  geo_state?: string;
  geo_region?: string;
  geo_city?: string;
  geo_isp?: string;
}

interface ProxySettings {
  proxy_type: string;  // "http", "https", "socks4", "socks5", "ss"
  host: string;
  port: number;
  username?: string;
  password?: string;
}
```

### Proxy validation
`check_proxy_validity(proxy_id, settings)`:
1. Start donut-proxy worker với upstream = proxy settings
2. Fetch public IP qua local proxy (`ip_utils::fetch_public_ip`)
3. Resolve geolocation (city, country, country_code)
4. Return `ProxyCheckResult { ip, city, country, country_code, timestamp, is_valid }`
5. Cached qua `get_cached_proxy_check(proxy_id)`

### Cloud proxy
- `CLOUD_PROXY_ID = "cloud-included-proxy"` — Donut cloud managed proxy
- Không pre-validate (chỉ fail mode = 402 usage limit tại request time)
- `cloud_get_countries/regions/cities/isps`, `create_cloud_location_proxy`

### Import/Export
- `export_proxies(format)` — JSON hoặc TXT
- `import_proxies_json(content)` — import từ JSON
- `parse_txt_proxies(content)` — parse text, return `ProxyParseResult` (parsed/ambiguous/invalid)
- `import_proxies_from_parsed(parsed, name_prefix)` — import từ parsed lines

### donut-proxy worker logs
`$TMPDIR/donut-proxy-<config_id>.log`:
- CONNECT requests, upstream accept/reject (vd `HTTP/1.1 402 user reached limit`)
- Tunnel errors ở INFO/WARN
- Finer detail ở TRACE (`RUST_LOG=donut_proxy=trace`)
- Warning đáng chú ý: `Upstream CONNECT response coalesced N byte(s) of payload — these would be dropped without forwarding` = bug thật trong `handle_connect_from_buffer`

### Cleanup
- Startup: kill orphaned proxy workers (PID died, no associated running browser)
- Preserve workers cho profile có browser vẫn running (app crash recovery)
- Periodic (30s): `cleanup_dead_proxies` — cleanup proxy cho dead browser PID
- `Cleanup: browser PID X is dead, stopping proxy worker <id>` trong log

## VPN

### Loại
- WireGuard (qua boringtun, userspace)
- Per-profile config

### Kiến trúc
```
Browser → donut-proxy → VPN worker SOCKS5 (127.0.0.1:port) → WireGuard tunnel → internet
```

VPN worker là **detached process** — survives GUI shutdown (cho browser vẫn chạy sau khi app tắt).

### Modules
| File | Vai trò |
|---|---|
| `vpn/mod.rs` | `VpnConfig`, `VpnImportResult`, `VpnStatus`, `VPN_STORAGE` (Mutex) |
| `vpn/config.rs` | WireGuard config parsing |
| `vpn/wireguard.rs` | WireGuard tunnel (boringtun) |
| `vpn/tunnel.rs` | Tunnel management |
| `vpn/socks5_server.rs` | SOCKS5 server cho VPN |
| `vpn/storage.rs` | VPN config persistence |
| `vpn_worker_runner.rs` | Start/stop VPN worker process (detached) |
| `vpn_worker_storage.rs` | VPN worker state (pid, local_port, vpn_id) |

### VpnConfig (`src/types.ts`)
```typescript
type VpnType = "WireGuard";

interface VpnConfig {
  id: string;
  name: string;
  vpn_type: VpnType;
  config_data: string;     // Raw config content
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
```

### VPN validation
`check_vpn_validity(vpn_id)` (`check_vpn_validity_core`):
1. Start VPN worker (hoặc dùng existing)
2. Start donut-proxy với upstream = `socks5://127.0.0.1:<vpn_port>`
3. Fetch public IP (3 attempts, 1s delay)
4. Resolve geolocation
5. Stop proxy + VPN worker (nếu không có sẵn)
6. Return `ProxyCheckResult`

### VPN worker lifecycle
- `connect_vpn(vpn_id)` = `start_vpn_worker(vpn_id)` + update `last_used`
- `disconnect_vpn(vpn_id)` = `stop_vpn_worker_by_vpn_id(vpn_id)`
- `get_vpn_status(vpn_id)` — check PID running
- `list_active_vpn_connections()` — filter running workers
- Startup: kill orphaned VPN workers (preserve cho running browsers)

### Profile network validation
`validate_profile_network(proxy_id, vpn_id)` — shared bởi Tauri command, REST API, MCP create:
- VPN ưu tiên nếu có
- Cloud proxy (`CLOUD_PROXY_ID`) skip pre-validate
- Proxy thường: `check_proxy_validity`, fail → `{"code": "PROXY_NOT_WORKING"}` hoặc `PROXY_PAYMENT_REQUIRED` (402)

## Fingerprint consistency (`fingerprint_consistency.rs`)
Launch-time check: proxy exit IP location vs fingerprint timezone/language.
- `check_profile_fingerprint_consistency`
- `match_profile_fingerprint_to_exit`
- Emit warning dialog nếu mismatch
