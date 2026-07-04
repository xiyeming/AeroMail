<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import {
  Languages,
  Star,
  Trash2,
  Expand,
  Minimize2,
  ChevronDown,
  ChevronUp,
  Mail,
  Paperclip,
  Reply,
  ReplyAll,
  Forward,
  Archive,
  AlertTriangle,
  ShieldCheck,
  Sparkles,
  X,
  Loader2,
} from '@lucide/vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import AiQuickActions from '@/components/AiQuickActions.vue';
import SandboxedHtml from '@/components/SandboxedHtml.vue';
import TranslatePanel from '@/components/TranslatePanel.vue';
import BaseSelect from '@/components/BaseSelect.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import { useToastStore } from '@/stores/toast';
import { useSettingsStore } from '@/stores/settings';
import { useTodoStore } from '@/stores/todo';
import { useTranslation } from '@/composables/useTranslation';
import type { AttachmentInfo } from '@/types/mail';
import type { TranslationProviderSummary } from '@/types/translation';

const { t } = useI18n();
const router = useRouter();
const { summarizeMail, extractTodos } = useAiChat();
const {
  translateText,
  listProviders: listTranslationProviders,
  getDefaultTargetLang,
  loadDefaultTargetLang,
  saveDefaultTargetLang,
  isTranslating,
} = useTranslation();
const aiStore = useAiStore();
const mailStore = useMailStore();
const toast = useToastStore();
const settingsStore = useSettingsStore();
const todoStore = useTodoStore();

const translatedText = ref<string | null>(null);
const translatedLang = ref<string | null>(null);
const translationCollapsed = ref(false);
const showTranslatePanel = ref(false);
const showDeleteConfirm = ref(false);
const attachments = ref<AttachmentInfo[]>([]);
const deleteDialogRef = ref<HTMLDivElement | null>(null);
const trustedDomains = ref<string[]>([]);
const temporarilyAllowedDomains = ref<string[]>([]);
const inlineImageMap = ref<Record<string, string>>({});
const attachmentsLoaded = ref(false);
const cspBlockedDomains = ref<string[]>([]);
const domRemoteDomains = ref<string[]>([]);
const cspRevision = ref(0);
const sandboxedHtmlRef = ref<InstanceType<typeof SandboxedHtml> | null>(null);
const selectedText = ref('');
const showSelectionMenu = ref(false);
const selectionMenuX = ref(0);
const selectionMenuY = ref(0);
const selectionMenuRef = ref<HTMLDivElement | null>(null);
const translationProviders = ref<TranslationProviderSummary[]>([]);
const selectedTranslationProviderId = ref('');
const translationTargetLang = ref(getDefaultTargetLang());

const translationLanguages = [
  { value: 'en', label: 'translation.language.english' },
  { value: 'zh-CN', label: 'translation.language.chinese' },
  { value: 'ja', label: 'translation.language.japanese' },
  { value: 'ko', label: 'translation.language.korean' },
];

const summaryText = ref('');
const isSummarizing = ref(false);
const isExtractingTodos = ref(false);

async function loadTranslationProviders() {
  translationProviders.value = await listTranslationProviders();
  if (translationProviders.value.length > 0 && !selectedTranslationProviderId.value) {
    selectedTranslationProviderId.value = translationProviders.value[0].id;
  }
  translationTargetLang.value = await loadDefaultTargetLang();
}

watch(translationTargetLang, (newLang) => {
  saveDefaultTargetLang(newLang);
});

function getSelectedText(): string {
  return window.getSelection()?.toString()?.trim() ?? '';
}

function clampMenuPosition(
  x: number,
  y: number,
  menuWidth = 120,
  menuHeight = 36
): { x: number; y: number } {
  const padding = 8;
  const maxX = window.innerWidth - menuWidth - padding;
  const maxY = window.innerHeight - menuHeight - padding;
  return {
    x: Math.max(padding, Math.min(x, maxX)),
    y: Math.max(padding, Math.min(y, maxY)),
  };
}

function updateSelectionMenu(e?: MouseEvent) {
  if (e) {
    const target = e.target as Node;
    const iframe = sandboxedHtmlRef.value?.iframeRef;
    // iframe 内的选区由 SandboxedHtml 的 selection 事件单独处理，
    // 父页面 mouseup 冒泡上来时不要把它覆盖/隐藏掉。
    if (iframe && (target === iframe || iframe.contains(target as Node))) return;
    // 点击翻译菜单内部时也不要因父页面选区为空而隐藏菜单。
    if (selectionMenuRef.value?.contains(target)) return;
  }
  const text = getSelectedText();
  if (!text) {
    showSelectionMenu.value = false;
    selectedText.value = '';
    return;
  }
  selectedText.value = text;
  const event = e ?? (window.event as MouseEvent | undefined);
  if (event) {
    const pos = clampMenuPosition(event.clientX, event.clientY - 8);
    selectionMenuX.value = pos.x;
    selectionMenuY.value = pos.y;
  }
  showSelectionMenu.value = true;
}

function handleIframeSelection(payload: { text: string; clientX: number; clientY: number }) {
  if (!payload.text) return;
  const iframe = sandboxedHtmlRef.value?.iframeRef;
  if (!iframe) return;
  const rect = iframe.getBoundingClientRect();
  selectedText.value = payload.text;
  const pos = clampMenuPosition(rect.left + payload.clientX, rect.top + payload.clientY - 8);
  selectionMenuX.value = pos.x;
  selectionMenuY.value = pos.y;
  showSelectionMenu.value = true;
}

function handleLinkClick(url: string) {
  invoke('open_url', { url }).catch((e) => {
    console.error('Failed to open link:', e);
  });
}

async function translateSelectedText() {
  if (!selectedText.value) return;
  if (!selectedTranslationProviderId.value) {
    toast.add({
      type: 'warning',
      message: t('translation.noProviders'),
      duration: 4000,
    });
    showSelectionMenu.value = false;
    return;
  }
  showSelectionMenu.value = false;
  try {
    const translated = await translateText(
      selectedText.value,
      translationTargetLang.value,
      selectedTranslationProviderId.value
    );
    handleTranslated({ text: translated, lang: translationTargetLang.value });
  } catch (e) {
    toast.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
      duration: 5000,
    });
  }
}

function hideSelectionMenu(e?: MouseEvent) {
  if (e && selectionMenuRef.value?.contains(e.target as Node)) return;
  showSelectionMenu.value = false;
}

function normalizeCid(cid: string): string {
  return cid.trim().replace(/^<|>$/g, '').toLowerCase();
}

function uint8ArrayToBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i += 1) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

const currentMailId = computed(() => mailStore.selectedMailId);
const mail = computed(() => mailStore.selectedMail);
const isStarred = computed(() => mail.value?.isStarred ?? false);
const isSpam = computed(() => mail.value?.isSpam ?? false);
const isArchived = computed(() => mail.value?.isArchived ?? false);

async function loadAttachments(mailId: string) {
  try {
    const list = await invoke<AttachmentInfo[]>('get_attachments', { mailId });
    attachments.value = list;

    const map: Record<string, string> = {};
    await Promise.all(
      list
        .filter((att) => att.contentId)
        .map(async (att) => {
          try {
            const bytes = await invoke<number[]>('get_attachment_content', {
              attachmentId: att.id,
            });
            const dataUrl = `data:${att.mimeType || 'application/octet-stream'};base64,${uint8ArrayToBase64(new Uint8Array(bytes))}`;
            const fullCid = normalizeCid(att.contentId!);
            map[fullCid] = dataUrl;
            // HTML often references inline images by the local part only (cid:logo),
            // while Content-ID headers include a domain suffix (<logo@domain.com>).
            // Register both forms so either reference resolves.
            const atIdx = fullCid.indexOf('@');
            if (atIdx > 0) {
              map[fullCid.slice(0, atIdx)] = dataUrl;
            }
          } catch (e) {
            console.error(`Failed to load inline image ${att.filename}:`, e);
          }
        })
    );
    inlineImageMap.value = map;
  } catch (e) {
    console.error('Failed to load attachments:', e);
  } finally {
    attachmentsLoaded.value = true;
  }
}

async function downloadAttachment(att: AttachmentInfo) {
  try {
    const targetPath = await save({ defaultPath: att.filename });
    if (!targetPath) {
      return;
    }
    await invoke('download_attachment', {
      attachmentId: att.id,
      targetPath,
    });
    toast.add({ type: 'success', message: t('mail.attachmentSaved') });
  } catch (e) {
    console.error('Failed to download attachment:', e);
    toast.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
      duration: 5000,
    });
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function handleQuickAction(action: 'summarize' | 'extractTodos') {
  if (!currentMailId.value) return;
  const providerId = aiStore.resolveProviderId();
  if (!providerId) {
    toast.add({
      type: 'error',
      message: t('aiActions.noProvider'),
      duration: 5000,
    });
    return;
  }

  if (action === 'summarize') {
    isSummarizing.value = true;
    try {
      summaryText.value = await summarizeMail(currentMailId.value, providerId);
    } catch (e) {
      toast.add({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
        duration: 5000,
      });
    } finally {
      isSummarizing.value = false;
    }
    return;
  }

  if (action === 'extractTodos') {
    isExtractingTodos.value = true;
    try {
      const items = await extractTodos(currentMailId.value, providerId);
      await todoStore.setFromAiTodos(items, currentMailId.value);
      todoStore.openPanel();
    } catch (e) {
      toast.add({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
        duration: 5000,
      });
    } finally {
      isExtractingTodos.value = false;
    }
    return;
  }
}

function closeSummary() {
  summaryText.value = '';
}

function handleTranslated(payload: { text: string; lang: string }) {
  translatedText.value = payload.text;
  translatedLang.value = payload.lang;
  translationCollapsed.value = false;
  showTranslatePanel.value = false;
}

function clearTranslation() {
  translatedText.value = null;
  translatedLang.value = null;
  translationCollapsed.value = false;
}

function toggleTranslationCollapsed() {
  translationCollapsed.value = !translationCollapsed.value;
}

function handleToggleStar() {
  if (currentMailId.value) {
    mailStore.toggleStar(currentMailId.value);
  }
}

function handleArchive() {
  if (currentMailId.value) {
    mailStore.archiveMail(currentMailId.value);
  }
}

function handleToggleSpam() {
  if (currentMailId.value) {
    mailStore.toggleSpam(currentMailId.value);
  }
}

function handleDelete() {
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  if (!currentMailId.value) return;
  await mailStore.deleteMail(currentMailId.value);
  showDeleteConfirm.value = false;
}

function cancelDelete() {
  showDeleteConfirm.value = false;
}

function handleReply() {
  if (currentMailId.value) {
    router.push({ name: 'reply', params: { mailId: currentMailId.value } });
  }
}

function handleReplyAll() {
  if (currentMailId.value) {
    router.push({ name: 'reply-all', params: { mailId: currentMailId.value } });
  }
}

function handleForward() {
  if (currentMailId.value) {
    router.push({ name: 'forward', params: { mailId: currentMailId.value } });
  }
}

function formatDate(timestamp: number | null): string {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

function formatAddresses(addresses: string | null): string {
  if (!addresses) return '';
  try {
    const parsed = JSON.parse(addresses) as string[];
    return parsed.join(', ');
  } catch {
    return addresses;
  }
}

function extractRemoteDomains(html: string): string[] {
  const domains = new Set<string>();

  const capture = (url: string) => {
    try {
      const normalized = url.startsWith('//') ? `https:${url}` : url;
      domains.add(new URL(normalized).hostname.toLowerCase());
    } catch {
      // ignore invalid URLs
    }
  };

  // HTML attributes: src / href / poster / background / data-src / data-original
  // Supports double-quoted, single-quoted, and unquoted values.
  const attrRe =
    /(?:src|href|poster|background|data-src|data-original)\s*=\s*(?:"((?:https?:)?\/\/[^"]*)"|'((?:https?:)?\/\/[^']*)'|((?:https?:)?\/\/[^\s>]+))/gi;
  let match: RegExpExecArray | null;
  while ((match = attrRe.exec(html)) !== null) {
    const url = match[1] || match[2] || match[3];
    if (url) capture(url);
  }

  // CSS url(...) including inline styles and <style> blocks
  const urlRe = /url\(\s*(['"]?)((?:https?:)?\/\/[^"')]+)\1\s*\)/gi;
  while ((match = urlRe.exec(html)) !== null) {
    capture(match[2]);
  }

  // @import url(...)
  const importRe = /@import\s+(?:url\()?\s*["']((?:https?:)?\/\/[^"')]+)["']\s*\)?/gi;
  while ((match = importRe.exec(html)) !== null) {
    capture(match[1]);
  }

  // srcset attribute (comma-separated url descriptors, supports protocol-relative)
  const srcsetRe = /srcset\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi;
  while ((match = srcsetRe.exec(html)) !== null) {
    const value = match[1] || match[2] || match[3];
    if (!value) continue;
    for (const part of value.split(',')) {
      const trimmed = part.trim();
      const spaceIdx = trimmed.search(/\s/);
      const url = spaceIdx > 0 ? trimmed.slice(0, spaceIdx) : trimmed;
      if (url.startsWith('http') || url.startsWith('//')) {
        capture(url);
      }
    }
  }

  return Array.from(domains).sort();
}

const remoteDomains = computed(() => {
  if (!mail.value?.bodyHtml) return domRemoteDomains.value;
  const fromRegex = extractRemoteDomains(mail.value.bodyHtml);
  const all = new Set([...fromRegex, ...cspBlockedDomains.value, ...domRemoteDomains.value]);
  return Array.from(all).sort();
});

const processedBodyHtml = computed(() => {
  const html = mail.value?.bodyHtml;
  if (!html) return html ?? null;
  // Wait for inline image attachments to finish loading before rendering the
  // iframe.  Without this guard the iframe first receives raw cid: references
  // that browsers cannot resolve — images fail silently and no CSP violation
  // is emitted (cid: is allow-listed), so the security banner never appears.
  if (!attachmentsLoaded.value) return null;
  const map = inlineImageMap.value;
  if (Object.keys(map).length === 0) return html;
  return html.replace(/cid:\s*<?([^>"'\s)]+)>?/gi, (match, cid: string) => {
    const dataUrl = map[normalizeCid(cid)];
    return dataUrl || match;
  });
});

const allowedDomains = computed(() => {
  // 只传递当前邮件中出现的已授权域名，而非所有历史授权域名
  const currentDomains = new Set(remoteDomains.value);
  const relevant = trustedDomains.value.filter((d) => currentDomains.has(d) || d === '*');
  return [...relevant, ...temporarilyAllowedDomains.value];
});

const untrustedDomains = computed(() =>
  remoteDomains.value.filter(
    (d) => !allowedDomains.value.includes(d) && !allowedDomains.value.includes('*')
  )
);

const showSecurityBanner = computed(() => untrustedDomains.value.length > 0);

function resetSecurityDomainTracking() {
  cspBlockedDomains.value = [];
  domRemoteDomains.value = [];
}

function allowRemoteOnce() {
  console.debug('[MailViewer] allowRemoteOnce, untrustedDomains:', untrustedDomains.value);
  temporarilyAllowedDomains.value = [...untrustedDomains.value];
  resetSecurityDomainTracking();
  cspRevision.value++;
  console.debug('[MailViewer] allowedDomains after allow:', allowedDomains.value);
}

function allowAllRemoteOnce() {
  console.debug('[MailViewer] allowAllRemoteOnce');
  temporarilyAllowedDomains.value = ['*'];
  resetSecurityDomainTracking();
  cspRevision.value++;
}

async function trustDomain(domain: string) {
  console.debug('[MailViewer] trustDomain:', domain);
  if (!trustedDomains.value.includes(domain)) {
    trustedDomains.value.push(domain);
    await settingsStore.set('trustedDomains', JSON.stringify(trustedDomains.value));
    resetSecurityDomainTracking();
    cspRevision.value++;
    console.debug('[MailViewer] allowedDomains after trust:', allowedDomains.value);
  }
}

async function loadTrustedDomains() {
  try {
    const raw = await settingsStore.get('trustedDomains');
    if (raw) {
      trustedDomains.value = JSON.parse(raw);
      console.debug('[MailViewer] loaded trustedDomains:', trustedDomains.value);
    }
  } catch (e) {
    console.error('Failed to load trusted domains:', e);
  }
}

function handleSecurityViolation(violation: { domain: string; blockedUri: string }) {
  if (!cspBlockedDomains.value.includes(violation.domain)) {
    cspBlockedDomains.value.push(violation.domain);
  }
}

function handleRemoteDomains(domains: string[]) {
  const currentSet = new Set(domRemoteDomains.value);
  const hasNew = domains.some((d) => !currentSet.has(d));
  if (!hasNew) return;
  domRemoteDomains.value = Array.from(new Set([...domRemoteDomains.value, ...domains]));
}

onMounted(() => {
  void loadTrustedDomains();
  void aiStore.loadDefaultProvider();
  void loadTranslationProviders();
  document.addEventListener('mouseup', updateSelectionMenu);
  document.addEventListener('mousedown', hideSelectionMenu);
});

onUnmounted(() => {
  document.removeEventListener('mouseup', updateSelectionMenu);
  document.removeEventListener('mousedown', hideSelectionMenu);
});

// Focus trap for delete dialog
function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  return Array.from(container.querySelectorAll(selector)).filter(
    (el) => !el.hasAttribute('disabled') && (el as HTMLElement).offsetParent !== null
  ) as HTMLElement[];
}

function handleDeleteKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    cancelDelete();
    return;
  }
  if (e.key !== 'Tab' || !deleteDialogRef.value) return;

  const focusable = getFocusableElements(deleteDialogRef.value);
  if (focusable.length === 0) return;
  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

watch(showDeleteConfirm, async (show) => {
  if (show) {
    await nextTick();
    const focusable = deleteDialogRef.value ? getFocusableElements(deleteDialogRef.value) : [];
    focusable[0]?.focus();
  }
});

watch(currentMailId, (newMailId) => {
  translatedText.value = null;
  translatedLang.value = null;
  translationCollapsed.value = false;
  showTranslatePanel.value = false;
  showDeleteConfirm.value = false;
  temporarilyAllowedDomains.value = [];
  attachments.value = [];
  inlineImageMap.value = {};
  attachmentsLoaded.value = false;
  cspBlockedDomains.value = [];
  domRemoteDomains.value = [];
  summaryText.value = '';
  if (newMailId) {
    void loadAttachments(newMailId);
  }
});
</script>

<template>
  <div class="flex h-full flex-col bg-base">
    <!-- Loading error banner -->
    <div
      v-if="mailStore.error"
      class="shrink-0 border-b border-border bg-danger/10 px-4 py-2 text-sm text-danger"
      role="alert"
    >
      <div class="flex items-center gap-2">
        <AlertTriangle class="h-4 w-4 shrink-0" aria-hidden="true" />
        <span class="flex-1">{{ mailStore.error }}</span>
        <button
          type="button"
          class="rounded p-1 text-danger hover:bg-danger/10"
          :title="t('mail.dismissError')"
          @click="mailStore.error = null"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="!currentMailId"
      class="flex flex-1 flex-col items-center justify-center gap-4 text-secondary"
    >
      <Mail class="h-16 w-16 opacity-20" aria-hidden="true" />
      <div class="text-center">
        <p class="text-lg font-medium">{{ $t('mail.selectEmail') }}</p>
        <p class="mt-1 text-sm">{{ $t('mail.selectEmailHint') }}</p>
      </div>
      <div class="mt-4 flex gap-4 text-xs text-tertiary">
        <span
          ><kbd class="rounded bg-raised px-2 py-0.5">J</kbd> /
          <kbd class="rounded bg-raised px-2 py-0.5">K</kbd> {{ $t('mail.navigate') }}</span
        >
        <span><kbd class="rounded bg-raised px-2 py-0.5">Enter</kbd> {{ $t('mail.open') }}</span>
        <span><kbd class="rounded bg-raised px-2 py-0.5">Esc</kbd> {{ $t('mail.close') }}</span>
      </div>
    </div>

    <template v-else-if="mail">
      <!-- Toolbar -->
      <div class="flex items-center justify-between border-b border-border px-3 py-2">
        <AiQuickActions
          :is-summarizing="isSummarizing"
          :is-extracting-todos="isExtractingTodos"
          @summarize="handleQuickAction('summarize')"
          @extract-todos="handleQuickAction('extractTodos')"
        />
        <div class="flex items-center gap-1">
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :class="{ 'text-warning': isStarred }"
            :title="isStarred ? t('mail.unstar') : t('mail.star')"
            @click="handleToggleStar"
          >
            <Star class="h-4 w-4" :fill="isStarred ? 'currentColor' : 'none'" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :class="{ 'text-primary': isArchived }"
            :title="t('mail.archive')"
            @click="handleArchive"
          >
            <Archive class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :class="{ 'text-danger': isSpam }"
            :title="isSpam ? t('mail.notSpam') : t('mail.markAsSpam')"
            @click="handleToggleSpam"
          >
            <ShieldCheck v-if="isSpam" class="h-4 w-4" />
            <AlertTriangle v-else class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="t('mail.reply')"
            @click="handleReply"
          >
            <Reply class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="t('mail.replyAll')"
            @click="handleReplyAll"
          >
            <ReplyAll class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="t('mail.forward')"
            @click="handleForward"
          >
            <Forward class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="mailStore.isReadingMode ? t('mail.exitReadingMode') : t('mail.readingMode')"
            @click="mailStore.toggleReadingMode()"
          >
            <Minimize2 v-if="mailStore.isReadingMode" class="h-4 w-4" />
            <Expand v-else class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised hover:text-danger"
            :title="t('mail.delete')"
            @click="handleDelete"
          >
            <Trash2 class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="t('translation.translate')"
            @click="showTranslatePanel = !showTranslatePanel"
          >
            <Languages class="h-4 w-4" />
          </button>
          <button
            type="button"
            class="flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised"
            :title="t('aiAssistant.title')"
            @click="aiStore.togglePanel()"
          >
            <Sparkles class="h-4 w-4" />
          </button>
        </div>
      </div>

      <!-- Translation panel -->
      <div v-if="showTranslatePanel" class="border-b border-border px-4 py-2">
        <TranslatePanel :mail-id="currentMailId" @translated="handleTranslated" />
      </div>

      <!-- Translation result card -->
      <div v-if="translatedText || isTranslating" class="border-b border-border bg-accent-subtle/30 px-6 py-3">
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <Languages class="h-4 w-4 text-accent" aria-hidden="true" />
            <span class="text-xs font-medium text-accent">
              {{ t('translation.translatedTo', { lang: translatedLang ?? 'target' }) }}
            </span>
          </div>
          <div class="flex items-center gap-1">
            <button
              type="button"
              class="rounded p-1 text-tertiary transition-colors hover:bg-raised hover:text-primary"
              :title="translationCollapsed ? t('translation.expand') : t('translation.collapse')"
              @click="toggleTranslationCollapsed"
            >
              <ChevronDown v-if="translationCollapsed" class="h-3.5 w-3.5" />
              <ChevronUp v-else class="h-3.5 w-3.5" />
            </button>
            <button
              type="button"
              class="rounded p-1 text-tertiary transition-colors hover:bg-raised hover:text-primary"
              :title="t('mail.close')"
              @click="clearTranslation"
            >
              <X class="h-3.5 w-3.5" />
            </button>
          </div>
        </div>
        <div
          v-show="!translationCollapsed"
          class="mt-2 max-h-96 overflow-y-auto text-sm text-secondary"
        >
          <div v-if="isTranslating" class="flex items-center gap-2 text-xs text-tertiary">
            <Loader2 class="h-3.5 w-3.5 animate-spin" />
            {{ t('translation.translating') }}
          </div>
          <p v-else>{{ translatedText }}</p>
        </div>
      </div>

      <!-- Mail header -->
      <div class="border-b border-border px-6 py-4">
        <h1 class="text-xl font-semibold text-primary">
          {{ mail.subject || t('mail.noSubject') }}
        </h1>
        <div class="mt-2 flex items-center gap-4 text-sm">
          <div class="flex items-center gap-2">
            <div
              class="flex h-8 w-8 items-center justify-center rounded-full bg-accent-subtle text-sm font-medium text-accent"
            >
              {{ (mail.fromName || mail.fromAddress || '?').charAt(0).toUpperCase() }}
            </div>
            <div>
              <div class="font-medium text-primary">
                {{ mail.fromName || mail.fromAddress || t('mail.unknownSender') }}
              </div>
              <div class="text-xs text-tertiary">
                {{ mail.fromAddress }}
              </div>
            </div>
          </div>
          <div class="ml-auto text-xs text-tertiary">
            {{ formatDate(mail.date) }}
          </div>
        </div>
        <div v-if="mail.toAddresses" class="mt-2 text-xs text-tertiary">
          {{ t('mail.to') }}: {{ formatAddresses(mail.toAddresses) }}
        </div>
        <div v-if="mail.ccAddresses" class="mt-1 text-xs text-tertiary">
          {{ t('mail.cc') }}: {{ formatAddresses(mail.ccAddresses) }}
        </div>
      </div>

      <!-- AI summary -->
      <div
        v-if="isSummarizing || summaryText"
        class="border-b border-border bg-accent-subtle/30 px-6 py-3"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-center gap-2">
            <Sparkles class="h-4 w-4 text-accent" aria-hidden="true" />
            <span class="text-xs font-medium text-accent">{{ t('aiActions.summaryTitle') }}</span>
          </div>
          <button
            v-if="summaryText"
            type="button"
            class="rounded p-1 text-tertiary transition-colors hover:bg-raised hover:text-primary"
            :title="t('mail.close')"
            @click="closeSummary"
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>
        <div class="mt-1 text-sm text-secondary">
          <div v-if="isSummarizing" class="flex items-center gap-2 text-xs text-tertiary">
            <Loader2 class="h-3.5 w-3.5 animate-spin" />
            {{ t('aiActions.summarizing') }}
          </div>
          <p v-else>{{ summaryText }}</p>
        </div>
      </div>

      <!-- Remote content security banner -->
      <div v-if="showSecurityBanner" class="border-b border-border bg-warning/10 px-6 py-3">
        <div class="flex items-start gap-3">
          <AlertTriangle class="mt-0.5 h-4 w-4 shrink-0 text-warning" aria-hidden="true" />
          <div class="flex-1">
            <p class="text-sm font-medium text-warning">
              {{ t('mail.remoteContentBlocked') }}
            </p>
            <p class="text-xs text-secondary">
              {{ t('mail.remoteContentHint') }}
            </p>
            <div class="mt-2 flex flex-wrap items-center gap-2">
              <button
                type="button"
                class="rounded px-2 py-1 text-xs font-medium text-warning transition-colors hover:bg-warning/10"
                @click="allowRemoteOnce"
              >
                {{ t('mail.allowOnce') }}
              </button>
              <button
                type="button"
                class="rounded px-2 py-1 text-xs font-medium text-warning transition-colors hover:bg-warning/10"
                @click="allowAllRemoteOnce"
              >
                {{ t('mail.allowAllOnce') }}
              </button>
              <button
                v-for="domain in untrustedDomains"
                :key="domain"
                type="button"
                class="rounded px-2 py-1 text-xs text-secondary transition-colors hover:bg-raised"
                @click="trustDomain(domain)"
              >
                {{ t('mail.alwaysTrustDomain', { domain }) }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Selected-text translate popup -->
      <Teleport to="body">
        <div
          v-if="showSelectionMenu"
          ref="selectionMenuRef"
          class="fixed z-50 flex items-center gap-1 rounded-lg border border-border bg-elevated p-1 shadow-md"
          :style="{ left: `${selectionMenuX}px`, top: `${selectionMenuY}px` }"
        >
          <div class="w-24">
            <BaseSelect
              v-model="translationTargetLang"
              size="sm"
              :options="translationLanguages.map((lang) => ({ value: lang.value, label: $t(lang.label) }))"
            />
          </div>
          <button
            type="button"
            class="flex h-7 w-7 items-center justify-center rounded-md bg-accent text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
            :title="t('translation.translate')"
            :disabled="!selectedTranslationProviderId || isTranslating"
            @click="translateSelectedText"
          >
            <Loader2 v-if="isTranslating" class="h-3.5 w-3.5 animate-spin" />
            <Languages v-else class="h-3.5 w-3.5" />
          </button>
        </div>
      </Teleport>

      <!-- Mail body -->
      <div class="flex-1 overflow-y-auto px-6 py-4">
        <SandboxedHtml
          v-if="processedBodyHtml"
          ref="sandboxedHtmlRef"
          :key="`${currentMailId}-${cspRevision}`"
          :html="processedBodyHtml"
          :allowed-domains="allowedDomains"
          class="prose prose-sm max-w-none text-primary"
          @security-violation="handleSecurityViolation"
          @remote-domains="handleRemoteDomains"
          @selection="handleIframeSelection"
          @link-click="handleLinkClick"
        />
        <div v-else-if="mail.bodyText" class="whitespace-pre-wrap text-sm text-primary">
          {{ mail.bodyText }}
        </div>
        <div v-else class="text-sm italic text-secondary">
          {{ t('mail.noContent') }}
        </div>
      </div>

      <!-- Attachments -->
      <div v-if="attachments.length > 0" class="border-t border-border px-6 py-3">
        <h3 class="mb-2 text-sm font-medium text-primary">
          {{ t('mail.attachments') }} ({{ attachments.length }})
        </h3>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="att in attachments"
            :key="att.id"
            type="button"
            class="flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm transition-colors hover:bg-raised focus:outline-none focus:ring-2 focus:ring-accent"
            @click="downloadAttachment(att)"
          >
            <Paperclip class="h-4 w-4 text-tertiary" />
            <span class="text-primary">{{ att.filename }}</span>
            <span class="text-xs text-tertiary">{{ formatSize(att.size) }}</span>
          </button>
        </div>
      </div>
    </template>

    <!-- Loading state -->
    <div v-else-if="currentMailId" class="flex flex-1 items-center justify-center text-secondary">
      <div class="flex items-center gap-2">
        <div
          class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent"
        />
        {{ t('mail.loading') }}
      </div>
    </div>

    <!-- Delete confirmation dialog -->
    <Teleport to="body">
      <div
        v-if="showDeleteConfirm"
        class="fixed inset-0 z-50 flex items-center justify-center bg-overlay"
        @click="cancelDelete"
      >
        <div
          ref="deleteDialogRef"
          role="alertdialog"
          aria-modal="true"
          aria-labelledby="delete-title"
          aria-describedby="delete-message"
          class="w-80 rounded-lg bg-elevated p-4 shadow-lg"
          tabindex="-1"
          @keydown="handleDeleteKeyDown"
          @click.stop
        >
          <h3 id="delete-title" class="text-lg font-medium text-primary">
            {{ t('mail.deleteConfirmTitle') }}
          </h3>
          <p id="delete-message" class="mt-2 text-sm text-secondary">
            {{ t('mail.deleteConfirmMessage') }}
          </p>
          <div class="mt-4 flex justify-end gap-2">
            <button
              type="button"
              class="flex h-9 items-center justify-center rounded-md border border-border px-3 text-sm text-secondary transition-colors hover:bg-raised"
              @click="cancelDelete"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              type="button"
              class="flex h-9 items-center justify-center rounded-md bg-danger px-3 text-sm text-white transition-colors hover:bg-danger-hover disabled:opacity-50"
              :disabled="currentMailId ? mailStore.deletingMailIds.has(currentMailId) : false"
              @click="confirmDelete"
            >
              <Loader2
                v-if="currentMailId && mailStore.deletingMailIds.has(currentMailId)"
                class="mr-1.5 h-4 w-4 animate-spin"
              />
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
