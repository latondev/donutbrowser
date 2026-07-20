# Cấu trúc Frontend (`src/`)

## Công nghệ
- **Next.js 16** (App Router, Turbopack dev)
- **React 19**
- **TypeScript 6** (strict)
- **Tailwind CSS 4** + **shadcn/ui** (Radix primitives)
- **i18next** + react-i18next (10 locale)
- **TanStack Table v8** + TanStack Virtual (profile data table)
- **Motion** (framer-motion) cho animation
- **lucide-react** + react-icons
- **Sonner** toast
- **cmdk** (command palette)
- **onborda** (onboarding tour)
- **recharts** (charts)

## Cây thư mục
```
src/
├── app/
│   ├── layout.tsx          # Root layout, providers, i18n init
│   └── page.tsx            # Trang chính (Home) — ~2144 dòng
│                            #   - Quản lý TẤT cả dialog state
│                            #   - Global keyboard shortcuts dispatcher
│                            #   - useProfileEvents, useCloudAuth, useVersionUpdater...
│                            #   - RailNav (left sidebar) + content area
├── components/              # ~60 component
│   ├── ui/                 # shadcn/ui primitives (dialog, button, table, tabs...)
│   ├── icons/              # Custom icons
│   ├── *-dialog.tsx        # Dialog components (create/edit/delete/import)
│   ├── profile-data-table.tsx  # Bảng profile chính (TanStack Table)
│   ├── home-header.tsx     # Header với search, actions
│   ├── rail-nav.tsx        # Left navigation rail (AppPage)
│   ├── command-palette.tsx # Mod+K command palette
│   ├── account-page.tsx    # Sub-page: tài khoản cloud
│   ├── settings-dialog.tsx # Sub-page: settings
│   ├── proxy-management-dialog.tsx  # Sub-page: proxy management (tabbed)
│   ├── extension-management-dialog.tsx  # Sub-page
│   ├── integrations-dialog.tsx  # Sub-page: MCP/API integrations
│   ├── wayfern-config-*.tsx  # Wayfern fingerprint config form
│   ├── theme-provider.tsx
│   ├── i18n-provider.tsx
│   └── onboarding-provider.tsx
├── hooks/                  # Event-driven React hooks
│   ├── use-profile-events.ts   # CRUD + running state cho profile
│   ├── use-proxy-events.ts
│   ├── use-vpn-events.ts
│   ├── use-group-events.ts
│   ├── use-extension-events.ts
│   ├── use-cloud-auth.ts
│   ├── use-version-updater.ts
│   ├── use-browser-*.ts      # Browser download/setup/state/support
│   ├── use-sync-session.ts
│   ├── use-team-locks.ts
│   ├── use-permissions.ts
│   ├── use-language.ts
│   ├── use-commercial-trial.ts
│   └── use-wayfern-terms.ts
├── lib/                    # Utilities (không phải React)
│   ├── themes.ts           # CSS variables cho theme colors (xuất object)
│   ├── backend-errors.ts   # Translate {"code":"..."} → t("backendErrors.foo")
│   ├── entitlements.ts     # Cloud plan capabilities
│   ├── shortcuts.ts        # SHORTCUTS[] + formatShortcut + matchesShortcut
│   ├── toast-utils.ts      # showSuccessToast, showErrorToast...
│   ├── browser-utils.ts
│   ├── flag-utils.ts       # Country flag
│   ├── dns-blocklist-levels.ts
│   ├── confetti.ts
│   ├── donut-physics.ts
│   ├── motion.ts
│   ├── logger.ts
│   ├── name-utils.ts
│   ├── error-utils.ts
│   └── utils.ts            # cn() — tailwind-merge
├── i18n/
│   └── locales/            # 10 file JSON: en, es, fr, ja, ko, pt, ru, tr, vi, zh
├── styles/
└── types.ts                # Shared TypeScript interfaces (BrowserProfile, ProxySettings...)
```

## Component patterns

### Sub-page Dialog
Một `<Dialog>` trở thành sub-page (không modal, không center) khi truyền prop `subPage`:
- Tabbed: dùng `Tabs` với `key={initialTab}` để remount khi đổi tab từ outside
- Class strings tuned cho sub-page chrome (xem AGENTS.md)

### Event-driven hooks
Hooks subscribe Tauri events và cập nhật state. Pattern:
```typescript
const { profiles, runningProfiles } = useProfileEvents();
// Internal: invoke("list_browser_profiles") + listen("profile-running-changed")
```

### Keyboard shortcuts
- Tất cả app-wide shortcuts ở `src/lib/shortcuts.ts` (`SHORTCUTS[]` + `ShortcutId`)
- Dispatcher trong `src/app/page.tsx::runShortcut`
- `formatShortcut(s)` trả token platform-correct (`⌘` mac, `Ctrl` khác)
- Mod+1..9 = switch group (dynamic, `matchesGroupDigit`)
- Mod+K = command palette

### Command Palette
- shadcn `Command` primitive
- `fuzzyFilter` (token-AND) trong `command-palette.tsx`
- `CommandDialog` forward `filter`/`shouldFilter`

## Quy tắc UI bắt buộc (từ AGENTS.md)
1. **Không hardcoded Tailwind colors** (`text-red-500`, `bg-green-600`...) — chỉ dùng CSS variables từ `themes.ts`:
   - `background`, `foreground`, `card`, `popover`, `primary`, `secondary`, `muted`, `accent`, `destructive`, `success`, `warning`, `border`, `chart-1..5`
   - Lighter variant: opacity (`bg-destructive/10`)
2. **Mọi user-facing string** qua `t("namespace.key")`:
   - Không raw English literals trong JSX/toast/dialog/placeholder/header/empty-state
   - Thêm key vào TẤT CẢ 10 locale files
   - Không dùng `t(key, "fallback")` 2-arg — fallback mask missing translation
   - Empty-string values trong non-English locale bị cấm
3. **Backend errors**: luôn JSON `{"code": "FOO_BAR"}` từ Rust, resolve qua `translateBackendError(t, err)` → `t("backendErrors.fooBar")`

## Scripts (từ package.json)
- `pnpm dev` — Next dev server (port 12341, turbopack)
- `pnpm build` — Production build
- `pnpm lint` — `biome check src/` + `tsc --noEmit` + rust clippy + typos spellcheck
- `pnpm test` — Rust unit + sync e2e + proxy/vpn integration
- `pnpm format` — biome write + cargo fmt/clippy-fix
- `pnpm tauri` — chạy Tauri dev/build (qua run-with-env.mjs)
- `pnpm copy-proxy-binary` — copy donut-proxy binary (chạy ở prebuild/predev/precargo)
