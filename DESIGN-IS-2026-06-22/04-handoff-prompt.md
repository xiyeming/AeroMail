# /make-plan Handoff — AeroMail Frontend UI Redesign

Copy the fenced prompt below into the next session to generate the redesign plan.

---

````markdown
/make-plan Redesign the AeroMail frontend UI. Current design failed audit at 15/30 with critical gaps in principles #4 (understandable), #5 (unobtrusive), #3 (aesthetic), and #10 (as little design as possible).

Verdict paragraph (quoted from 03-verdict.md):
> The current AeroMail frontend ships a functional email client, but the design does not yet meet the threshold for refinement. The score falls below 20, and multiple load-bearing dimensions (especially #4 understandable and #5 unobtrusive) are weak. The UI should be redesigned from purpose rather than polished in place.

Why redesign and not refine: The primary task is supported, but understandable, unobtrusive, aesthetic, and restraint principles all scored 1/3, meaning the structure and language—not just styling—need to change.

Preserve from current design (MUST be non-empty):
- Dark-first default theme and CSS-variable token architecture (`src/styles/theme.css`, `src/main.ts:9`).
- Three-column mail-client layout pattern (`src/layouts/AppLayout.vue:21-57`) as the canonical desktop form.
- Existing i18n infrastructure (`src/i18n/index.ts`, `src/i18n/locales/en.json`, `src/i18n/locales/zh-CN.json`) and bilingual requirement.
- Vue 3 + Tailwind CSS v4 + Pinia stack constraint.
- Keyboard-shortcut foundation (`src/composables/useKeyboardShortcuts.ts`) and command palette concept (`src/components/CommandPalette.vue`).
- HTML-email sandbox security model (`src/components/SandboxedHtml.vue`).

Discard (MUST be non-empty):
- Heavy bordering on every panel/card/toolbar (`border-border` everywhere in `AppLayout.vue`, `MailList.vue`, `MailViewer.vue`). Caused failure on principle #5 (unobtrusive).
- Technical/jargon labels exposed to users: "IMAP Host/Port", "SMTP Host/Port", "API Key", "Base URL", "Endpoint", and raw provider identifiers (`AccountForm.vue:109-181`, `SettingsView.vue:187-228`). Caused failure on principle #4 (understandable).
- Duplicate affordances: language switcher in status bar and command palette (`StatusBar.vue:71`, `CommandPalette.vue:49`); AI assistant toggle in sidebar and command palette (`AppSidebar.vue:173`, `CommandPalette.vue:64`); account list in sidebar and accounts view (`AppSidebar.vue:127`, `AccountList.vue:27`). Caused failure on principle #10 (as little design as possible).
- Ad-hoc spacing, type, and color values (`py-1.5`, `px-1.5`, `min-w-[120px]`, `w-[360px]`, `text-yellow-500`, `bg-yellow-200/800`, no type-scale tokens). Caused failure on principle #3 (aesthetic).
- Non-focusable interactive elements (`AppSidebar.vue:127-138`, `MailList.vue:206-215`, `MailViewer.vue:373-382`, `CommandPalette.vue:227-243`) and modals without focus trapping/Escape handling (`MoveMailDialog.vue`, `MailViewer.vue` delete dialog). Caused failure on principle #8 (thorough).

Top 5 moves from the audit (verbatim):
1. **#4 — Understandable**: Rename technical labels to plain language and fix icon/label mismatches. Evidence: "IMAP Host/Port", "SMTP Host/Port", "API Key", "Base URL", "Endpoint" (`AccountForm.vue:109-181`, `SettingsView.vue:187-228`); raw identifiers `azure_openai`, `Netease163`, `TencentExmail` (`SettingsView.vue:200-201`, `AccountForm.vue:117`); `AlertTriangle` spam icon never changes (`MailViewer.vue:234`); `Maximize2`/`Minimize2` reading-mode icons are semantically reversed (`MailViewer.vue:262`).
2. **#5 — Unobtrusive**: Reduce chrome and reclaim the reading surface. Evidence: every panel/card/toolbar uses `border-border` (`AppLayout.vue`, `MailList.vue`, `MailViewer.vue`); three-column layout is nine DOM levels deep before reaching a toolbar button (`AppLayout.vue:22` → `MailViewer.vue:215`); status bar, bulk-actions bar, two sidebars, and a full toolbar surround the message (`AppLayout.vue:21-57`, `StatusBar.vue`, `BulkActions.vue`).
3. **#3 — Aesthetic**: Lock a single token system for type, spacing, and color. Evidence: no type-scale tokens (`theme.css`, `fonts.css`); ad-hoc spacing `py-1.5`, `px-1.5`, `min-w-[120px]`, `w-[360px]` (`AccountForm.vue:114`, `RecipientInput.vue:26`, `AiAssistantPanel.vue:45`, `CommandPalette.vue:211`); ad-hoc functional colors `text-yellow-500`, `text-red-500`, `bg-yellow-200/800` and many opacity modifiers (`MailViewer.vue:217`, `ContextMenu.vue:142`, `CommandPalette.vue:249`).
4. **#10 — As little design as possible**: Remove duplicate affordances and empty wrappers. Evidence: language switcher appears in both `StatusBar.vue:71` and `CommandPalette.vue:49`; AI assistant toggle in `AppSidebar.vue:173` and `CommandPalette.vue:64`; account list is read-only/non-functional in `AppSidebar.vue:127` and functional in `AccountList.vue:27`; `InboxView.vue` and `AccountsView.vue` are near-empty wrappers.
5. **#8 — Thorough**: Add accessibility and motion details that the current design omits. Evidence: no `prefers-reduced-motion` support anywhere; no skip-link; sidebar account items and mail rows are not focusable (`AppSidebar.vue:127-138`, `MailList.vue:206-215`); modals lack focus trapping and Escape handling (`MoveMailDialog.vue`, `MailViewer.vue` delete dialog); mail-selection checkbox and subject input lack accessible names (`MailList.vue:218`, `ComposeHeader.vue:46`).

Redesign principles in priority order:
1. **#4 — Understandable** — every label, icon, and control is understandable to a non-technical email user on first sight; no raw API identifiers or server jargon.
2. **#5 — Unobtrusive** — content is the figure and chrome is the ground; reduce borders, consolidate bars, and maximize the reading area.
3. **#10 — As little design as possible** — every element earns its place; remove duplicate toggles, empty wrappers, and decorative dividers.
4. **#3 — Aesthetic** — one systematic type scale, one spacing scale, and a capped semantic color palette; no ad-hoc utilities.
5. **#8 — Thorough** — all primary controls are keyboard-focusable, modals trap focus and close with Escape, motion respects `prefers-reduced-motion`, and empty/loading/error states are designed.

Deliverables for the plan:
- New information architecture (not derived from old) — sidebar, list pane, reader pane, compose, accounts, settings.
- New primary flow (low-fi, labeled, compared side-by-side to current) — read/triage, compose/send, add account, configure AI/translation.
- Token/spec changes consolidated in one place — type scale, spacing scale, color palette cap, border/shadow philosophy.
- States checklist (empty, loading, error, success, focus, disabled) for every primary surface.
- Accessibility checklist (focus order, keyboard shortcuts, ARIA landmarks, skip-link, reduced-motion support, contrast targets).
- Migration path for users currently on the old design — this is a single-user desktop app, so migration is cutover on next update.
- Cutover criteria — old design is retired when the new design passes the same Dieter Rams audit at ≥20/30 with no 0 scores.

Anti-patterns to guard against (specific to REDESIGN):
- Porting old structure under new styling.
- Keeping both designs behind a flag indefinitely.
- Redesigning to follow a trend rather than the principles above.
- Treating the Preserve list as optional — it must be filled before this handoff is valid.
````
