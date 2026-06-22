# Design Audit Scope — AeroMail Frontend UI

## What is being audited

- **Repository**: `/home/xiyeming/CodeSpaces/ToolsProjects/RutMail`
- **Application**: AeroMail — cross-platform desktop email client (Tauri v2 + Vue 3 + TypeScript)
- **Audited surfaces**:
  - Primary mail triage surface: `src/layouts/AppLayout.vue`, `src/components/AppSidebar.vue`, `src/components/MailList.vue`, `src/components/MailViewer.vue`, `src/views/InboxView.vue`
  - Compose surface: `src/views/ComposeView.vue` + `src/components/compose/*`
  - Account management surface: `src/views/AccountsView.vue`, `src/components/AccountForm.vue`, `src/components/AccountList.vue`
  - Settings surface: `src/views/SettingsView.vue`
  - Global chrome & feedback: `src/components/StatusBar.vue`, `src/components/ToastContainer.vue`, `src/components/CommandPalette.vue`, `src/components/ContextMenu.vue`
  - Theme/tokens: `src/styles/theme.css`, `src/styles/fonts.css`

## Primary user and primary task

- **Primary user**: A desktop email user on Linux/macOS/Windows who manages 1–5 personal or work IMAP accounts.
- **Primary task**: Read, triage, search, compose, and send email with minimal friction.
- **Adjacent tasks**: Switch accounts/folders, manage account credentials, configure AI/translation providers, use keyboard shortcuts for power-user workflows.

## Constraints

- **Brand / visual direction**: Dark-first desktop app (default `data-theme="dark"`), Tailwind CSS v4, CSS-variable-based tokens, Chinese + English bilingual UI.
- **Stack**: Vue 3 Composition API, Vue Router, Pinia, vue-i18n, Vite, Tauri v2 Webview.
- **Platform**: Desktop-only; window is resizable; three-column layout is the canonical mail-client form.
- **Accessibility floor**: Keyboard shortcuts exist (`useKeyboardShortcuts.ts`), but no explicit WCAG target is documented.
- **Deadline / maturity**: Active development; no shipping deadline stated. Many features are partially implemented.

## Reference designs / competitors

- Apple Mail, Thunderbird, Outlook, Spark, Notion Mail, Mimestream as functional comparables.
- No explicit reference mock or Figma frame was provided.

## Scope exclusions

- Backend Rust services, database schema, and Tauri IPC commands are out of scope.
- Deep security review of HTML email sandboxing is out of scope (it is implemented but not audited here).
- Iconography and illustration asset quality are noted only if they materially affect a principle.

## Input materials

- Source files listed above.
- No running dev server or deployed URL was available at audit start; visual evidence will be inferred from source tokens and component markup where marked `INFERRED`.
