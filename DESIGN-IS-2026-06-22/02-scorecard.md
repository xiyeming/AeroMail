# Dieter Rams Scorecard — AeroMail Frontend UI

## 1. Good design is innovative — Score: 1/3
- **Evidence**: Three-column mail layout (`AppLayout.vue:21-57`), folder sidebar (`AppSidebar.vue:80-181`), toolbar actions (`MailViewer.vue:186-281`), command palette (`CommandPalette.vue:198-273`), AI assistant (`AiAssistantPanel.vue`).
- **Justification**: The client follows the standard Thunderbird/Outlook/Apple Mail three-pane pattern with command-palette and AI-assistant features common in modern clients (Spark, Notion Mail); no new interaction pattern is introduced.

## 2. Good design makes a product useful — Score: 2/3
- **Evidence**: Read/triage/compose/search tasks are all reachable, but `Archive` has no keyboard shortcut (`useKeyboardShortcuts.ts`), sidebar account list items are non-functional/clickable-but-unfocusable (`AppSidebar.vue:127-138`), and switching accounts requires finding the small status-bar language/account affordances.
- **Justification**: Primary email tasks complete, but adjacent surfaces add friction and steps.

## 3. Good design is aesthetic — Score: 1/3
- **Evidence**: No type-scale tokens (`theme.css`, `fonts.css`); ad-hoc spacing (`py-1.5`, `px-1.5`, `min-w-[120px]`, `w-[360px]`); ad-hoc hardcoded colors (`text-yellow-500`, `text-red-500`, `bg-yellow-200/800`, `bg-primary/10`, `bg-muted/20`); heavy `border-border` on nearly every panel/card/toolbar (`AppLayout.vue`, `MailList.vue`, `MailViewer.vue`).
- **Justification**: Three to four visual-system inconsistencies plus pervasive bordering create visual noise.

## 4. Good design makes a product understandable — Score: 1/3
- **Evidence**: Jargon labels "Provider", "AI Providers", "API Key", "Base URL", "Endpoint", "IMAP Host/Port", "SMTP Host/Port" (`AccountForm.vue`, `SettingsView.vue`); raw provider identifiers shown to users (`azure_openai`, `custom_openai_compatible`, `moonshot`, `Netease163`, `TencentExmail`); icon/label mismatches (`AlertTriangle` for both spam and not-spam, `Maximize2` for reading mode which actually collapses chrome).
- **Justification**: More than two controls are unclear without domain knowledge, and several icons do not match their labels.

## 5. Good design is unobtrusive — Score: 1/3
- **Evidence**: Every panel and card is bordered (`border-border`); status bar, bulk-actions bar, two sidebars, and a nine-level-deep toolbar dominate the reading surface (`AppLayout.vue`, `MailViewer.vue:186-281`, `StatusBar.vue`, `BulkActions.vue`).
- **Justification**: Chrome and decoration compete with email content for attention.

## 6. Good design is honest — Score: 2/3
- **Evidence**: No marketing superlatives or dark patterns found; however, the Send button is disabled when subject is empty (`ComposeHeader.vue:79`) without explaining why, spam icon does not change with state (`MailViewer.vue:234`), and reading-mode icon semantics are reversed.
- **Justification**: No deceptive flows, but a few label/behavior mismatches weaken honesty.

## 7. Good design is long-lasting — Score: 2/3
- **Evidence**: Neutral dark-first palette and CSS-variable system (`theme.css`); no skeuomorphs or gradients; but heavy borders and ad-hoc accent colors risk a dated utility-first look.
- **Justification**: One potential dated marker (heavy bordering / ad-hoc accent colors).

## 8. Good design is thorough down to the last detail — Score: 2/3
- **Evidence**: Empty, loading, error, success, focus, and disabled states are all present (`MailList.vue`, `MailViewer.vue`, `AccountForm.vue`, `SettingsView.vue`, `ToastContainer.vue`), but `prefers-reduced-motion` is ignored, no skip-link exists, focus trapping is missing in modals/dialogs, and several interactive elements lack focus states.
- **Justification**: Core states are covered, but accessibility and motion details are rough or missing.

## 9. Good design is environmentally friendly — Score: 2/3
- **Evidence**: Initial JS ~258 KB (`dist/assets/index-*.js` 220 KB + locale 4 KB + CSS 28 KB), dark mode by default (`main.ts:9`), no autoplay video, but `prefers-reduced-motion` is not respected and `animate-pulse` sync dot runs unconditionally (`StatusBar.vue:55`).
- **Justification**: Bundle is under 500 KB and dark mode is honored, but motion is not gated.

## 10. Good design is as little design as possible — Score: 1/3
- **Evidence**: Wrapper views with no logic (`InboxView.vue`, `AccountsView.vue`), duplicate toggles for language (`StatusBar.vue:71` + `CommandPalette.vue:49`) and AI assistant (`AppSidebar.vue:173` + `CommandPalette.vue:64`), duplicate account lists (`AppSidebar.vue:127` + `AccountList.vue:27`), and pervasive bordering.
- **Justification**: Several elements are removable or duplicated without breaking the primary task.

---

**Total: 15 / 30**
