<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import {
  Languages,
  Star,
  Trash2,
  Maximize2,
  Minimize2,
  Mail,
  Paperclip,
  Reply,
  ReplyAll,
  Forward,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import AiQuickActions from '@/components/AiQuickActions.vue';
import SandboxedHtml from '@/components/SandboxedHtml.vue';
import TranslatePanel from '@/components/TranslatePanel.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import type { AttachmentInfo } from '@/types/mail';

const { t } = useI18n();
const router = useRouter();
const { createSession, sendMessage } = useAiChat();
const aiStore = useAiStore();
const mailStore = useMailStore();

const translatedText = ref<string | null>(null);
const showTranslation = ref(false);
const showTranslatePanel = ref(false);
const showDeleteConfirm = ref(false);
const attachments = ref<AttachmentInfo[]>([]);

const currentMailId = computed(() => mailStore.selectedMailId);
const mail = computed(() => mailStore.selectedMail);
const isStarred = computed(() => mail.value?.isStarred ?? false);

async function loadAttachments(mailId: string) {
  try {
    attachments.value = await invoke<AttachmentInfo[]>('get_attachments', { mailId });
  } catch (e) {
    console.error('Failed to load attachments:', e);
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
  const session = await createSession(provider.id, currentMailId.value);
  await sendMessage(session.id, PROMPTS[promptKey]);
}

function handleTranslated(text: string) {
  translatedText.value = text;
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

// Reset translation and load attachments when mail changes
watch(currentMailId, (newMailId) => {
  translatedText.value = null;
  showTranslation.value = false;
  showTranslatePanel.value = false;
  showDeleteConfirm.value = false;
  attachments.value = [];
  if (newMailId) {
    loadAttachments(newMailId);
  }
});
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <div
      v-if="!currentMailId"
      class="flex flex-1 flex-col items-center justify-center gap-4 text-muted"
    >
      <Mail class="h-16 w-16 opacity-20" />
      <div class="text-center">
        <p class="text-lg font-medium">{{ $t('mail.selectEmail') }}</p>
        <p class="mt-1 text-sm">{{ $t('mail.selectEmailHint') }}</p>
      </div>
      <div class="mt-4 flex gap-4 text-xs text-muted/60">
        <span
          ><kbd class="rounded bg-card px-1.5 py-0.5">J</kbd> /
          <kbd class="rounded bg-card px-1.5 py-0.5">K</kbd> {{ $t('mail.navigate') }}</span
        >
        <span><kbd class="rounded bg-card px-1.5 py-0.5">Enter</kbd> {{ $t('mail.open') }}</span>
        <span><kbd class="rounded bg-card px-1.5 py-0.5">Esc</kbd> {{ $t('mail.close') }}</span>
      </div>
    </div>
    <template v-else-if="mail">
      <!-- Toolbar -->
      <div class="flex items-center justify-between border-b border-border px-4 py-2">
        <AiQuickActions
          @summarize="handleQuickAction('summarize')"
          @reply="handleQuickAction('reply')"
          @extract-todos="handleQuickAction('extractTodos')"
        />
        <div class="flex items-center gap-1">
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :class="{ 'text-yellow-500': isStarred }"
            :title="t('mail.star')"
            @click="handleToggleStar"
          >
            <Star class="h-3.5 w-3.5" :fill="isStarred ? 'currentColor' : 'none'" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :title="t('mail.reply')"
            @click="handleReply"
          >
            <Reply class="h-3.5 w-3.5" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :title="t('mail.replyAll')"
            @click="handleReplyAll"
          >
            <ReplyAll class="h-3.5 w-3.5" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :title="t('mail.forward')"
            @click="handleForward"
          >
            <Forward class="h-3.5 w-3.5" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :title="mailStore.isReadingMode ? t('mail.exitReadingMode') : t('mail.readingMode')"
            @click="mailStore.toggleReadingMode()"
          >
            <Minimize2 v-if="mailStore.isReadingMode" class="h-3.5 w-3.5" />
            <Maximize2 v-else class="h-3.5 w-3.5" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card hover:text-red-500"
            :title="t('mail.delete')"
            @click="handleDelete"
          >
            <Trash2 class="h-3.5 w-3.5" />
          </button>
          <button
            class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
            :title="t('translation.translate')"
            @click="showTranslatePanel = !showTranslatePanel"
          >
            <Languages class="h-3.5 w-3.5" />
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
        class="flex items-center justify-between border-b border-border bg-primary/5 px-4 py-1.5"
      >
        <span class="text-xs text-primary">
          {{ t('translation.translatedTo', { lang: 'target' }) }}
        </span>
        <button
          class="text-xs text-primary underline hover:text-primary-hover"
          @click="toggleTranslation"
        >
          {{ t('translation.showOriginal') }}
        </button>
      </div>

      <!-- Mail header -->
      <div class="border-b border-border px-6 py-4">
        <h1 class="text-xl font-semibold text-text">
          {{ mail.subject || t('mail.noSubject') }}
        </h1>
        <div class="mt-2 flex items-center gap-4 text-sm text-text-secondary">
          <div class="flex items-center gap-2">
            <div
              class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-medium text-primary"
            >
              {{ (mail.fromName || mail.fromAddress || '?').charAt(0).toUpperCase() }}
            </div>
            <div>
              <div class="font-medium text-text">
                {{ mail.fromName || mail.fromAddress || t('mail.unknownSender') }}
              </div>
              <div class="text-xs text-muted">
                {{ mail.fromAddress }}
              </div>
            </div>
          </div>
          <div class="text-xs text-muted">
            {{ formatDate(mail.date) }}
          </div>
        </div>
        <div v-if="mail.toAddresses" class="mt-2 text-xs text-muted">
          {{ t('mail.to') }}: {{ formatAddresses(mail.toAddresses) }}
        </div>
        <div v-if="mail.ccAddresses" class="mt-1 text-xs text-muted">
          {{ t('mail.cc') }}: {{ formatAddresses(mail.ccAddresses) }}
        </div>
      </div>

      <!-- Mail body -->
      <div class="flex-1 overflow-y-auto px-6 py-4">
        <div
          v-if="translatedText && !showTranslation"
          class="mb-4 rounded-lg bg-muted/10 p-4 text-sm text-text-secondary"
        >
          {{ translatedText }}
          <button
            class="ml-2 text-xs text-primary underline hover:text-primary-hover"
            @click="toggleTranslation"
          >
            {{ t('translation.showOriginal') }}
          </button>
        </div>

        <SandboxedHtml
          v-if="mail.bodyHtml"
          :html="mail.bodyHtml"
          class="prose prose-sm max-w-none text-text"
        />
        <div v-else-if="mail.bodyText" class="whitespace-pre-wrap text-sm text-text">
          {{ mail.bodyText }}
        </div>
        <div v-else class="text-sm text-muted italic">
          {{ t('mail.noContent') }}
        </div>
      </div>

      <!-- Attachments -->
      <div v-if="attachments.length > 0" class="border-t border-border px-6 py-3">
        <h3 class="mb-2 text-sm font-medium text-text">
          {{ t('mail.attachments') }} ({{ attachments.length }})
        </h3>
        <div class="flex flex-wrap gap-2">
          <div
            v-for="att in attachments"
            :key="att.id"
            class="flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-card"
          >
            <Paperclip class="h-4 w-4 text-muted" />
            <span class="text-text">{{ att.filename }}</span>
            <span class="text-xs text-muted">{{ formatSize(att.size) }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- Loading state -->
    <div v-else-if="currentMailId" class="flex flex-1 items-center justify-center text-muted">
      <div class="flex items-center gap-2">
        <div
          class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
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
        <div class="w-80 rounded-lg bg-panel p-4 shadow-lg" @click.stop>
          <h3 class="text-lg font-medium text-text">{{ t('mail.deleteConfirmTitle') }}</h3>
          <p class="mt-2 text-sm text-text-secondary">{{ t('mail.deleteConfirmMessage') }}</p>
          <div class="mt-4 flex justify-end gap-2">
            <button
              class="rounded-md border border-border px-3 py-1.5 text-sm text-text-secondary hover:bg-card"
              @click="cancelDelete"
            >
              {{ t('common.cancel') }}
            </button>
            <button
              class="rounded-md bg-red-500 px-3 py-1.5 text-sm text-white hover:bg-red-600"
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
