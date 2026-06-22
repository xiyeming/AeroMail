# AeroMail Frontend UI Redesign — Design Spec

**Date:** 2026-06-22  
**Approach:** One-shot redesign + rewrite (Option B)  
**Scope:** Frontend UI + synchronized backend terminology  
**Visual direction:** Modern, light, fine-line partitioning  

---

## 1. Goals

- Make the UI understandable to a non-technical email user.
- Reduce visual chrome so content (email) dominates.
- Establish a single, systematic token system for type, spacing, and color.
- Remove duplicate affordances and empty wrapper views.
- Meet baseline accessibility: keyboard focus, ARIA landmarks, reduced-motion support, proper labels.
- Align backend error messages and API terminology with the new plain-language UI labels.

---

## 2. Non-Goals

- No change to backend protocol logic (IMAP/SMTP/SQLite/sync engine).
- No change to the AI/translation provider model beyond terminology and error copy.
- No change to the HTML-email sandbox security model.
- No mobile or responsive layout work beyond the existing desktop three-pane form.

---

## 3. Information Architecture

Keep the desktop three-pane mail-client form, but redistribute controls.

### 3.1 Sidebar (left)

- **Top**: app logo + "New message" button.
- **Middle**: folder navigation (Inbox, Starred, Sent, Drafts, Archived, Spam, Trash).
  - Each folder shows an icon + label + unread badge.
- **Bottom**: account switcher (compact popover), Settings link, Command Palette hint (`Ctrl+K`).

**Removed from sidebar**:
- Account list management (moved to Settings).
- AI Assistant toggle (moved to Reader toolbar / Command Palette).
- Language switcher (moved to Settings; Command Palette keeps it as a shortcut only).

### 3.2 Mail List (middle)

- **Header**: current folder title + search input + filter/select-all controls.
- **List**: scrollable mail rows.
  - Row content: sender avatar/initials, sender name, subject snippet, date, unread dot, attachment icon.
  - Hover reveals quick actions (star, archive).
- **Footer**: bulk-actions bar appears only when ≥1 mail selected.

### 3.3 Reader (right)

- **Toolbar**: Star, Archive, Spam/Not spam, Reply, Reply all, Forward, Translate, More (delete, move, print).
- **Header**: subject, sender, recipients, date.
- **Body**: sandboxed HTML / plain text.
- **Footer**: attachment chips.
- **AI Assistant**: collapsible side panel triggered from toolbar, not sidebar.

### 3.4 Compose

- Keep as a dedicated route or a large modal overlay.
- Header: account selector, send button, discard.
- Fields: To, Cc/Bcc toggle, Subject, rich-text editor, attachments.

### 3.5 Settings

- Single-page vertical sections:
  - General (language, theme).
  - Accounts (add/edit/delete email accounts).
  - Smart Assistants (AI providers).
  - Translation (standard + smart providers).

---

## 4. Unified Terminology (UI + Backend)

| UI Label | Backend Field / Error Key | Notes |
|---|---|---|
| Email service | `provider` (account) | Keep enum values; map to friendly names in UI |
| Label | `name` (account) | Rename UI label only |
| Email address | `email` | unchanged |
| Incoming mail server | `imap_host` | Error messages use "incoming mail server" |
| Incoming server port | `imap_port` | Error messages use "incoming server port" |
| Outgoing mail server | `smtp_host` | Error messages use "outgoing mail server" |
| Outgoing server port | `smtp_port` | Error messages use "outgoing server port" |
| Password / app password | `password` | UI placeholder: "Password or app password" |
| Smart Assistants | `ai_providers` | Table/field names unchanged; copy changes |
| Access key | `api_key` | Error messages use "access key" |
| Server address | `base_url` / `endpoint` | For both AI and translation providers |
| Model | `model` | unchanged |
| Standard translation | `traditional` provider kind | kind enum unchanged |
| Smart translation | `ai` provider kind | kind enum unchanged |
| Provider (translation) | `provider_kind` | UI maps to friendly names |

### Backend error-code copy changes

Update the following error messages in `src-tauri/src/error.rs` and `src/i18n/locales/*.json`:

- `IMAP_CONNECTION_TIMEOUT` → "Could not reach the incoming mail server. Check your network or server address."
- `IMAP_AUTH_FAILED` → "Sign-in failed for {account}. Check your email address and password or app password."
- `SMTP_SEND_FAILED` → "Could not send the message. Check your outgoing mail server settings."
- `SMTP_CONNECTION_FAILED` → "Could not connect to the outgoing mail server."
- `SMTP_AUTH_FAILED` → "Outgoing mail sign-in failed. Check your password or app password."
- `CONNECTION_TEST_FAILED` → "Account connection test failed: {reason}."
- `INVALID_ACCOUNT_CONFIG` → "Account settings are incomplete or incorrect."
- `AI_API_ERROR` / `AI_RATE_LIMITED` → use "Smart assistant" instead of "AI provider" in user-facing copy.
- `TRANSLATION_API_ERROR` → "Translation failed: {reason}."

---

## 5. Token System

All design tokens live in `src/styles/theme.css`. Tailwind utilities must map to these tokens; ad-hoc values are forbidden except for one-off illustration sizes.

### 5.1 Color

Reduce to a capped semantic palette.

```css
:root {
  /* Backgrounds */
  --bg-base: #0b0f14;
  --bg-elevated: #111820;
  --bg-overlay: rgba(0, 0, 0, 0.6);

  /* Text */
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-tertiary: #64748b;
  --text-disabled: #475569;

  /* Accent */
  --accent: #4d8dff;
  --accent-hover: #3a7de8;
  --accent-active: #2a6fd6;
  --accent-subtle: rgba(77, 141, 255, 0.12);

  /* Status */
  --success: #12b76a;
  --warning: #f79009;
  --danger: #f04438;
  --danger-subtle: rgba(240, 68, 56, 0.12);

  /* Borders */
  --border: rgba(148, 163, 184, 0.16);
  --border-strong: rgba(148, 163, 184, 0.24);

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.24);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.32);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.40);
}
```

Light mode tokens are defined in `[data-theme='light']` with the same semantic names.

**Rules**:
- No `text-yellow-500`, `text-red-500`, `bg-yellow-200`, etc. in components.
- Functional colors (star, spam, delete) use semantic tokens (`--warning`, `--danger`) or `--accent-subtle` backgrounds.

### 5.2 Type Scale

Define explicit type tokens:

```css
:root {
  --font-sans: 'Inter', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  --text-xs: 11px;
  --text-sm: 13px;
  --text-base: 14px;
  --text-md: 15px;
  --text-lg: 17px;
  --text-xl: 20px;
  --text-2xl: 24px;

  --font-regular: 400;
  --font-medium: 500;
  --font-semibold: 600;
}
```

Tailwind config maps `text-xs` → `--text-xs`, etc. Custom `text-h1` and `text-tiny` are removed.

### 5.3 Spacing Scale

Keep the existing token set but forbid ad-hoc fractional utilities:

```css
:root {
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-10: 40px;
  --space-12: 48px;
  --space-16: 64px;
}
```

**Rules**:
- No `py-1.5`, `px-1.5` in components; use `p-2`/`p-3` or add `--space-2-5: 10px` if 10 px is genuinely needed.
- No arbitrary widths like `w-[360px]`; define container tokens if needed.

### 5.4 Radius

```css
:root {
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-full: 9999px;
}
```

---

## 6. Component Rewrite List

Rewrite the following components from scratch, following the new architecture and token system.

### 6.1 Layout / Navigation

- `src/layouts/AppLayout.vue`
  - Three-pane grid with CSS Grid or flex.
  - Remove `BulkActions` from layout; render inside `MailList`.
  - Keep `AiAssistantPanel` as an optional right-side overlay, not a permanent column.
- `src/components/AppSidebar.vue`
  - New compact design: logo, New message, folders, account switcher, Settings.
  - Remove account list and AI Assistant toggle.
- `src/views/InboxView.vue`
  - Remove wrapper; route directly to `AppLayout` with `MailViewer` in the right pane.
- `src/views/AccountsView.vue`
  - Convert to Settings sub-section; remove standalone wrapper.

### 6.2 Mail Surfaces

- `src/components/MailList.vue`
  - Use semantic tokens.
  - Mail row is a focusable `<button>` or has `role="button" tabindex="0"`.
  - Checkbox has an accessible label (`aria-label="Select this message"`).
  - Hover quick actions: star, archive.
- `src/components/MailViewer.vue`
  - Toolbar: use icon + text labels or icon with accessible tooltip.
  - Spam icon changes based on state (`isSpam`).
  - Reading-mode icon uses `Expand` / `Compress` instead of `Maximize2` / `Minimize2`.
  - Delete confirmation: close on Escape, focus trap.
- `src/components/BulkActions.vue`
  - Inline at bottom of `MailList`, not global layout.
- `src/components/MoveMailDialog.vue`
  - Close on Escape, focus trap.
- `src/components/ContextMenu.vue`
  - Keep as right-click menu; ensure keyboard navigation and ARIA roles.

### 6.3 Compose

- `src/views/ComposeView.vue`
  - Fix hardcoded toast: replace with i18n key.
- `src/components/compose/ComposeHeader.vue`
  - Subject input has a visible `<label>` or `aria-label`.
  - Account select has a visible `<label>`.
  - Send button disabled state explained via tooltip.
- `src/components/compose/ComposeEditor.vue`
  - Toolbar labels use i18n keys (or short labels defined in locale files).
- `src/components/compose/RecipientInput.vue`
  - Keep behavior; ensure focus ring uses token.
- `src/components/compose/ComposeAttachmentList.vue`
  - Remove button has accessible label.

### 6.4 Settings / Accounts

- `src/views/SettingsView.vue`
  - Use plain-language labels from the terminology table.
  - Provider type/kind maps to friendly names.
  - Form inputs always have labels.
- `src/components/AccountForm.vue`
  - Rename labels: "Email service", "Label", "Incoming mail server", etc.
  - Provider names map to friendly labels.
- `src/components/AccountList.vue`
  - Inline into Settings; remove standalone view.

### 6.5 Global Chrome

- `src/components/StatusBar.vue`
  - Remove language switcher (moved to Settings).
  - Show only sync status, unread count, last sync, online status, version.
- `src/components/CommandPalette.vue`
  - Keep language command as shortcut only.
  - Result rows are focusable; close on Escape.
- `src/components/ToastContainer.vue`
  - Add `aria-live="polite"` region.
- `src/components/LocaleSwitch.vue`
  - Move into Settings.

### 6.6 AI / Translation

- `src/components/AiAssistantPanel.vue`
  - Triggered from Reader toolbar, not sidebar.
  - Use plain-language labels.
- `src/components/AiQuickActions.vue`
  - Keep as toolbar popover; labels i18n-ized.
- `src/components/TranslatePanel.vue`
  - Language options use i18n keys, not hardcoded strings.

---

## 7. Accessibility Requirements

- **Focus order**: Sidebar → Mail list → Reader → Compose/Settings. All primary controls reachable via Tab.
- **Focusable elements**: Mail rows, sidebar account switcher, command palette results, attachment chips must be focusable.
- **ARIA landmarks**: One `<main>`, one `<nav>` for sidebar, `<search>` for search, `<aside>` for AI panel.
- **Skip link**: Add "Skip to mail list" / "Skip to reader" links.
- **Reduced motion**: Respect `prefers-reduced-motion` for all transitions, toasts, and spinners.
- **Contrast**: All text must meet WCAG AA (4.5:1 for normal text). Disabled text is exempt but must still be legible.
- **Modals**: Focus trap, Escape to close, return focus to trigger.
- **Live regions**: Toasts and sync status use `aria-live`.

---

## 8. Backend Copy Alignment

Files to update:
- `src-tauri/src/error.rs` — update `to_payload()` strings for the error codes listed in §4.
- `src-tauri/src/models/account.rs` — no field renames; only doc comments.
- `src-tauri/src/commands/account.rs` — update validation error copy if emitted directly.

Frontend files to update:
- `src/i18n/locales/en.json`
- `src/i18n/locales/zh-CN.json`
- `src/types/account.ts`, `src/types/ai.ts`, `src/types/translation.ts` — only comments/type labels if exposed.

---

## 9. Verification Checklist

- [ ] `pnpm type-check` passes.
- [ ] `pnpm i18n:check` passes (all keys present in both locales).
- [ ] `cargo clippy` passes with no new warnings.
- [ ] App launches (`cargo tauri dev`) and primary flows work: read, triage, compose, send, add account, configure provider.
- [ ] Keyboard-only smoke test passes: Tab through all primary controls, use J/K in mail list, Escape closes modals/palette.
- [ ] No ad-hoc Tailwind colors/spacing remain in rewritten components (grep for `yellow-500`, `red-500`, `py-1.5`, `w-[` in `src/components` and `src/views`).
- [ ] No hardcoded user-facing strings remain in rewritten components.

---

## 10. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| One-shot rewrite breaks existing flows | Keep IPC contracts and stores unchanged; verify checklist before declaring done. |
| Chinese locale becomes inconsistent | Update `zh-CN.json` in parallel with every `en.json` change; run `pnpm i18n:check`. |
| Backend copy changes break frontend error mapping | Keep error codes stable; only change human-readable `args`/`message` payloads. |
| Heavy borders removed make boundaries unclear | Use background elevation + fine borders + generous whitespace. |
| Modern style feels too different from current app | Provide a short onboarding / changelog note; no feature flag needed for single-user app. |

---

## 11. Decisions

1. **Compose surface**: Full-page route.
2. **AI Assistant panel**: Slides in from the right as a fixed-width side panel over the Reader.
3. **Theme toggle**: Explicit Dark / Light / Auto selector in Settings → General.
