# Design Audit Verdict — AeroMail Frontend UI

## Verdict: **REDESIGN**

**Total score: 15 / 30.**

The current AeroMail frontend ships a functional email client, but the design does not yet meet the threshold for refinement. The score falls below 20, and multiple load-bearing dimensions (especially **#4 understandable** and **#5 unobtrusive**) are weak. The UI should be redesigned from purpose rather than polished in place.

## Why redesign and not refine

- **#4 Understandable scored 1/3**: a first-time user cannot reliably name several primary controls. Technical jargon ("IMAP Host", "API Key", "Endpoint", raw provider identifiers) is exposed directly in the UI, and icon semantics are reversed or static across states.
- **#5 Unobtrusive scored 1/3**: the reading surface is surrounded by heavy borders, multiple sidebars, status bars, and toolbars; the interface competes with the content it is meant to display.
- **#3 Aesthetic scored 1/3** and **#10 As little design as possible scored 1/3**: the visual system is inconsistent (no type-scale tokens, ad-hoc spacing and colors, duplicated affordances, wrapper views with no behavior).
- While the primary task is supported (#2 Useful = 2/3), the surrounding friction is high enough that incremental refinement would leave the core problems intact.

## Highest-leverage moves

1. **#4 Understandable — Rename technical labels to plain language and fix icon/label mismatches.**
   - Evidence: "IMAP Host/Port", "SMTP Host/Port", "API Key", "Base URL", "Endpoint" (`AccountForm.vue:109-181`, `SettingsView.vue:187-228`); raw identifiers `azure_openai`, `Netease163`, `TencentExmail` (`SettingsView.vue:200-201`, `AccountForm.vue:117`); `AlertTriangle` spam icon never changes (`MailViewer.vue:234`); `Maximize2`/`Minimize2` reading-mode icons are semantically reversed (`MailViewer.vue:262`).

2. **#5 Unobtrusive — Reduce chrome and reclaim the reading surface.**
   - Evidence: every panel/card/toolbar uses `border-border` (`AppLayout.vue`, `MailList.vue`, `MailViewer.vue`); three-column layout is nine DOM levels deep before reaching a toolbar button (`AppLayout.vue:22` → `MailViewer.vue:215`); status bar, bulk-actions bar, two sidebars, and a full toolbar surround the message (`AppLayout.vue:21-57`, `StatusBar.vue`, `BulkActions.vue`).

3. **#3 Aesthetic — Lock a single token system for type, spacing, and color.**
   - Evidence: no type-scale tokens (`theme.css`, `fonts.css`); ad-hoc spacing `py-1.5`, `px-1.5`, `min-w-[120px]`, `w-[360px]` (`AccountForm.vue:114`, `RecipientInput.vue:26`, `AiAssistantPanel.vue:45`, `CommandPalette.vue:211`); ad-hoc functional colors `text-yellow-500`, `text-red-500`, `bg-yellow-200/800` and many opacity modifiers (`MailViewer.vue:217`, `ContextMenu.vue:142`, `CommandPalette.vue:249`).

4. **#10 As little design as possible — Remove duplicate affordances and empty wrappers.**
   - Evidence: language switcher appears in both `StatusBar.vue:71` and `CommandPalette.vue:49`; AI assistant toggle in `AppSidebar.vue:173` and `CommandPalette.vue:64`; account list is read-only/non-functional in `AppSidebar.vue:127` and functional in `AccountList.vue:27`; `InboxView.vue` and `AccountsView.vue` are near-empty wrappers.

5. **#8 Thorough — Add accessibility and motion details that the current design omits.**
   - Evidence: no `prefers-reduced-motion` support anywhere; no skip-link; sidebar account items and mail rows are not focusable (`AppSidebar.vue:127-138`, `MailList.vue:206-215`); modals lack focus trapping and Escape handling (`MoveMailDialog.vue`, `MailViewer.vue` delete dialog); mail-selection checkbox and subject input lack accessible names (`MailList.vue:218`, `ComposeHeader.vue:46`).
