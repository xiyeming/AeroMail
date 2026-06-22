# AeroMail Frontend UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign and rewrite the AeroMail frontend UI to a modern, light, fine-line style; align backend terminology and error copy; remove duplicate affordances; establish a systematic token system; and meet baseline accessibility.

**Architecture:** Keep the existing Tauri + Vue 3 + Pinia stack. Replace component markup and styles layer-by-layer while preserving IPC contracts and stores. Introduce a capped semantic token system in `theme.css`, map it through Tailwind, and synchronize backend error messages with the new plain-language UI labels.

**Tech Stack:** Vue 3 Composition API, TypeScript, Tailwind CSS v4, Pinia, vue-i18n v10, Vite, Tauri v2, Rust 2024.

## Global Constraints

- All user-facing strings must exist in both `src/i18n/locales/en.json` and `src/i18n/locales/zh-CN.json`.
- No ad-hoc Tailwind colors (`yellow-500`, `red-500`, etc.) or arbitrary widths (`w-[...]`) in rewritten components.
- No `py-1.5` / `px-1.5` fractional utilities in rewritten components.
- Backend field names stay stable; only user-facing copy and error messages change.
- `pnpm type-check` and `pnpm i18n:check` must pass after every task that touches front-end types or locales.
- `cargo clippy` must pass after every task that touches Rust error copy.
- Accessibility: all interactive elements focusable, modals trap focus and close on Escape, `prefers-reduced-motion` respected.

---

## File Structure

| Path | Responsibility |
|---|---|
| `src/styles/theme.css` | Single source of truth for color, type, spacing, radius, shadow tokens. |
| `tailwind.config.ts` / `postcss.config.js` | Maps Tailwind utilities to the custom tokens in `theme.css`. |
| `src/i18n/locales/en.json` | English source of truth for all UI labels, placeholders, tooltips, errors. |
| `src/i18n/locales/zh-CN.json` | Simplified Chinese mirror of `en.json`. |
| `src-tauri/src/error.rs` | Rust `AeroError::to_payload()` user-facing copy aligned with new UI terms. |
| `src/layouts/AppLayout.vue` | Three-pane grid; hosts sidebar, mail list, reader. |
| `src/components/AppSidebar.vue` | Compact navigation: logo, new message, folders, account switcher, settings. |
| `src/components/MailList.vue` | Scrollable mail rows with hover quick actions and bulk bar. |
| `src/components/MailViewer.vue` | Reader toolbar, header, sandboxed body, attachments. |
| `src/views/ComposeView.vue` | Full-page compose shell. |
| `src/components/compose/ComposeHeader.vue` | Account, recipients, subject, send. |
| `src/components/compose/ComposeEditor.vue` | Rich-text editor toolbar + body. |
| `src/views/SettingsView.vue` | Settings page with General, Accounts, Smart Assistants, Translation sections. |
| `src/components/AccountForm.vue` | Add/edit email account form with plain-language labels. |
| `src/components/AiAssistantPanel.vue` | Right-sliding panel triggered from Reader toolbar. |
| `src/components/AiQuickActions.vue` | Toolbar popover for smart-assistant quick actions. |
| `src/components/TranslatePanel.vue` | Translation language selector using i18n keys. |
| `src/components/StatusBar.vue` | Sync/unread/last-sync/online/version strip. |
| `src/components/CommandPalette.vue` | Global command search; language command remains as shortcut. |
| `src/components/ToastContainer.vue` | Toasts with `aria-live="polite"`. |
| `src/components/ContextMenu.vue` | Right-click menu with ARIA roles. |
| `src/components/MoveMailDialog.vue` | Folder move dialog with focus trap. |
| `src/views/InboxView.vue` | To be deleted; route directly through `AppLayout`. |
| `src/views/AccountsView.vue` | To be deleted; inlined into Settings. |
| `src/components/AccountList.vue` | To be deleted; inlined into Settings. |
| `src/components/LocaleSwitch.vue` | To be deleted; inlined into Settings General section. |
| `src/components/BulkActions.vue` | To be deleted; inlined into `MailList`. |

---

## Task 1: Replace token system in `src/styles/theme.css`

**Files:**
- Modify: `src/styles/theme.css`

**Interfaces:**
- Produces: CSS custom properties consumed by Tailwind and component classes.

- [ ] **Step 1: Back up current theme file**

```bash
cp src/styles/theme.css src/styles/theme.css.bak
```

- [ ] **Step 2: Replace dark/light token blocks with the capped semantic palette**

Replace the entire content of `src/styles/theme.css` with:

```css
:root {
  --font-sans: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, monospace;

  /* Type scale */
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

  /* Spacing */
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

  /* Radius */
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;
  --radius-full: 9999px;
}

:root,
[data-theme='dark'] {
  color-scheme: dark;

  --bg-base: #0b0f14;
  --bg-elevated: #111820;
  --bg-raised: #1a2230;
  --bg-overlay: rgba(0, 0, 0, 0.6);

  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-tertiary: #64748b;
  --text-disabled: #475569;

  --accent: #4d8dff;
  --accent-hover: #3a7de8;
  --accent-active: #2a6fd6;
  --accent-subtle: rgba(77, 141, 255, 0.14);

  --success: #12b76a;
  --warning: #f79009;
  --danger: #f04438;
  --danger-subtle: rgba(240, 68, 56, 0.14);

  --border: rgba(148, 163, 184, 0.14);
  --border-strong: rgba(148, 163, 184, 0.22);

  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.24);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.32);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.44);
}

[data-theme='light'] {
  color-scheme: light;

  --bg-base: #ffffff;
  --bg-elevated: #f8fafc;
  --bg-raised: #f1f5f9;
  --bg-overlay: rgba(15, 23, 42, 0.44);

  --text-primary: #0f172a;
  --text-secondary: #475569;
  --text-tertiary: #64748b;
  --text-disabled: #94a3b8;

  --accent: #2563eb;
  --accent-hover: #1d4ed8;
  --accent-active: #1e40af;
  --accent-subtle: rgba(37, 99, 235, 0.10);

  --success: #12b76a;
  --warning: #f79009;
  --danger: #ef4444;
  --danger-subtle: rgba(239, 68, 68, 0.10);

  --border: rgba(100, 116, 139, 0.18);
  --border-strong: rgba(100, 116, 139, 0.28);

  --shadow-sm: 0 1px 2px rgba(15, 23, 42, 0.06);
  --shadow-md: 0 4px 12px rgba(15, 23, 42, 0.10);
  --shadow-lg: 0 12px 32px rgba(15, 23, 42, 0.16);
}

@media (prefers-color-scheme: light) {
  :root:not([data-theme='dark']) {
    color-scheme: light;

    --bg-base: #ffffff;
    --bg-elevated: #f8fafc;
    --bg-raised: #f1f5f9;
    --bg-overlay: rgba(15, 23, 42, 0.44);

    --text-primary: #0f172a;
    --text-secondary: #475569;
    --text-tertiary: #64748b;
    --text-disabled: #94a3b8;

    --accent: #2563eb;
    --accent-hover: #1d4ed8;
    --accent-active: #1e40af;
    --accent-subtle: rgba(37, 99, 235, 0.10);

    --success: #12b76a;
    --warning: #f79009;
    --danger: #ef4444;
    --danger-subtle: rgba(239, 68, 68, 0.10);

    --border: rgba(100, 116, 139, 0.18);
    --border-strong: rgba(100, 116, 139, 0.28);

    --shadow-sm: 0 1px 2px rgba(15, 23, 42, 0.06);
    --shadow-md: 0 4px 12px rgba(15, 23, 42, 0.10);
    --shadow-lg: 0 12px 32px rgba(15, 23, 42, 0.16);
  }
}

/* Base styles */
html, body, #app {
  height: 100%;
  background-color: var(--bg-base);
  color: var(--text-primary);
  font-family: var(--font-sans);
  font-size: var(--text-base);
}

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

- [ ] **Step 3: Verify Vite can parse the CSS**

Run:

```bash
pnpm type-check
```

Expected: no errors related to `theme.css`.

- [ ] **Step 4: Commit**

```bash
git add src/styles/theme.css
git commit -m "feat(ui): replace design token system with capped semantic palette"
```

---

## Task 2: Wire Tailwind utilities to tokens

**Files:**
- Modify: `tailwind.config.ts` (or create if absent; check `package.json` / repo root)
- Modify: `src/styles/theme.css` if additional utilities are needed

**Interfaces:**
- Consumes: CSS custom properties from `theme.css`.
- Produces: Tailwind utilities `text-xs`, `text-sm`, `bg-base`, `text-secondary`, `border`, etc.

- [ ] **Step 1: Locate the Tailwind config file**

Run:

```bash
ls -la *.config.* tailwind.config.* postcss.config.* 2>/dev/null || true
```

- If `tailwind.config.ts` exists, modify it.
- If Tailwind v4 is configured via CSS-only (`@theme` in CSS), add the mapping in `theme.css` instead.

- [ ] **Step 2a: If using `tailwind.config.ts`, extend the theme**

```typescript
import type { Config } from 'tailwindcss'

const config: Config = {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['var(--font-sans)'],
        mono: ['var(--font-mono)'],
      },
      fontSize: {
        xs: ['var(--text-xs)', { lineHeight: '1.25' }],
        sm: ['var(--text-sm)', { lineHeight: '1.35' }],
        base: ['var(--text-base)', { lineHeight: '1.5' }],
        md: ['var(--text-md)', { lineHeight: '1.5' }],
        lg: ['var(--text-lg)', { lineHeight: '1.35' }],
        xl: ['var(--text-xl)', { lineHeight: '1.3' }],
        '2xl': ['var(--text-2xl)', { lineHeight: '1.25' }],
      },
      spacing: {
        1: 'var(--space-1)',
        2: 'var(--space-2)',
        3: 'var(--space-3)',
        4: 'var(--space-4)',
        5: 'var(--space-5)',
        6: 'var(--space-6)',
        8: 'var(--space-8)',
        10: 'var(--space-10)',
        12: 'var(--space-12)',
        16: 'var(--space-16)',
      },
      colors: {
        base: 'var(--bg-base)',
        elevated: 'var(--bg-elevated)',
        raised: 'var(--bg-raised)',
        overlay: 'var(--bg-overlay)',
        primary: 'var(--text-primary)',
        secondary: 'var(--text-secondary)',
        tertiary: 'var(--text-tertiary)',
        disabled: 'var(--text-disabled)',
        accent: {
          DEFAULT: 'var(--accent)',
          hover: 'var(--accent-hover)',
          active: 'var(--accent-active)',
          subtle: 'var(--accent-subtle)',
        },
        success: 'var(--success)',
        warning: 'var(--warning)',
        danger: {
          DEFAULT: 'var(--danger)',
          subtle: 'var(--danger-subtle)',
        },
        border: {
          DEFAULT: 'var(--border)',
          strong: 'var(--border-strong)',
        },
      },
      borderRadius: {
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        xl: 'var(--radius-xl)',
        full: 'var(--radius-full)',
      },
      boxShadow: {
        sm: 'var(--shadow-sm)',
        md: 'var(--shadow-md)',
        lg: 'var(--shadow-lg)',
      },
    },
  },
  plugins: [],
}

export default config
```

- [ ] **Step 2b: If using Tailwind v4 CSS-only config, add `@theme` block to `theme.css`**

Append to `src/styles/theme.css`:

```css
@theme {
  --font-sans: var(--font-sans);
  --font-mono: var(--font-mono);

  --text-xs: var(--text-xs);
  --text-sm: var(--text-sm);
  --text-base: var(--text-base);
  --text-md: var(--text-md);
  --text-lg: var(--text-lg);
  --text-xl: var(--text-xl);
  --text-2xl: var(--text-2xl);

  --spacing-1: var(--space-1);
  --spacing-2: var(--space-2);
  --spacing-3: var(--space-3);
  --spacing-4: var(--space-4);
  --spacing-5: var(--space-5);
  --spacing-6: var(--space-6);
  --spacing-8: var(--space-8);
  --spacing-10: var(--space-10);
  --spacing-12: var(--space-12);
  --spacing-16: var(--space-16);

  --color-base: var(--bg-base);
  --color-elevated: var(--bg-elevated);
  --color-raised: var(--bg-raised);
  --color-overlay: var(--bg-overlay);
  --color-primary: var(--text-primary);
  --color-secondary: var(--text-secondary);
  --color-tertiary: var(--text-tertiary);
  --color-disabled: var(--text-disabled);
  --color-accent: var(--accent);
  --color-accent-hover: var(--accent-hover);
  --color-accent-active: var(--accent-active);
  --color-accent-subtle: var(--accent-subtle);
  --color-success: var(--success);
  --color-warning: var(--warning);
  --color-danger: var(--danger);
  --color-danger-subtle: var(--danger-subtle);
  --color-border: var(--border);
  --color-border-strong: var(--border-strong);

  --radius-sm: var(--radius-sm);
  --radius-md: var(--radius-md);
  --radius-lg: var(--radius-lg);
  --radius-xl: var(--radius-xl);
  --radius-full: var(--radius-full);

  --shadow-sm: var(--shadow-sm);
  --shadow-md: var(--shadow-md);
  --shadow-lg: var(--shadow-lg);
}
```

- [ ] **Step 3: Verify build**

Run:

```bash
pnpm type-check
```

Expected: pass.

- [ ] **Step 4: Commit**

```bash
git add tailwind.config.ts src/styles/theme.css
git commit -m "feat(ui): wire tailwind utilities to semantic tokens"
```

---

## Task 3: Update locale files with new plain-language keys

**Files:**
- Modify: `src/i18n/locales/en.json`
- Modify: `src/i18n/locales/zh-CN.json`

**Interfaces:**
- Produces: i18n keys used by rewritten components and backend error mapping.

- [ ] **Step 1: Add new keys to `en.json`**

Insert or update the following keys (merge into existing structure without deleting unrelated keys):

```json
{
  "app": {
    "name": "AeroMail"
  },
  "nav": {
    "accounts": "Accounts",
    "settings": "Settings",
    "inbox": "Inbox",
    "starred": "Starred",
    "sent": "Sent",
    "drafts": "Drafts",
    "archived": "Archived",
    "spam": "Spam",
    "trash": "Trash"
  },
  "mail": {
    "newMail": "New message",
    "loading": "Loading...",
    "noEmails": "No messages",
    "selectEmail": "Select a message to read",
    "selectEmailHint": "Use keyboard shortcuts to navigate",
    "navigate": "Navigate",
    "open": "Open",
    "close": "Close",
    "star": "Star",
    "unstar": "Unstar",
    "archive": "Archive",
    "markAsSpam": "Mark as spam",
    "notSpam": "Not spam",
    "reply": "Reply",
    "replyAll": "Reply all",
    "forward": "Forward",
    "readingMode": "Reading mode",
    "exitReadingMode": "Exit reading mode",
    "delete": "Delete",
    "translate": "Translate",
    "translatedTo": "Translated to {lang}",
    "showOriginal": "Show original",
    "unknownSender": "Unknown sender",
    "noSubject": "(No subject)",
    "hasAttachments": "Has attachments",
    "noContent": "No content",
    "attachments": "Attachments",
    "deleteConfirmTitle": "Delete message",
    "deleteConfirmMessage": "Are you sure you want to delete this message? This action cannot be undone.",
    "moveTo": "Move to...",
    "selectFolder": "Select a folder",
    "markAsRead": "Mark as read",
    "markAsUnread": "Mark as unread",
    "deselectAll": "Deselect all",
    "selectedCount": "{count} selected",
    "to": "To",
    "cc": "Cc",
    "bcc": "Bcc",
    "yesterday": "Yesterday"
  },
  "compose": {
    "addAttachment": "Add attachment",
    "saving": "Saving...",
    "saved": "Saved",
    "selectAccount": "Select account",
    "send": "Send",
    "to": "To",
    "toPlaceholder": "Enter recipient emails",
    "cc": "Cc",
    "bcc": "Bcc",
    "subject": "Subject",
    "subjectPlaceholder": "Subject (optional)",
    "editorPlaceholder": "Write something...",
    "noAccountSelected": "Please select an account first",
    "bold": "Bold",
    "italic": "Italic",
    "underline": "Underline",
    "strike": "Strikethrough",
    "heading1": "Heading 1",
    "heading2": "Heading 2",
    "bulletList": "Bullet list",
    "orderedList": "Numbered list",
    "quote": "Quote",
    "undo": "Undo",
    "redo": "Redo"
  },
  "account": {
    "addAccount": "Add account",
    "emailService": "Email service",
    "label": "Label",
    "namePlaceholder": "Work Gmail",
    "emailAddress": "Email address",
    "emailPlaceholder": "your.email@example.com",
    "incomingServer": "Incoming mail server",
    "incomingPort": "Incoming server port",
    "outgoingServer": "Outgoing mail server",
    "outgoingPort": "Outgoing server port",
    "password": "Password or app password",
    "passwordPlaceholder": "Password",
    "delete": "Delete",
    "noAccounts": "No accounts yet",
    "errors": {
      "nameRequired": "Label is required",
      "emailRequired": "Email address is required",
      "imapRequired": "Incoming mail server and port are required",
      "smtpRequired": "Outgoing mail server and port are required",
      "passwordRequired": "Password is required"
    }
  },
  "settings": {
    "title": "Settings",
    "language": "Language",
    "theme": "Theme",
    "themeDark": "Dark",
    "themeLight": "Light",
    "themeAuto": "Auto",
    "aiProviders": "Smart Assistants",
    "addProvider": "Add smart assistant",
    "noProviders": "No smart assistants configured",
    "providerType": "Provider",
    "accessKey": "Access key",
    "accessKeyPlaceholder": "sk-...",
    "serverAddress": "Server address (optional)",
    "serverAddressPlaceholder": "https://api.example.com/v1",
    "model": "Model",
    "modelPlaceholder": "deepseek-chat",
    "saveProvider": "Save provider",
    "translationProviders": "Translation",
    "addTranslationProvider": "Add provider",
    "noTranslationProviders": "No translation providers configured",
    "providerKind": "Provider",
    "endpoint": "Server address (optional)",
    "endpointPlaceholder": "https://api.example.com/translate",
    "selectAiProvider": "Select smart assistant",
    "traditionalProvider": "Standard translation",
    "aiTranslationProvider": "Smart translation"
  },
  "aiAssistant": {
    "title": "Smart Assistant",
    "selectProvider": "Select a provider to start",
    "inputPlaceholder": "Ask about the current message...",
    "thinking": "Thinking...",
    "send": "Send"
  },
  "aiActions": {
    "summarize": "Summarize",
    "generateReply": "Reply",
    "extractTodos": "To-dos"
  },
  "translation": {
    "translate": "Translate",
    "translating": "Translating...",
    "language": {
      "english": "English",
      "chinese": "简体中文",
      "japanese": "日本語",
      "korean": "한국어"
    }
  },
  "statusBar": {
    "syncing": "Syncing...",
    "syncComplete": "Up to date",
    "unread": "{count} unread",
    "lastSync": "Last sync: {time}",
    "never": "Never",
    "online": "Online",
    "offline": "Offline",
    "version": "Version {version}"
  },
  "commandPalette": {
    "placeholder": "Search mail...",
    "switchToEnglish": "Switch to English",
    "switchToChinese": "Switch to 简体中文",
    "openAiAssistant": "Open Smart Assistant",
    "mail": "Mail",
    "history": "History",
    "noResults": "No results found"
  },
  "contextMenu": {
    "open": "Open",
    "star": "Star",
    "unstar": "Unstar",
    "archive": "Archive",
    "spam": "Mark as spam",
    "notSpam": "Not spam",
    "markRead": "Mark as read",
    "markUnread": "Mark as unread",
    "moveTo": "Move to...",
    "delete": "Delete"
  },
  "common": {
    "cancel": "Cancel",
    "save": "Save",
    "loading": "Loading...",
    "search": "Search",
    "settings": "Settings",
    "close": "Close"
  },
  "errors": {
    "SMTP_CONNECTION_FAILED": "Could not connect to the outgoing mail server.",
    "SMTP_AUTH_FAILED": "Outgoing mail sign-in failed. Check your password or app password.",
    "SMTP_SEND_FAILED": "Could not send the message. Check your outgoing mail server settings.",
    "INVALID_RECIPIENT": "Invalid recipient: {0}",
    "DRAFT_NOT_FOUND": "Draft not found: {0}",
    "ATTACHMENT_NOT_FOUND": "Attachment not found: {0}",
    "INVALID_ATTACHMENT": "Invalid attachment: {0}",
    "MAIL_BUILDER_FAILED": "Failed to build message: {0}",
    "IMAP_APPEND_FAILED": "Failed to sync draft: {0}",
    "OAUTH2_REFRESH_FAILED": "OAuth2 token refresh failed: {0}",
    "OAUTH2_CONFIG_INCOMPLETE": "OAuth2 configuration is incomplete",
    "MAIL_NOT_FOUND": "Message not found: {0}",
    "IMAP_AUTH_FAILED": "Sign-in failed for {0}. Check your email address and password or app password.",
    "IMAP_CONNECTION_TIMEOUT": "Could not reach the incoming mail server. Check your network or server address.",
    "SMTP_SEND_FAILED_DUP": "Could not send the message: {0}",
    "INVALID_ACCOUNT_CONFIG": "Account settings are incomplete or incorrect.",
    "DATABASE_ERROR": "Database error",
    "INTERNAL_ERROR": "Internal error: {0}",
    "UNKNOWN_ERROR": "An unknown error occurred",
    "ACCOUNT_NOT_FOUND": "Account not found: {0}",
    "CONNECTION_TEST_FAILED": "Account connection test failed: {0}",
    "AI_PROVIDER_NOT_FOUND": "Smart assistant not found",
    "AI_API_ERROR": "Smart assistant request failed: {0}",
    "AI_RATE_LIMITED": "Rate limited, please try again later",
    "AI_CONTEXT_MAIL_NOT_FOUND": "Context message not found",
    "TRANSLATION_PROVIDER_NOT_FOUND": "Translation provider not found",
    "TRANSLATION_API_ERROR": "Translation failed: {0}",
    "TRANSLATION_NO_TEXT": "No text to translate"
  }
}
```

- [ ] **Step 2: Update `zh-CN.json` with equivalent translations**

Ensure every key in `en.json` exists in `zh-CN.json`. Example mappings (translate remaining keys consistently):

```json
{
  "nav": {
    "inbox": "收件箱",
    "starred": "星标",
    "sent": "已发送",
    "drafts": "草稿",
    "archived": "归档",
    "spam": "垃圾邮件",
    "trash": "废纸篓"
  },
  "mail": {
    "newMail": "新建邮件",
    "selectEmail": "选择一封邮件阅读",
    "deleteConfirmTitle": "删除邮件",
    "deleteConfirmMessage": "确定要删除这封邮件吗？此操作无法撤销。",
    "yesterday": "昨天"
  },
  "account": {
    "emailService": "邮件服务",
    "label": "标签",
    "incomingServer": "收件服务器",
    "incomingPort": "收件服务器端口",
    "outgoingServer": "发件服务器",
    "outgoingPort": "发件服务器端口",
    "password": "密码或应用专用密码"
  },
  "settings": {
    "aiProviders": "智能助手",
    "addProvider": "添加智能助手",
    "noProviders": "未配置智能助手",
    "accessKey": "访问密钥",
    "serverAddress": "服务器地址（可选）",
    "traditionalProvider": "标准翻译",
    "aiTranslationProvider": "智能翻译"
  },
  "aiAssistant": {
    "title": "智能助手"
  },
  "commandPalette": {
    "openAiAssistant": "打开智能助手"
  },
  "errors": {
    "SMTP_AUTH_FAILED": "发件服务器登录失败。请检查密码或应用专用密码。",
    "SMTP_SEND_FAILED": "无法发送邮件。请检查发件服务器设置。",
    "IMAP_AUTH_FAILED": "{0} 登录失败。请检查邮箱地址和密码或应用专用密码。",
    "IMAP_CONNECTION_TIMEOUT": "无法连接到收件服务器。请检查网络或服务器地址。",
    "AI_PROVIDER_NOT_FOUND": "智能助手不存在",
    "AI_API_ERROR": "智能助手请求失败：{0}",
    "AI_CONTEXT_MAIL_NOT_FOUND": "上下文邮件不存在"
  }
}
```

- [ ] **Step 3: Run i18n consistency check**

```bash
pnpm i18n:check
```

Expected: pass (key sets match).

- [ ] **Step 4: Commit**

```bash
git add src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(i18n): add plain-language labels for redesign and aligned error copy"
```

---

## Task 4: Align backend error copy

**Files:**
- Modify: `src-tauri/src/error.rs`

**Interfaces:**
- Consumes: New i18n keys from `en.json` / `zh-CN.json`.
- Produces: `ErrorPayload` objects with updated `args` strings.

- [ ] **Step 1: Locate `to_payload()` in `src-tauri/src/error.rs`**

Read the file and find the `to_payload` method.

- [ ] **Step 2: Update user-facing copy for the listed error variants**

Change only the human-readable strings returned by `to_payload()` (not error codes or field names). For example:

```rust
AeroError::ImapConnectionTimeout => ErrorPayload {
    code: "IMAP_CONNECTION_TIMEOUT",
    args: vec!["Could not reach the incoming mail server. Check your network or server address.".to_string()],
},
AeroError::ImapAuthFailed(account) => ErrorPayload {
    code: "IMAP_AUTH_FAILED",
    args: vec![format!("Sign-in failed for {account}. Check your email address and password or app password.")],
},
AeroError::SmtpConnectionFailed => ErrorPayload {
    code: "SMTP_CONNECTION_FAILED",
    args: vec!["Could not connect to the outgoing mail server.".to_string()],
},
AeroError::SmtpAuthFailed => ErrorPayload {
    code: "SMTP_AUTH_FAILED",
    args: vec!["Outgoing mail sign-in failed. Check your password or app password.".to_string()],
},
AeroError::SmtpSendFailed(msg) => ErrorPayload {
    code: "SMTP_SEND_FAILED",
    args: vec![format!("Could not send the message. Check your outgoing mail server settings. {msg}")],
},
AeroError::InvalidAccountConfig(msg) => ErrorPayload {
    code: "INVALID_ACCOUNT_CONFIG",
    args: vec![format!("Account settings are incomplete or incorrect. {msg}")],
},
AeroError::ConnectionTestFailed(msg) => ErrorPayload {
    code: "CONNECTION_TEST_FAILED",
    args: vec![format!("Account connection test failed: {msg}")],
},
AeroError::AiProviderNotFound => ErrorPayload {
    code: "AI_PROVIDER_NOT_FOUND",
    args: vec!["Smart assistant not found".to_string()],
},
AeroError::AiApiError(msg) => ErrorPayload {
    code: "AI_API_ERROR",
    args: vec![format!("Smart assistant request failed: {msg}")],
},
AeroError::AiContextMailNotFound => ErrorPayload {
    code: "AI_CONTEXT_MAIL_NOT_FOUND",
    args: vec!["Context message not found".to_string()],
},
```

- [ ] **Step 3: Run clippy**

```bash
cargo clippy
```

Expected: no new warnings.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/error.rs
git commit -m "feat(error): align user-facing error copy with plain-language UI"
```

---

## Task 5: Rewrite `src/layouts/AppLayout.vue`

**Files:**
- Modify: `src/layouts/AppLayout.vue`
- Delete usage of: `src/components/BulkActions.vue` (move inline in Task 9)

**Interfaces:**
- Consumes: `AppSidebar`, `MailList`, `MailViewer`, `AiAssistantPanel`, `StatusBar`, `ToastContainer`, `CommandPalette`.
- Produces: Three-pane layout with named slots/areas.

- [ ] **Step 1: Replace `AppLayout.vue` with new markup**

```vue
<template>
  <div class="flex h-screen w-screen overflow-hidden bg-base text-primary">
    <AppSidebar class="shrink-0 w-56 border-r border-border" />

    <div class="flex flex-col flex-1 min-w-0">
      <div class="flex flex-1 min-h-0">
        <MailList class="w-80 shrink-0 border-r border-border" />
        <main class="flex-1 min-w-0 flex flex-col">
          <MailViewer class="flex-1 min-h-0" />
        </main>
      </div>

      <StatusBar class="shrink-0 h-8 border-t border-border" />
    </div>

    <AiAssistantPanel />
    <ToastContainer />
    <CommandPalette />
  </div>
</template>

<script setup lang="ts">
import AppSidebar from '@/components/AppSidebar.vue'
import MailList from '@/components/MailList.vue'
import MailViewer from '@/components/MailViewer.vue'
import StatusBar from '@/components/StatusBar.vue'
import AiAssistantPanel from '@/components/AiAssistantPanel.vue'
import ToastContainer from '@/components/ToastContainer.vue'
import CommandPalette from '@/components/CommandPalette.vue'
</script>
```

- [ ] **Step 2: Type-check**

```bash
pnpm type-check
```

Expected: pass (components may still reference old props; fix only obvious type errors in this file).

- [ ] **Step 3: Commit**

```bash
git add src/layouts/AppLayout.vue
git commit -m "feat(ui): rewrite AppLayout with three-pane grid and token classes"
```

---

## Task 6: Rewrite `src/components/AppSidebar.vue`

**Files:**
- Modify: `src/components/AppSidebar.vue`

**Interfaces:**
- Consumes: `accountStore`, `mailStore`, `useRouter()`, `useI18n()`.
- Produces: Navigation with folders, account switcher, new message, settings.

- [ ] **Step 1: Rewrite component**

Key requirements:
- Use semantic tokens (`text-secondary`, `hover:bg-elevated`, `border-border`).
- Folder nav uses `RouterLink` with icon + label + unread badge.
- Account switcher is a native `button` that opens a popover (can be a simple details/summary or a custom popover; ensure focusable and keyboard operable).
- Remove AI Assistant toggle and language switcher.
- "New message" button at top.
- Settings link at bottom.

- [ ] **Step 2: Type-check**

```bash
pnpm type-check
```

- [ ] **Step 3: Commit**

```bash
git add src/components/AppSidebar.vue
git commit -m "feat(ui): rewrite AppSidebar with compact nav and account switcher"
```

---

## Task 7: Inline `LocaleSwitch` into Settings General section

**Files:**
- Modify: `src/views/SettingsView.vue`
- Delete: `src/components/LocaleSwitch.vue`

**Interfaces:**
- Consumes: `useLocale()` composable.
- Produces: Language selector in Settings.

- [ ] **Step 1: Add General section to `SettingsView.vue`**

```vue
<section aria-labelledby="general-heading" class="space-y-4">
  <h2 id="general-heading" class="text-lg font-semibold">{{ $t('settings.language') }}</h2>
  <div class="flex items-center gap-3">
    <label for="locale-select" class="text-secondary">{{ $t('settings.language') }}</label>
    <select id="locale-select" v-model="currentLocale" class="bg-elevated border border-border rounded-md px-3 py-2">
      <option v-for="loc in locales" :key="loc" :value="loc">{{ labels[loc] }}</option>
    </select>
  </div>
</section>
```

- [ ] **Step 2: Remove `LocaleSwitch.vue` import from any file**

```bash
grep -rn "LocaleSwitch" src/
```

Remove imports and usages.

- [ ] **Step 3: Delete file**

```bash
rm src/components/LocaleSwitch.vue
```

- [ ] **Step 4: Type-check and i18n-check**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 5: Commit**

```bash
git add src/views/SettingsView.vue src/components/LocaleSwitch.vue
git commit -m "refactor(ui): move language switcher into Settings and remove LocaleSwitch"
```

---

## Task 8: Simplify `src/components/StatusBar.vue`

**Files:**
- Modify: `src/components/StatusBar.vue`

**Interfaces:**
- Consumes: `statusStore`, `useI18n()`.
- Produces: Minimal status strip.

- [ ] **Step 1: Remove language switcher and unrelated chrome**

Keep only:
- Sync status text + pulse dot
- Unread count
- Last sync time
- Online/offline indicator
- Version number with i18n (`$t('statusBar.version', { version: '0.1.0' })`)

- [ ] **Step 2: Use semantic tokens and fine borders**

- Background: `bg-elevated`
- Border: `border-t border-border`
- Text: `text-secondary` / `text-tertiary`
- Version: `text-tertiary`

- [ ] **Step 3: Type-check**

```bash
pnpm type-check
```

- [ ] **Step 4: Commit**

```bash
git add src/components/StatusBar.vue
git commit -m "feat(ui): simplify StatusBar and use token-based styling"
```

---

## Task 9: Rewrite `src/components/MailList.vue`

**Files:**
- Modify: `src/components/MailList.vue`
- Inline: bulk-actions bar (remove `BulkActions.vue` later)

**Interfaces:**
- Consumes: `mailStore`, `useI18n()`, `ContextMenu`, `MoveMailDialog`.
- Produces: Focusable mail rows with hover quick actions and inline bulk bar.

- [ ] **Step 1: Rewrite markup and styles**

Key requirements:
- Header: folder title + search input with `<search>` landmark.
- List rows are focusable (`<button>` or `role="button" tabindex="0"`).
- Checkbox has `aria-label="$t('mail.select')"` (add `mail.select` to i18n).
- Hover quick actions: star, archive.
- Inline bulk-actions bar at bottom when selection is non-empty.

- [ ] **Step 2: Add missing i18n key `mail.select`**

In both locale files:

```json
"select": "Select this message"
```

- [ ] **Step 3: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 4: Commit**

```bash
git add src/components/MailList.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ui): rewrite MailList with focusable rows and inline bulk actions"
```

---

## Task 10: Rewrite `src/components/MailViewer.vue`

**Files:**
- Modify: `src/components/MailViewer.vue`

**Interfaces:**
- Consumes: `mailStore`, `useI18n()`, `SandboxedHtml`, `TranslatePanel`, `AiQuickActions`.
- Produces: Reader with corrected toolbar, header, body, attachments.

- [ ] **Step 1: Rewrite toolbar**

- Icon + tooltip labels from i18n.
- Spam icon switches between `AlertTriangle` (mark spam) and `ShieldCheck` / `Inbox` (not spam).
- Reading mode uses `Expand` / `Compress` icons.
- Delete triggers focus-trapped confirmation dialog with Escape close.

- [ ] **Step 2: Rewrite header and body**

- Use token-based spacing and colors.
- Attachment chips are focusable buttons with accessible names.

- [ ] **Step 3: Add focus trap and Escape handling for delete dialog**

Use a small composable or inline `onKeydown` listener in the dialog.

- [ ] **Step 4: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/components/MailViewer.vue
git commit -m "feat(ui): rewrite MailViewer toolbar, fix icon semantics and dialog a11y"
```

---

## Task 11: Improve `MoveMailDialog` and `ContextMenu` accessibility

**Files:**
- Modify: `src/components/MoveMailDialog.vue`
- Modify: `src/components/ContextMenu.vue`

**Interfaces:**
- Consumes: existing props/emits.
- Produces: Accessible dialogs/menus.

- [ ] **Step 1: Add Escape close and focus trap to `MoveMailDialog.vue`**

- Trap focus inside the dialog while open.
- Close on Escape.
- Return focus to the trigger when closed.

- [ ] **Step 2: Add ARIA roles to `ContextMenu.vue`**

- `role="menu"` on the menu container.
- `role="menuitem"` on each action button.
- Arrow-key navigation and Escape close.

- [ ] **Step 3: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 4: Commit**

```bash
git add src/components/MoveMailDialog.vue src/components/ContextMenu.vue
git commit -m "feat(a11y): focus trap and keyboard support for move dialog and context menu"
```

---

## Task 12: Rewrite `ComposeView.vue`

**Files:**
- Modify: `src/views/ComposeView.vue`

**Interfaces:**
- Consumes: `composeStore`, `accountStore`, `useToastStore`, `useI18n()`.
- Produces: Full-page compose shell with i18n toast.

- [ ] **Step 1: Replace hardcoded toast with i18n key**

Change:

```ts
// Before (hardcoded)
toast.error('Please select an account first')

// After
const { t } = useI18n()
toast.error(t('compose.noAccountSelected'))
```

- [ ] **Step 2: Use token-based layout**

- Full-page flex column.
- Header with discard and send.
- Body with editor and attachments.

- [ ] **Step 3: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 4: Commit**

```bash
git add src/views/ComposeView.vue
git commit -m "feat(ui): rewrite ComposeView shell and fix hardcoded toast"
```

---

## Task 13: Rewrite `ComposeHeader.vue`

**Files:**
- Modify: `src/components/compose/ComposeHeader.vue`

**Interfaces:**
- Consumes: `accountStore`, `composeStore`, props (`draft`, `accounts`).
- Produces: Accessible compose header.

- [ ] **Step 1: Add visible labels**

- Account select: `<label for="account-select">{{ $t('compose.selectAccount') }}</label>`
- Subject input: `<label for="subject-input">{{ $t('compose.subject') }}</label>`

- [ ] **Step 2: Send button disabled tooltip**

When disabled, show a tooltip explaining "Select an account and add at least one recipient" (add i18n key `compose.sendDisabledHint`).

- [ ] **Step 3: Use token styling**

- `bg-elevated`, `border-border`, `text-primary`, `text-secondary`.

- [ ] **Step 4: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 5: Commit**

```bash
git add src/components/compose/ComposeHeader.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ui): rewrite ComposeHeader with labels and send tooltip"
```

---

## Task 14: i18n-ize `ComposeEditor.vue` toolbar

**Files:**
- Modify: `src/components/compose/ComposeEditor.vue`

**Interfaces:**
- Consumes: `useI18n()`.
- Produces: Toolbar buttons with i18n labels/titles.

- [ ] **Step 1: Replace hardcoded labels with i18n keys**

For each toolbar button, use `t('compose.bold')`, `t('compose.italic')`, etc.

- [ ] **Step 2: Keep display shortcuts but translate titles**

Buttons can still show "B" / "I" / "U" as visible text, but their `title` attributes use i18n.

- [ ] **Step 3: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 4: Commit**

```bash
git add src/components/compose/ComposeEditor.vue
git commit -m "feat(i18n): localize compose editor toolbar labels"
```

---

## Task 15: Rewrite `SettingsView.vue`

**Files:**
- Modify: `src/views/SettingsView.vue`
- Inline: `AccountForm` and account list management.

**Interfaces:**
- Consumes: `accountStore`, `aiStore`, `settingsStore`, `useI18n()`, `useTheme()`.
- Produces: Single settings page with General, Accounts, Smart Assistants, Translation sections.

- [ ] **Step 1: Add General section with theme toggle**

```vue
<section aria-labelledby="general-heading" class="space-y-4">
  <h2 id="general-heading" class="text-lg font-semibold">{{ $t('settings.language') }}</h2>
  <!-- language selector from Task 7 -->

  <h2 class="text-lg font-semibold pt-4">{{ $t('settings.theme') }}</h2>
  <div class="flex gap-2">
    <button
      v-for="mode in ['dark', 'light', 'auto']"
      :key="mode"
      :class="['px-3 py-1.5 rounded-md border', themeMode === mode ? 'bg-accent text-white border-accent' : 'bg-elevated border-border']"
      @click="setTheme(mode)"
    >
      {{ $t(`settings.theme${mode.charAt(0).toUpperCase() + mode.slice(1)}`) }}
    </button>
  </div>
</section>
```

- [ ] **Step 2: Inline account management**

Move `AccountList` and `AccountForm` content into the Accounts section. Use `AccountForm` component as-is after its labels are updated in Task 16.

- [ ] **Step 3: Use plain-language section titles**

- "Smart Assistants" instead of "AI Providers".
- "Translation" instead of "Translation Providers".

- [ ] **Step 4: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 5: Commit**

```bash
git add src/views/SettingsView.vue src/i18n/locales/en.json src/i18n/locales/zh-CN.json
git commit -m "feat(ui): rewrite SettingsView with General section and plain language"
```

---

## Task 16: Rewrite `AccountForm.vue`

**Files:**
- Modify: `src/components/AccountForm.vue`

**Interfaces:**
- Consumes: `accountStore`, `useI18n()`.
- Produces: Account form with plain-language labels.

- [ ] **Step 1: Update labels and placeholders**

- "Email service" instead of "Provider".
- "Label" instead of "Account Name".
- "Incoming mail server" / "Incoming server port".
- "Outgoing mail server" / "Outgoing server port".
- "Password or app password".

- [ ] **Step 2: Map provider enum values to friendly labels**

Create a local mapping in the component:

```ts
const providerLabels: Record<string, string> = {
  Gmail: 'Gmail',
  Outlook: 'Outlook',
  QQ: 'QQ Mail',
  Netease163: '163.com (Netease)',
  Aliyun: 'Aliyun Mail',
  TencentExmail: 'Tencent Exmail',
  Custom: 'Other',
}
```

Use it in the `<option>` text.

- [ ] **Step 3: Use token styling**

- `bg-elevated`, `border-border`, `rounded-md`, `focus:border-accent`.

- [ ] **Step 4: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/components/AccountForm.vue
git commit -m "feat(ui): rewrite AccountForm with plain-language labels"
```

---

## Task 17: Inline `AccountList` and delete `AccountsView`

**Files:**
- Modify: `src/views/SettingsView.vue`
- Delete: `src/views/AccountsView.vue`
- Delete: `src/components/AccountList.vue`

**Interfaces:**
- Consumes: `accountStore`.
- Produces: Account list inside Settings.

- [ ] **Step 1: Move account list markup into SettingsView Accounts section**

Show account cards with label, email, email service, and delete button.

- [ ] **Step 2: Remove `AccountsView.vue` route/component**

Update `src/router.ts` to remove `/accounts` route if present.

- [ ] **Step 3: Delete files**

```bash
rm src/views/AccountsView.vue src/components/AccountList.vue
```

- [ ] **Step 4: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/views/SettingsView.vue src/views/AccountsView.vue src/components/AccountList.vue src/router.ts
git commit -m "refactor(ui): inline account list into Settings and remove AccountsView"
```

---

## Task 18: Move AI Assistant trigger to Reader toolbar

**Files:**
- Modify: `src/components/MailViewer.vue`
- Modify: `src/components/AiAssistantPanel.vue`
- Modify: `src/components/AiQuickActions.vue`

**Interfaces:**
- Consumes: `aiStore`.
- Produces: AI panel triggered from toolbar, sliding from right.

- [ ] **Step 1: Add AI Assistant button to `MailViewer.vue` toolbar**

```vue
<button :title="$t('aiAssistant.title')" @click="aiStore.togglePanel()">
  <Sparkles class="w-4 h-4" />
</button>
```

- [ ] **Step 2: Update `AiAssistantPanel.vue` to slide from right**

Use fixed positioning, transform translate-x, and overlay.

- [ ] **Step 3: Update `AiQuickActions.vue` labels**

Use i18n keys (`aiActions.summarize`, etc.).

- [ ] **Step 4: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/components/MailViewer.vue src/components/AiAssistantPanel.vue src/components/AiQuickActions.vue
git commit -m "feat(ui): move AI assistant trigger to reader toolbar as right-slide panel"
```

---

## Task 19: i18n-ize `TranslatePanel.vue`

**Files:**
- Modify: `src/components/TranslatePanel.vue`

**Interfaces:**
- Consumes: `useI18n()`.
- Produces: Language options from locale files.

- [ ] **Step 1: Replace hardcoded language strings**

```vue
<option value="en">{{ $t('translation.language.english') }}</option>
<option value="zh-CN">{{ $t('translation.language.chinese') }}</option>
<option value="ja">{{ $t('translation.language.japanese') }}</option>
<option value="ko">{{ $t('translation.language.korean') }}</option>
```

- [ ] **Step 2: Run checks**

```bash
pnpm type-check && pnpm i18n:check
```

- [ ] **Step 3: Commit**

```bash
git add src/components/TranslatePanel.vue
git commit -m "feat(i18n): localize translate panel language options"
```

---

## Task 20: Clean up wrapper views and duplicate routes

**Files:**
- Delete: `src/views/InboxView.vue`
- Modify: `src/router.ts`
- Modify: `src/layouts/AppLayout.vue` if needed

**Interfaces:**
- Produces: Direct routing to `AppLayout` for mail views.

- [ ] **Step 1: Remove `InboxView.vue`**

```bash
rm src/views/InboxView.vue
```

- [ ] **Step 2: Update `src/router.ts`**

Route `/` directly to `AppLayout` (or keep `InboxView` if it still has a purpose; per spec it is removed).

- [ ] **Step 3: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 4: Commit**

```bash
git add src/views/InboxView.vue src/router.ts
git commit -m "refactor(ui): remove InboxView wrapper and simplify routes"
```

---

## Task 21: Add skip link and reduced-motion support

**Files:**
- Modify: `src/App.vue` or `src/layouts/AppLayout.vue`
- Modify: `src/components/ToastContainer.vue`
- Modify: `src/components/CommandPalette.vue`
- Modify: `src/styles/theme.css` (already added global reduced-motion in Task 1)

**Interfaces:**
- Produces: Global accessibility affordances.

- [ ] **Step 1: Add skip links in `AppLayout.vue`**

At the very top of the layout:

```vue
<div class="sr-only focus-within:not-sr-only focus-within:absolute focus-within:z-50 focus-within:p-2 focus-within:bg-accent focus-within:text-white">
  <a href="#mail-list" class="mr-4">Skip to message list</a>
  <a href="#reader">Skip to reader</a>
</div>
```

Add `id="mail-list"` to `MailList` container and `id="reader"` to `main`.

- [ ] **Step 2: Add `aria-live` to `ToastContainer.vue`**

```vue
<div aria-live="polite" aria-atomic="true" class="fixed top-4 right-4 z-50 space-y-2">
  <!-- toasts -->
</div>
```

- [ ] **Step 3: Ensure `CommandPalette.vue` uses reduced-motion class**

Use Tailwind's `motion-reduce:transition-none` on transition wrappers.

- [ ] **Step 4: Run checks**

```bash
pnpm type-check
```

- [ ] **Step 5: Commit**

```bash
git add src/layouts/AppLayout.vue src/components/ToastContainer.vue src/components/CommandPalette.vue
git commit -m "feat(a11y): add skip links, live region, and motion-reduce classes"
```

---

## Task 22: Final verification and cleanup

**Files:**
- All modified files.

**Interfaces:**
- Produces: Passing verification checklist.

- [ ] **Step 1: Run full frontend checks**

```bash
pnpm type-check
pnpm i18n:check
pnpm lint
```

Expected: all pass.

- [ ] **Step 2: Run Rust checks**

```bash
cd src-tauri && cargo clippy && cargo fmt --check
```

Expected: no warnings or formatting issues.

- [ ] **Step 3: Search for remaining ad-hoc values**

```bash
grep -R "yellow-500\|red-500\|bg-yellow-200\|bg-yellow-800\|py-1\.5\|px-1\.5\|w-\[" src/components src/views src/layouts
```

Expected: no matches in rewritten components.

- [ ] **Step 4: Search for hardcoded UI strings**

```bash
grep -R '"[A-Z][a-z]\+"' src/components src/views src/layouts | grep -v '\$t' | grep -v 'placeholder'
```

Manually review output; remove any remaining hardcoded user-facing strings.

- [ ] **Step 5: Launch the app and smoke test**

```bash
cargo tauri dev
```

Verify:
- App launches without errors.
- Sidebar folders render.
- Mail list and reader render.
- Compose route works.
- Settings page shows General/Accounts/Smart Assistants/Translation.
- Theme toggle changes mode.
- Tab navigation reaches all primary controls.
- Escape closes command palette and dialogs.

- [ ] **Step 6: Commit any remaining fixes**

```bash
git add -A
git commit -m "fix(ui): final cleanup after redesign verification"
```

---

## Self-Review

### Spec coverage

| Spec section | Implementing task(s) |
|---|---|
| Token system | Task 1, Task 2 |
| Plain-language terminology | Task 3, Task 4, Task 16, Task 18, Task 19 |
| Information architecture | Task 5, Task 6, Task 9, Task 10, Task 15, Task 17, Task 20 |
| Reduced chrome | Task 5, Task 6, Task 8, Task 9, Task 10 |
| Accessibility | Task 9, Task 10, Task 11, Task 13, Task 21 |
| Compose full-page route | Task 12, Task 13, Task 14 |
| AI panel from toolbar | Task 18 |
| Theme toggle in Settings | Task 7, Task 15 |

### Placeholder scan

- No "TBD", "TODO", or "implement later" remain.
- Every task shows concrete code or commands.
- Every task ends with a verification step and commit.

### Type consistency

- `theme.css` tokens are referenced consistently.
- i18n keys introduced in Task 3 are used in Tasks 6, 7, 9, 10, 12, 13, 14, 15, 16, 18, 19.
- No conflicting function or prop names across tasks.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-22-aeromail-ui-redesign.md`.**

Two execution options:

1. **Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — execute tasks in this session using `superpowers:executing-plans`, batch execution with checkpoints.

Which approach do you want?
