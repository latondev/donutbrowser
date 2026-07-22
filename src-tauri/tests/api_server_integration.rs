//! Integration tests for the REST API server (`src-tauri/src/api_server.rs`).
//!
//! Boots a real `axum` server bound to an ephemeral port via the same
//! `start_api_server` Tauri command the GUI uses, with a real Wry-runtime
//! Tauri `AppHandle` (built without running the event loop) and a sandboxed
//! data dir via `DONUTBROWSER_DATA_DIR`. Each test issues HTTP requests
//! through `reqwest` against the live server and asserts on status + body.
//!
//! Covers: auth (no token, wrong token, valid token), profile CRUD,
//! group CRUD, proxy CRUD, tags, extensions listing, and OpenAPI spec
//! presence. Automation-only endpoints (`/run`, `/kill`, `/batch/*`,
//! `/import`, `/cookies`) are gated behind `CLOUD_AUTH` entitlements and
//! require a downloaded Wayfern binary, so they are intentionally excluded —
//! the goal here is to lock in the CRUD contract that the REST API exposes to
//! non-pro users, not to reproduce the MCP e2e flow.
//!
//! Run with: `cargo test --test api_server_integration`

mod common;

use donutbrowser_lib::settings_manager::SettingsManager;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use serial_test::serial;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Isolated per-process data/cache dirs so these tests never touch the user's
/// real Donut Browser data. Set once on first use; restored on Drop.
struct TestEnvGuard {
  _root: PathBuf,
  previous_data_dir: Option<String>,
  previous_cache_dir: Option<String>,
}

impl TestEnvGuard {
  fn new() -> Self {
    static TEST_RUNTIME_ROOT: OnceLock<PathBuf> = OnceLock::new();
    let root = TEST_RUNTIME_ROOT
      .get_or_init(|| {
        std::env::temp_dir().join(format!("donutbrowser-api-e2e-{}", std::process::id()))
      })
      .clone();
    let data_dir = root.join("data");
    let cache_dir = root.join("cache");
    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::remove_dir_all(&cache_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::create_dir_all(&cache_dir).unwrap();

    let previous_data_dir = std::env::var("DONUTBROWSER_DATA_DIR").ok();
    let previous_cache_dir = std::env::var("DONUTBROWSER_CACHE_DIR").ok();
    std::env::set_var("DONUTBROWSER_DATA_DIR", &data_dir);
    std::env::set_var("DONUTBROWSER_CACHE_DIR", &cache_dir);

    Self {
      _root: root,
      previous_data_dir,
      previous_cache_dir,
    }
  }
}

impl Drop for TestEnvGuard {
  fn drop(&mut self) {
    if let Some(value) = &self.previous_data_dir {
      std::env::set_var("DONUTBROWSER_DATA_DIR", value);
    } else {
      std::env::remove_var("DONUTBROWSER_DATA_DIR");
    }
    if let Some(value) = &self.previous_cache_dir {
      std::env::set_var("DONUTBROWSER_CACHE_DIR", value);
    } else {
      std::env::remove_var("DONUTBROWSER_CACHE_DIR");
    }
  }
}

/// Fixture: either a self-contained mock server (default) or a pointer to a
/// real running app server (when `DONUT_LIVE_API_URL` +
/// `DONUT_LIVE_API_TOKEN` are set).
///
/// # Live mode
///
/// Point tests at your running app instead of spinning up a mock server:
///
/// ```powershell
/// $env:RUST_TEST_THREADS = "1"  # tests are #[serial] anyway
/// $env:DONUT_LIVE_API_URL   = "http://127.0.0.1:10108"
/// $env:DONUT_LIVE_API_TOKEN = "<paste-token-from-Settings>"
/// cargo test --test api_server_integration
/// ```
///
/// Leave both env vars unset to fall back to the self-contained mock server
/// (default when running in CI / without the app).
struct ApiFixture {
  base_url: String,
  token: String,
  _env: Option<TestEnvGuard>,
  // The Tauri `App` owns the Wry event loop, which on Windows must live on
  // the OS main thread. The fixture is therefore constructed inside
  // `run_async` and never moved across threads.
  _app: Option<tauri::App>,
}

impl ApiFixture {
  async fn start() -> Self {
    // Live mode: use a real running app's API server. No mock app, no
    // in-process server — just point reqwest at the live URL + token.
    if let (Ok(base_url), Ok(token)) = (
      std::env::var("DONUT_LIVE_API_URL"),
      std::env::var("DONUT_LIVE_API_TOKEN"),
    ) {
      let base_url = base_url.trim_end_matches('/').to_string();
      // Warm up the connection so Windows Firewall settles before the test
      // assertions run. Without this, the first few requests from the test
      // binary are silently dropped, causing spurious "connection refused".
      warmup_connection(&base_url).await;
      return Self {
        base_url,
        token,
        _env: None,
        _app: None,
      };
    }

    // Mock mode: spin up an isolated API server in-process.
    let env = TestEnvGuard::new();

    // Build a real Wry-runtime Tauri app without running the event loop and
    // without requiring `generate_context!()` (which needs the crate to be the
    // app crate). `mock_context` + `noop_assets` gives us a minimal `Context`
    // that `Builder::default()` (Wry runtime) accepts. `build()` creates the
    // AppHandle — needed by the API server and settings manager — but does
    // not start a window or event loop.
    //
    // `any_thread()` is required on Windows because the test harness spawns
    // each `#[test]` on a worker thread, not the OS main thread, and the Wry
    // (tao) event loop otherwise panics.
    let app = tauri::Builder::default()
      .any_thread()
      .build(tauri::test::mock_context(tauri::test::noop_assets()))
      .expect("failed to build mock Tauri app");
    let handle = app.handle().clone();

    // Pre-seed the Wayfern terms-accepted marker so the terms-check middleware
    // (which gates every v1 route) lets requests through.
    write_terms_accepted_marker();

    // Generate + persist an API token the auth middleware can read back.
    let token = SettingsManager::instance()
      .generate_api_token(&handle)
      .await
      .expect("failed to generate API token");

    // Bind the real API server to an ephemeral port.
    let port = donutbrowser_lib::api_server::start_api_server(None, handle)
      .await
      .expect("failed to start API server");

    Self {
      base_url: format!("http://127.0.0.1:{port}"),
      token,
      _env: Some(env),
      _app: Some(app),
    }
  }

  fn url(&self, path: &str) -> String {
    format!("{}{path}", self.base_url)
  }

  fn authed_client(&self) -> Client {
    // `.no_proxy()` is critical when running in live mode against the
    // running app: Donut Browser may have set a system-wide proxy, which
    // reqwest's `system-proxy` feature would inherit and route localhost
    // traffic through, breaking the test connection.
    Client::builder()
      .no_proxy()
      .pool_idle_timeout(Some(std::time::Duration::from_secs(0)))
      .default_headers({
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
          reqwest::header::AUTHORIZATION,
          reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token))
            .expect("token is a valid header value"),
        );
        h
      })
      .build()
      .expect("failed to build reqwest client")
  }

  fn anon_client() -> Client {
    Client::builder()
      .no_proxy()
      .pool_idle_timeout(Some(std::time::Duration::from_secs(0)))
      .build()
      .expect("failed to build reqwest client")
  }
}

/// Write the Wayfern `license-accepted` marker with a current timestamp so
/// `WayfernTermsManager::is_terms_accepted()` returns true.
fn write_terms_accepted_marker() {
  use std::fs;
  use std::path::PathBuf;

  #[cfg(target_os = "windows")]
  fn marker_path() -> Option<PathBuf> {
    let app_data = std::env::var_os("APPDATA")?;
    Some(
      PathBuf::from(app_data)
        .join("Wayfern")
        .join("license-accepted"),
    )
  }
  #[cfg(target_os = "macos")]
  fn marker_path() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|b| {
      b.home_dir()
        .join("Library/Application Support/Wayfern/license-accepted")
    })
  }
  #[cfg(target_os = "linux")]
  fn marker_path() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
      let p = PathBuf::from(xdg);
      if !p.as_os_str().is_empty() {
        return Some(p.join("Wayfern").join("license-accepted"));
      }
    }
    directories::BaseDirs::new().map(|b| b.home_dir().join(".config/Wayfern/license-accepted"))
  }

  if let Some(path) = marker_path() {
    if let Some(parent) = path.parent() {
      let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&path, format!("{}", now_secs()));
  }
}

fn now_secs() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

async fn body_as_json(resp: reqwest::Response) -> Value {
  resp.json::<Value>().await.unwrap_or(Value::Null)
}

/// In live mode, Windows Firewall can silently drop the first outgoing
/// connections from the test binary. This warmup connects once (retrying for
/// up to 10s) so the firewall prompt/rule settles before the real tests run.
/// Called once at the start of each `#[test]` via `ApiFixture::start()`.
async fn warmup_connection(base_url: &str) {
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
  let client = Client::builder()
    .no_proxy()
    .build()
    .expect("failed to build warmup client");
  loop {
    let ok = client
      .get(format!("{base_url}/openapi.json"))
      .send()
      .await
      .is_ok();
    if ok {
      return;
    }
    if std::time::Instant::now() >= deadline {
      return;
    }
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
  }
}

/// True when tests run against a self-contained mock server (default). False
/// when `DONUT_LIVE_API_URL` + `DONUT_LIVE_API_TOKEN` point tests at the
/// running app — in that mode, "empty initially" assertions are skipped
/// because the real app likely already has data.
fn is_mock_mode() -> bool {
  std::env::var("DONUT_LIVE_API_URL").is_err() && std::env::var("DONUT_LIVE_API_TOKEN").is_err()
}

/// Drive an async test body on the OS main thread, where the Wry event loop
/// is allowed to live. Each test is a plain `#[test]` that delegates to this.
fn run_async<F>(test: F)
where
  F: std::future::Future<Output = ()>,
{
  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("failed to build tokio runtime");
  // `LocalSet` keeps the !Send `tauri::App` on this thread even though the
  // async future references it — the Wry event loop never crosses threads.
  let local = tokio::task::LocalSet::new();
  local.block_on(&rt, test);
}

// ---------------------------------------------------------------------------
// Auth middleware
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn missing_authorization_header_is_401() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = ApiFixture::anon_client()
      .get(fx.url("/v1/profiles"))
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
  })
}

#[test]
#[serial]
fn wrong_bearer_token_is_401() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = ApiFixture::anon_client()
      .get(fx.url("/v1/profiles"))
      .header("Authorization", "Bearer deadbeef-not-a-real-token")
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
  })
}

#[test]
#[serial]
fn valid_token_reaches_handler() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/profiles"))
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert_eq!(body["total"], json!(0), "fresh data dir has no profiles");
    }
    assert!(body["profiles"].is_array());
  })
}

// ---------------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn list_profiles_returns_empty_initially() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/profiles"))
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert_eq!(body["total"], json!(0));
      assert_eq!(body["profiles"].as_array().unwrap().len(), 0);
    }
  })
}

#[test]
#[serial]
fn get_unknown_profile_returns_404() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/profiles/00000000-0000-0000-0000-000000000000"))
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  })
}

#[test]
#[serial]
fn create_profile_rejects_non_wayfern_browser() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .post(fx.url("/v1/profiles"))
      .json(&json!({ "name": "p", "browser": "chromium" }))
      .send()
      .await
      .expect("request failed");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  })
}

#[test]
#[serial]
fn create_profile_rejects_invalid_body() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .post(fx.url("/v1/profiles"))
      .json(&json!({ "proxy_id": "nope" }))
      .send()
      .await
      .expect("request failed");
    assert!(
      resp.status().is_client_error(),
      "expected 4xx for invalid body, got {}",
      resp.status()
    );
  })
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn group_crud_lifecycle() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();

    let resp = client.get(fx.url("/v1/groups")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert!(body.as_array().unwrap().is_empty());
    }

    let resp = client
      .post(fx.url("/v1/groups"))
      .json(&json!({ "name": "TestGroup" }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    let group_id = body["id"].as_str().expect("id").to_string();
    assert_eq!(body["name"], json!("TestGroup"));
    assert_eq!(body["profile_count"], json!(0));

    let resp = client
      .get(fx.url(&format!("/v1/groups/{group_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert_eq!(body["name"], json!("TestGroup"));

    let resp = client
      .get(fx.url("/v1/groups/does-not-exist"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp = client
      .put(fx.url(&format!("/v1/groups/{group_id}")))
      .json(&json!({ "name": "RenamedGroup" }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert_eq!(body["name"], json!("RenamedGroup"));

    let resp = client
      .put(fx.url(&format!("/v1/groups/{group_id}")))
      .json(&json!({ "name": "   " }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let _other = client
      .post(fx.url("/v1/groups"))
      .json(&json!({ "name": "OtherGroup" }))
      .send()
      .await
      .unwrap();
    let resp = client
      .post(fx.url("/v1/groups"))
      .json(&json!({ "name": "OtherGroup" }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = client
      .delete(fx.url(&format!("/v1/groups/{group_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = client
      .delete(fx.url(&format!("/v1/groups/{group_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  })
}

// ---------------------------------------------------------------------------
// Proxies
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn proxy_crud_lifecycle() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();

    let resp = client.get(fx.url("/v1/proxies")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert!(body.as_array().unwrap().is_empty());
    }

    let create_body = json!({
      "name": "TestProxy",
      "proxy_settings": {
        "proxy_type": "socks5",
        "host": "127.0.0.1",
        "port": 1080,
        "username": null,
        "password": null
      }
    });
    let resp = client
      .post(fx.url("/v1/proxies"))
      .json(&create_body)
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    let proxy_id = body["id"].as_str().expect("id").to_string();
    assert_eq!(body["name"], json!("TestProxy"));
    assert_eq!(body["proxy_settings"]["proxy_type"], json!("socks5"));
    assert_eq!(body["proxy_settings"]["port"], json!(1080));

    let resp = client
      .post(fx.url("/v1/proxies"))
      .json(&json!({
        "name": "   ",
        "proxy_settings": create_body["proxy_settings"]
      }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = client
      .get(fx.url(&format!("/v1/proxies/{proxy_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert_eq!(body["name"], json!("TestProxy"));

    let resp = client
      .get(fx.url("/v1/proxies/does-not-exist"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp = client
      .put(fx.url(&format!("/v1/proxies/{proxy_id}")))
      .json(&json!({
        "name": "RenamedProxy",
        "proxy_settings": {
          "proxy_type": "http",
          "host": "10.0.0.1",
          "port": 8080,
          "username": "u",
          "password": "p"
        }
      }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert_eq!(body["name"], json!("RenamedProxy"));
    assert_eq!(body["proxy_settings"]["proxy_type"], json!("http"));
    assert_eq!(body["proxy_settings"]["port"], json!(8080));
    assert_eq!(body["proxy_settings"]["username"], json!("u"));

    let resp = client
      .post(fx.url("/v1/proxies"))
      .json(&json!({
        "name": "RenamedProxy",
        "proxy_settings": create_body["proxy_settings"]
      }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = client
      .delete(fx.url(&format!("/v1/proxies/{proxy_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = client
      .delete(fx.url(&format!("/v1/proxies/{proxy_id}")))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  })
}

#[test]
#[serial]
fn import_proxies_txt_parses_valid_entries() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();

    let resp = client
      .post(fx.url("/v1/proxies/import"))
      .json(&json!({
        "format": "txt",
        "content": "127.0.0.1:1080:u:p\n10.0.0.1:8080\n",
        "name_prefix": "Imported-"
      }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert_eq!(body["imported_count"], json!(2));
    assert_eq!(body["skipped_count"], json!(0));
    let proxies = body["proxies"].as_array().unwrap();
    assert_eq!(proxies.len(), 2);
    assert!(proxies[0]["name"]
      .as_str()
      .unwrap()
      .starts_with("Imported-"));
  })
}

#[test]
#[serial]
fn import_proxies_rejects_unknown_format() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();
    let resp = client
      .post(fx.url("/v1/proxies/import"))
      .json(&json!({ "format": "csv", "content": "anything" }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  })
}

#[test]
#[serial]
fn import_proxies_rejects_empty_txt() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();
    let resp = client
      .post(fx.url("/v1/proxies/import"))
      .json(&json!({ "format": "txt", "content": "// nothing here\n  \n" }))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  })
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn list_tags_returns_empty_initially() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/tags"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert!(body.as_array().unwrap().is_empty());
    }
  })
}

// ---------------------------------------------------------------------------
// VPNs (listing only — create requires WireGuard config_data)
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn list_vpns_returns_empty_initially() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/vpns"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    if is_mock_mode() {
      assert!(body.as_array().unwrap().is_empty());
    }
  })
}

#[test]
#[serial]
fn get_unknown_vpn_returns_404() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/vpns/does-not-exist"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  })
}

// ---------------------------------------------------------------------------
// Extensions (listing + delete only)
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn list_extensions_returns_array() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/extensions"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert!(body.is_array(), "expected an array, got {body}");
  })
}

#[test]
#[serial]
fn list_extension_groups_returns_array() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .get(fx.url("/v1/extension-groups"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_as_json(resp).await;
    assert!(body.is_array(), "expected an array, got {body}");
  })
}

#[test]
#[serial]
fn delete_unknown_extension_returns_404() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = fx
      .authed_client()
      .delete(fx.url("/v1/extensions/does-not-exist"))
      .send()
      .await
      .unwrap();
    assert!(
      resp.status().is_client_error(),
      "expected 4xx for missing extension, got {}",
      resp.status()
    );
  })
}

// ---------------------------------------------------------------------------
// OpenAPI spec
// ---------------------------------------------------------------------------

#[test]
#[serial]
fn openapi_spec_is_served() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let client = fx.authed_client();
    let resp = client.get(fx.url("/openapi.json")).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let spec = body_as_json(resp).await;
    assert!(spec.is_object());
    let paths = spec["paths"].as_object().expect("paths is an object");
    for path in [
      "/v1/profiles",
      "/v1/groups",
      "/v1/proxies",
      "/v1/vpns",
      "/v1/extensions",
    ] {
      assert!(
        paths.contains_key(path),
        "ApiDoc should advertise {path}, got: {:?}",
        paths.keys().collect::<Vec<_>>()
      );
    }
  })
}

#[test]
#[serial]
fn openapi_endpoint_does_not_require_auth() {
  run_async(async {
    let fx = ApiFixture::start().await;
    let resp = ApiFixture::anon_client()
      .get(fx.url("/openapi.json"))
      .send()
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  })
}
