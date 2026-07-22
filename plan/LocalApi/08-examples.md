# Code Examples

Ví dụ tự động hóa Donut Browser bằng nhiều ngôn ngữ.

---

## PowerShell

### Setup

```powershell
$BASE = "http://127.0.0.1:10108"
$TOKEN = "your-token-here"
$HEADERS = @{ Authorization = "Bearer $TOKEN" }
```

### Tạo profile → launch → mở URL → kill

```powershell
# 1. Tạo profile
$body = @{
    name = "automation-demo"
    browser = "wayfern"
    version = "149.0.7827.116"
} | ConvertTo-Json

$resp = Invoke-RestMethod -Uri "$BASE/v1/profiles" -Method Post `
    -Headers $HEADERS -ContentType "application/json" -Body $body
$profileId = $resp.profile.id
Write-Host "Created profile: $profileId"

# 2. Launch browser
$runBody = @{ headless = $false } | ConvertTo-Json
$runResp = Invoke-RestMethod -Uri "$BASE/v1/profiles/$profileId/run" -Method Post `
    -Headers $HEADERS -ContentType "application/json" -Body $runBody
$cdpPort = $runResp.remote_debugging_port
Write-Host "Launched on CDP port: $cdpPort"

# 3. Mở URL
$urlBody = @{ url = "https://example.com" } | ConvertTo-Json
Invoke-RestMethod -Uri "$BASE/v1/profiles/$profileId/open-url" -Method Post `
    -Headers $HEADERS -ContentType "application/json" -Body $urlBody | Out-Null
Write-Host "Opened URL"

# 4. Chờ 10 giây
Start-Sleep -Seconds 10

# 5. Kill
$killBody = @{} | ConvertTo-Json
Invoke-RestMethod -Uri "$BASE/v1/profiles/$profileId/kill" -Method Post `
    -Headers $HEADERS -ContentType "application/json" -Body $killBody | Out-Null
Write-Host "Killed browser"
```

### Batch launch nhiều profile

```powershell
$profiles = (Invoke-RestMethod -Uri "$BASE/v1/profiles" -Headers $HEADERS).profiles
$ids = $profiles | Select-Object -First 5 | ForEach-Object { $_.id }

$body = @{
    profile_ids = $ids
    headless = $false
} | ConvertTo-Json

$result = Invoke-RestMethod -Uri "$BASE/v1/profiles/batch/run" -Method Post `
    -Headers $HEADERS -ContentType "application/json" -Body $body

$result.results | ForEach-Object {
    if ($_.ok) {
        Write-Host "Profile $($_.profile_id): port $($_.remote_debugging_port)"
    } else {
        Write-Host "Profile $($_.profile_id): FAILED - $($_.error)"
    }
}
```

---

## Python

### Setup

```python
import requests

BASE = "http://127.0.0.1:10108"
TOKEN = "your-token-here"
HEADERS = {"Authorization": f"Bearer {TOKEN}"}
```

### Tạo profile → launch → mở URL → kill

```python
# 1. Tạo profile
resp = requests.post(f"{BASE}/v1/profiles", headers=HEADERS, json={
    "name": "python-demo",
    "browser": "wayfern",
    "version": "149.0.7827.116"
})
profile_id = resp.json()["profile"]["id"]
print(f"Created: {profile_id}")

# 2. Launch
resp = requests.post(f"{BASE}/v1/profiles/{profile_id}/run", headers=HEADERS, json={
    "headless": False
})
cdp_port = resp.json()["remote_debugging_port"]
print(f"CDP port: {cdp_port}")

# 3. Mở URL
requests.post(f"{BASE}/v1/profiles/{profile_id}/open-url", headers=HEADERS, json={
    "url": "https://example.com"
})

# 4. Chờ
import time
time.sleep(10)

# 5. Kill
requests.post(f"{BASE}/v1/profiles/{profile_id}/kill", headers=HEADERS, json={})
print("Done")
```

### Import proxy

```python
proxies_text = "127.0.0.1:8080:user:pass\n10.0.0.1:1080\nsocks5://user:pass@172.16.0.1:9050"

resp = requests.post(f"{BASE}/v1/proxies/import", headers=HEADERS, json={
    "format": "txt",
    "content": proxies_text
})
print(f"Imported: {resp.json()['imported_count']}")
```

### Import cookies

```python
netscape_cookies = """# Netscape HTTP Cookie File
.example.com\tTRUE\t/\tFALSE\t99999999999\tsession\tabc123
.example.com\tTRUE\t/\tFALSE\t99999999999\ttoken\txyz789"""

resp = requests.post(
    f"{BASE}/v1/profiles/{profile_id}/cookies/import",
    headers=HEADERS,
    json={"content": netscape_cookies}
)
print(f"Cookies imported: {resp.json()['cookies_imported']}")
```

---

## JavaScript / Node.js

### Setup

```javascript
const BASE = "http://127.0.0.1:10108";
const TOKEN = "your-token-here";

async function api(path, method = "GET", body = null) {
  const opts = {
    method,
    headers: {
      Authorization: `Bearer ${TOKEN}`,
      ...(body && { "Content-Type": "application/json" }),
    },
  };
  if (body) opts.body = JSON.stringify(body);
  const res = await fetch(`${BASE}${path}`, opts);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.status === 204 ? null : res.json();
}
```

### Workflow hoàn chỉnh

```javascript
// 1. Tạo profile
const { profile } = await api("/v1/profiles", "POST", {
  name: "js-demo",
  browser: "wayfern",
  version: "149.0.7827.116",
});
console.log("Created:", profile.id);

// 2. Gắn proxy
await api(`/v1/profiles/${profile.id}`, "PUT", {
  proxy_id: "05308bcb-3f11-4f09-b6fd-fb17d0823b41",
});

// 3. Launch
const { remote_debugging_port } = await api(
  `/v1/profiles/${profile.id}/run`,
  "POST",
  { headless: false }
);
console.log("CDP port:", remote_debugging_port);

// 4. Mở URL
await api(`/v1/profiles/${profile.id}/open-url`, "POST", {
  url: "https://example.com",
});

// 5. Chờ
await new Promise((r) => setTimeout(r, 10000));

// 6. Kill
await api(`/v1/profiles/${profile.id}/kill`, "POST", {});
console.log("Done");
```

### Connect CDP với Playwright/puppeteer

```javascript
const { chromium } = require("playwright");

// Sau khi launch qua API:
const browser = await chromium.connectOverCDP(
  `http://127.0.0.1:${remote_debugging_port}`
);

const context = browser.contexts()[0];
const page = context.pages()[0] || (await context.newPage());

await page.goto("https://example.com");
console.log(await page.title());

// Đừng đóng browser qua Playwright — dùng API để kill
await browser.close(); // chỉ ngắt kết nối CDP, không tắt browser
await api(`/v1/profiles/${profile.id}/kill`, "POST", {});
```

---

## Bash / curl

### Workflow hoàn chỉnh

```bash
#!/bin/bash
BASE="http://127.0.0.1:10108"
TOKEN="your-token-here"
AUTH="Authorization: Bearer $TOKEN"
CT="Content-Type: application/json"

# 1. Tạo profile
PROFILE_ID=$(curl -s -X POST -H "$AUTH" -H "$CT" \
  -d '{"name":"bash-demo","browser":"wayfern","version":"149.0.7827.116"}' \
  "$BASE/v1/profiles" | jq -r '.profile.id')
echo "Profile: $PROFILE_ID"

# 2. Launch
CDP_PORT=$(curl -s -X POST -H "$AUTH" -H "$CT" -d '{"headless":false}' \
  "$BASE/v1/profiles/$PROFILE_ID/run" | jq -r '.remote_debugging_port')
echo "CDP: $CDP_PORT"

# 3. Mở URL
curl -s -X POST -H "$AUTH" -H "$CT" \
  -d '{"url":"https://example.com"}' \
  "$BASE/v1/profiles/$PROFILE_ID/open-url"

# 4. Chờ
sleep 10

# 5. Kill
curl -s -X POST -H "$AUTH" -H "$CT" -d '{}' \
  "$BASE/v1/profiles/$PROFILE_ID/kill"
echo "Done"
```
