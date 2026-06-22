# Design Audit Evidence — AeroMail Frontend UI

This file consolidates the evidence gathered by the five Phase-1 subagents. All scores and verdicts are derived from this evidence.

---

## Structural Evidence

### Interactive-element count
- Grand total across audited surfaces: ~104 interactive elements (counting template occurrences, not runtime multiplied list items).
- Largest clusters:
  - `SettingsView.vue`: 15 interactive controls across AI/translation provider forms.
  - `MailViewer.vue`: 12 controls (8 toolbar buttons + attachments + delete dialog).
  - `AppSidebar.vue`: 14 controls (compose, folders, accounts, settings, AI assistant, account list).
  - `ComposeEditor.vue`: 11 toolbar formatting buttons.

### Max nesting depth
- Deepest observed path: `AppLayout > div > div > main > MailViewer > div > div > div > div > button` — **9 DOM levels**.
- Source: `AppLayout.vue:22` → `AppLayout.vue:45` → `InboxView.vue:6` → `MailViewer.vue:187-215`.

### Repeated patterns
- Functional duplicates:
  - Star / Archive / Spam / Delete appear in both `MailViewer.vue` toolbar and `ContextMenu.vue`.
  - Language switcher appears in `StatusBar.vue:71` and `CommandPalette.vue:49`.
  - AI Assistant toggle appears in `AppSidebar.vue:173` and `CommandPalette.vue:64`.
  - Account list appears read-only in `AppSidebar.vue:127` and functional in `AccountList.vue:27`.
  - Account-avatar initials repeated in sidebar, account list, and mail header.
- Wrapper components with no distinct behavior:
  - `InboxView.vue`: pure wrapper around `MailViewer`.
  - `AccountsView.vue`: thin layout wrapper.
  - `ComposeAttachmentList.vue`: presentational list only.

### Dead props / unused imports
- **0 confirmed unused imports** in audited Vue/TS files.
- All imported components appear to be referenced.

### File:line anchors
- Layout grid: `AppLayout.vue:21-57`
- Sidebar: `AppSidebar.vue:80-181`
- Mail list: `MailList.vue:174-299`
- Mail viewer: `MailViewer.vue:186-425`
- Compose: `ComposeView.vue:1-173`
- Settings: `SettingsView.vue:143-396`

---

## Visual Evidence

### Spacing scale
- CSS tokens: `theme.css:39-48` define `--space-1` through `--space-12` (4, 8, 12, 16, 20, 24, 32, 40, 48 px).
- Tailwind utilities mostly follow the scale, but ad-hoc values appear:
  - `py-1.5` / `px-1.5` (6 px) in `AccountForm.vue:114`, `SettingsView.vue:156`, etc.
  - Arbitrary widths: `min-w-[120px]` (`RecipientInput.vue:26`), `w-[360px]` (`AiAssistantPanel.vue:45`), `w-[560px]` (`CommandPalette.vue:211`).

### Type scale
- **No explicit type-scale tokens** in `theme.css` or `fonts.css`.
- Relies on Tailwind defaults (`text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`) plus custom utilities `text-h1` and `text-tiny` whose values are not inspectable without the Tailwind config.

### Color palette
- ~35+ unique color declarations including dark/light variants and opacity modifiers.
- Systematic semantic tokens for core UI (`--background`, `--panel`, `--card`, `--primary`, `--text`, etc.).
- Ad-hoc hardcoded functional accents: `text-yellow-500` (star), `text-red-500` (spam/delete), `bg-yellow-200/800` (search highlight), `bg-primary/5`, `bg-primary/10`, `bg-primary/20`, `bg-muted/10`, `bg-muted/20`, `bg-danger/10`, `bg-card/50`, `bg-card/30`, `bg-panel/50`, `text-muted/60`.

### Lowest contrast (INFERRED)
- `text-disabled` on `bg-panel`: ~3.8:1 dark, ~2.4:1 light → fails WCAG AA.
- `text-muted/60` on `bg-background`: ~2.5:1 → fails WCAG AA.
- `text-warning` on `bg-panel` dark: ~4.0:1 → marginal.

### States present checklist
| State | Status | Evidence |
|-------|--------|----------|
| empty | PRESENT | `MailList.vue:193-199`, `MailViewer.vue:188-205`, `AccountList.vue:23-25`, `SettingsView.vue:164-168`, `CommandPalette.vue:262-267` |
| loading | PRESENT | `MailList.vue:181-191`, `AccountList.vue:20-22`, `MailViewer.vue:387-395`, `CommandPalette.vue:222-224` |
| error | PRESENT | `AccountForm.vue:203`, `AccountForm.vue:212`, `AccountList.vue:51-53`, `ComposeView.vue:77-80` |
| success | PRESENT | `ToastContainer.vue:10`, `ToastContainer.vue:21`, `MailList.vue:142-153`, `BulkActions.vue:19-28` |
| focus | PRESENT | `AccountForm.vue:114`, `RecipientInput.vue:5`, `ComposeHeader.vue:49` |
| disabled | PRESENT | `ComposeHeader.vue:16`, `AiAssistantPanel.vue:86`, `SettingsView.vue:231` |

### Visual noise
- **Borders are very heavy**: almost every panel, card, toolbar, and list item has a `border-border` 1 px border.
- **Shadows are functional only**: `shadow-modal`, `shadow-toast`, `shadow-lg` used for overlays.
- **Badges**: unread count badges in `AppSidebar.vue:113-118`.
- **Animations**: `animate-spin` loading, `animate-pulse` sync dot, Vue `<TransitionGroup>` toasts, `<Transition>` command palette.
- **No gradients** observed.

### Dark / light mode
- Dark mode is default and systematic: `main.ts:9`, `theme.css:56-73`.
- Light mode implemented: `theme.css:76-95` and `prefers-color-scheme: light` at `theme.css:135-156`.
- Theme toggle composable exists but no visible UI toggle was found in Settings.

---

## Copy & Honesty Evidence

### User-facing strings
- Most strings are externalized via `vue-i18n` in `en.json` / `zh-CN.json`.
- Hardcoded non-i18n strings found:
  - `ComposeView.vue:77` — `"Please select an account first"` (toast).
  - `ComposeEditor.vue:56-121` — toolbar labels "B", "I", "U", "S", "H1", "H2", "• List", "1. List", "\"", "↶", "↷".
  - `SettingsView.vue:211,219,227,344,352` — placeholders "sk-...", "https://api.example.com/v1", "deepseek-chat", "https://api.example.com/translate".
  - `TranslatePanel.vue:45-48` — "English", "简体中文", "日本語", "한국어".
  - `StatusBar.vue:88` — "v0.1.0".

### Missing i18n key
- `mail.yesterday` used in `MailList.vue:89` but **missing in both locale files**.

### Inflations / dark patterns
- **None found.** No marketing superlatives, forced continuity, hidden cost, fake scarcity, or confirmshaming.

### Jargon / unclear labels
- "Provider" in account form (`AccountForm.vue:109`) — could mean email or AI provider.
- "AI Providers" / "API Key" / "Base URL" / "Endpoint" in settings (`SettingsView.vue`).
- "IMAP Host", "IMAP Port", "SMTP Host", "SMTP Port" in account form.
- Raw provider kind identifiers shown to users: `azure_openai`, `custom_openai_compatible`, `moonshot`, `qwen`, etc. (`SettingsView.vue:200-201`).
- Raw account provider names: `Netease163`, `TencentExmail`, `Aliyun` (`AccountForm.vue:117`).

### Label→behavior mismatches
- Send button is disabled when subject is empty (`ComposeHeader.vue:79`) — many clients allow no-subject sends.
- Spam icon always `AlertTriangle` regardless of `isSpam` state (`MailViewer.vue:234`); label toggles but icon does not.
- "Maximize2" icon for "Reading Mode" and "Minimize2" for "Exit Reading Mode" are semantically reversed (reading mode collapses chrome, maximize usually expands).

### Error-message issues
- Many backend error strings are not actionable (e.g., "SMTP connection failed: {0}", "IMAP authentication failed for {0}", "An unknown error occurred").
- Hardcoded English toast breaks bilingual promise.

---

## Weight & Friction Evidence

### Initial JS bytes
- ~224 KB initial JS (`dist/assets/index-*.js` = 220 KB + 4 KB locale chunk).
- Total initial payload with CSS ≈ **258 KB**.

### Network request count for primary view
- ~7 requests: index.html, app JS, CSS, locale chunk, Tauri IPC handshake, `loadLocaleMessages`, `loadAccounts`.
- Fonts rely on system fallbacks; no external font files found in `dist/assets/`.

### Time-to-interactive (INFERRED)
- **~400–700 ms** in Tauri WebView (local file://, no network latency).

### Animation count on idle screen
- 3 distinct motion instances: toast transition group, command-palette transition, sync-status pulse.
- ~30+ hover/focus micro-transitions across components.
- **No `prefers-reduced-motion` support** anywhere in `src/`.

### Interruptive elements on initial load
- 4 types: `ToastContainer`, `CommandPalette`, `ContextMenu`, `MoveMailDialog`.
- Unread badges in sidebar are static, not interruptive.
- No auto-triggered toasts on first load unless an error occurs.

### Polling / auto-refresh
- No active polling loops on idle.
- Passive Tauri event listener for sync progress.
- Compose autosave debounce (2 s) and draft sync (5 s) only while composing.

---

## Accessibility Evidence

### Contrast (INFERRED)
- `text-disabled` on `bg-panel` fails WCAG AA in both themes.
- `text-muted/60` on `bg-background` fails WCAG AA.
- `text-warning` on dark panel is marginal (~4.0:1).

### Focus order
- Tab order is mostly logical for native controls.
- Several primary controls are **not focusable**:
  - Account list items in sidebar (`AppSidebar.vue:127-138`).
  - AI Assistant toggle in sidebar (`AppSidebar.vue:172-178`).
  - Mail list rows (`MailList.vue:206-215`) — keyboard navigation relies on J/K shortcuts.
  - Attachment download items (`MailViewer.vue:373-382`).
  - Command-palette result rows (`CommandPalette.vue:227-243`).

### Keyboard reachability of primary actions
| Action | Reachable | Notes |
|--------|-----------|-------|
| Compose | Yes | button + maybe shortcut (none found) |
| Send | Yes | focusable button |
| Search | Yes | `Ctrl/Cmd+K` |
| Select mail | Partial | J/K shortcuts; rows not Tab-focusable |
| Delete | Yes | `Delete`/`Backspace` shortcut + button |
| Archive | Partial | button focusable, no dedicated shortcut |
| Switch folder/account | Partial | folders focusable; accounts not |
| Open command palette | Yes | `Ctrl/Cmd+K` |
| Close modal/dialog | Partial | Escape works for context menu; delete/move dialogs lack Escape handlers |

### ARIA landmarks
- 6 landmarks: 1 `<main>` (`AppLayout.vue:45`), 1 `<aside>` (`AppSidebar.vue:81`), 1 `<nav>` (`AppSidebar.vue:97`), 3 `<section>` (`SettingsView.vue`).
- **No `role` attributes** anywhere in audited files.
- **No skip-link** found.

### Other accessibility gaps
- Color-only status indicator: unread dot (`MailList.vue:230-232`).
- Missing accessible names: mail-selection checkbox (`MailList.vue:218`), account select (`ComposeHeader.vue:4`), subject input (`ComposeHeader.vue:46`).
- No `aria-live` regions for toasts or sync status.
- No focus trapping in `MoveMailDialog.vue`, delete confirmation, or command palette.
- Mail list lacks `aria-setsize` / `aria-posinset` for virtual/scroll scenarios.

---

## Known Gaps Across All Evidence

1. No running dev server; all visual and contrast claims are INFERRED from source.
2. Tailwind config was not inspected; custom utilities (`text-h1`, `text-tiny`, exact shadows) may be defined there.
3. Some auxiliary components (`AiAssistantPanel`, `AiQuickActions`, `SandboxedHtml`, `TranslatePanel`, `LocaleSwitch`, `BulkActions`) were not deeply inspected.
4. Backend error strings and sync behavior are out of frontend scope.
5. No runtime accessibility audit (axe, Lighthouse) or performance trace was performed.
