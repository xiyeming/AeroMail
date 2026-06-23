<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import {
  Languages,
  Star,
  Trash2,
  Expand,
  Minimize2,
  Mail,
  Paperclip,
  Reply,
  ReplyAll,
  Forward,
  Archive,
  AlertTriangle,
  ShieldCheck,
  Sparkles,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import AiQuickActions from '@/components/AiQuickActions.vue';
import SandboxedHtml from '@/components/SandboxedHtml.vue';
import TranslatePanel from '@/components/TranslatePanel.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import { useToastStore } from '@/stores/toast';
import { useSettingsStore } from '@/stores/settings';
import type { AttachmentInfo } from '@/types/mail';

const { t } = useI18n();
const router = useRouter();
const { createSession, sendMessage } = useAiChat();
const aiStore = useAiStore();
const mailStore = useMailStore();
const toast = useToastStore();
const settingsStore = useSettingsStore();

const translatedText = ref<string | null>(null);
const translatedLang = ref<string | null>(null);
const showTranslation = ref(false);
const showTranslatePanel = ref(false);
const showDeleteConfirm = ref(false);
const attachments = ref<AttachmentInfo[]>([]);
const deleteDialogRef = ref<HTMLDivElement | null>(null);
const trustedDomains = ref<string[]>([]);
const temporarilyAllowedDomains = ref<string[]>([]);

const currentMailId = computed(() => mailStore.selectedMailId);
const mail = computed(() => mailStore.selectedMail);
const isStarred = computed(() => mail.value?.isStarred ?? false);
const isSpam = computed(() => mail.value?.isSpam ?? false);
const isArchived = computed(() => mail.value?.isArchived ?? false);

async function loadAttachments(mailId: string) {
  try {
    attachments.value = await invoke<AttachmentInfo[]>('get_attachments', { mailId });
  } catch (e) {
    console.error('Failed to load attachments:', e);
  }
}

async function downloadAttachment(att: AttachmentInfo) {
  try {
    const bytes = await invoke<number[]>('get_attachment_content', { attachmentId: att.id });
    const uint8 = new Uint8Array(bytes);
    const blob = new Blob([uint8], { type: att.mimeType || 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = att.filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (e) {
    console.error('Failed to download attachment:', e);
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const PROMPTS = {
  summarize: 'Please summarize this email in 3 sentences.',
  reply: 'Please write a polite reply draft for this email. Keep it concise.',
  extractTodos: 'Please extract all action items from this email, listed by priority.',
};

async function handleQuickAction(promptKey: keyof typeof PROMPTS) {
  if (!currentMailId.value) return;
  if (!aiStore.isPanelOpen) aiStore.togglePanel();
  const provider = aiStore.providers[0];
  if (!provider) return;
  try {
    const session = await createSession(provider.id, currentMailId.value);
    await sendMessage(session.id, PROMPTS[promptKey]);
  } catch (e) {
    toast.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
      duration: 5000,
    });
  }
}

function handleTranslated(payload: { text: string; lang: string }) {
  translatedText.value = payload.text;
  translatedLang.value = payload.lang;
  showTranslation.value = true;
  showTranslatePanel.value = false;
}

function toggleTranslation() {
  showTranslation.value = !showTranslation.value;
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

function confirmDelete() {
  if (currentMailId.value) {
    mailStore.deleteMail(currentMailId.value);
    showDeleteConfirm.value = false;
  }
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

  // src / href / background attributes (support protocol-relative URLs)
  const attrRe = /(?:src|href|background)\s*=\s*["']((?:https?:)?\/\/[^"']+)["']/gi;
  let match: RegExpExecArray | null;
  while ((match = attrRe.exec(html)) !== null) {
    capture(match[1]);
  }

  // CSS url(...)
  const urlRe = /url\((['"]?)((?:https?:)?\/\/[^"')]+)\1\)/gi;
  while ((match = urlRe.exec(html)) !== null) {
    capture(match[2]);
  }

  // @import url(...)
  const importRe = /@import\s+(?:url\()?["']((?:https?:)?\/\/[^"')]+)["']\)?/gi;
  while ((match = importRe.exec(html)) !== null) {
    capture(match[1]);
  }

  // srcset attribute (comma-separated url descriptors, supports protocol-relative)
  const srcsetRe = /srcset\s*=\s*["']([^"']+)["']/gi;
  while ((match = srcsetRe.exec(html)) !== null) {
    for (const part of match[1].split(',')) {
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
  if (!mail.value?.bodyHtml) return [];
  return extractRemoteDomains(mail.value.bodyHtml);
});

const allowedDomains = computed(() => [
  ...trustedDomains.value,
  ...temporarilyAllowedDomains.value,
]);

const untrustedDomains = computed(() =>
  remoteDomains.value.filter(
    (d) => !allowedDomains.value.includes(d)
  )
);

const showSecurityBanner = computed(() => untrustedDomains.value.length > 0);

function allowRemoteOnce() {
  temporarilyAllowedDomains.value = [...untrustedDomains.value];
}

async function trustDomain(domain: string) {
  if (!trustedDomains.value.includes(domain)) {
    trustedDomains.value.push(domain);
    await settingsStore.set('trustedDomains', JSON.stringify(trustedDomains.value));
  }
}

async function loadTrustedDomains() {
  try {
    const raw = await settingsStore.get('trustedDomains');
    if (raw) {
      trustedDomains.value = JSON.parse(raw);
    }
  } catch (e) {
    console.error('Failed to load trusted domains:', e);
  }
}

onMounted(() => {
  void loadTrustedDomains();
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
  showTranslation.value = false;
  showTranslatePanel.value = false;
  showDeleteConfirm.value = false;
  temporarilyAllowedDomains.value = [];
  attachments.value = [];
  if (newMailId && mail.value?.hasAttachments) {
    loadAttachments(newMailId);
  }
});
</script>

<template>
  <div class="flex h-full flex-col bg-base">
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
          @summarize="handleQuickAction('summarize')"
          @reply="handleQuickAction('reply')"
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

      <!-- Translation display -->
      <div
        v-if="translatedText && showTranslation"
        class="flex h-9 items-center justify-between border-b border-border bg-accent-subtle px-4"
      >
        <span class="text-xs text-accent">
          {{ t('translation.translatedTo', { lang: translatedLang ?? 'target' }) }}
        </span>
        <button
          class="text-xs text-accent underline hover:text-accent-hover"
          @click="toggleTranslation"
        >
          {{ t('translation.showOriginal') }}
        </button>
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

      <!-- Remote content security banner -->
      <div
        v-if="showSecurityBanner"
        class="border-b border-border bg-warning/10 px-6 py-3"
      >
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

      <!-- Mail body -->
      <div class="flex-1 overflow-y-auto px-6 py-4">
        <div
          v-if="translatedText && !showTranslation"
          class="mb-4 rounded-lg bg-raised p-4 text-sm text-secondary"
        >
          {{ translatedText }}
          <button
            class="ml-2 text-xs text-accent underline hover:text-accent-hover"
            @click="toggleTranslation"
          >
            {{ t('translation.showOriginal') }}
          </button>
        </div>

        <SandboxedHtml
          v-if="mail.bodyHtml"
          :key="`${currentMailId}-${allowedDomains.join(',')}`"
          :html="mail.bodyHtml"
          :allowed-domains="allowedDomains"
          class="prose prose-sm max-w-none text-primary"
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
              class="flex h-9 items-center justify-center rounded-md bg-danger px-3 text-sm text-white transition-colors hover:bg-danger-hover"
              @click="confirmDelete"
            >
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
