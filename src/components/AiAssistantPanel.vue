<script setup lang="ts">
import { ref, onMounted, nextTick, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, Eraser, Pencil, Trash2, X } from '@lucide/vue';
import { useAiChat } from '@/composables/useAiChat';
import AiMessageList from '@/components/AiMessageList.vue';
import { useAiStore } from '@/stores/ai';
import type { MailDetail, MailHeader } from '@/types/mail';

const { t } = useI18n();
const aiStore = useAiStore();
const {
  sessions,
  messages,
  isLoading,
  error,
  activeSessionId,
  loadSessions,
  createSession,
  sendMessage,
  selectSession,
  deleteSession,
  clearSession,
  renameSession,
  setSessionProvider,
} = useAiChat();

const input = ref('');
const messagesContainer = ref<HTMLElement>();
const sessionMenuOpen = ref(false);
const providerMenuOpen = ref(false);
const renamingSessionId = ref<string | null>(null);
const renameDraft = ref('');

const currentSession = computed(() => sessions.value.find((s) => s.id === activeSessionId.value));

const currentSessionTitle = computed(() => currentSession.value?.title ?? '');

const currentProvider = computed(() =>
  aiStore.providers.find((p) => p.id === currentSession.value?.providerId)
);

// --- @ mention mail quoting ---
const inputRef = ref<HTMLTextAreaElement>();
const mention = ref<{ active: boolean; query: string; startIndex: number } | null>(null);
const mentionResults = ref<MailHeader[]>([]);
const mentionIndex = ref(0);
const mentionedMails = ref<{ id: string; subject: string }[]>([]);
let mentionTimeout: number | null = null;

function getMentionAtCursor(): { startIndex: number; query: string } | null {
  const el = inputRef.value;
  if (!el) return null;
  const cursor = el.selectionStart ?? 0;
  const text = input.value;
  let atIndex = -1;
  for (let i = cursor - 1; i >= 0; i--) {
    const ch = text[i];
    if (ch === '@') {
      atIndex = i;
      break;
    }
    if (/\s/.test(ch)) break;
  }
  if (atIndex === -1) return null;
  if (atIndex > 0 && !/\s/.test(text[atIndex - 1])) return null;
  return { startIndex: atIndex, query: text.slice(atIndex + 1, cursor) };
}

async function searchMentions(query: string) {
  try {
    const results = await invoke<MailHeader[]>('search_mail_summaries', { query });
    mentionResults.value = results;
    mentionIndex.value = 0;
  } catch (e) {
    console.error('Failed to search mail mentions:', e);
    mentionResults.value = [];
  }
}

function handleMentionInput() {
  const m = getMentionAtCursor();
  if (m) {
    mention.value = { active: true, query: m.query, startIndex: m.startIndex };
    if (mentionTimeout) window.clearTimeout(mentionTimeout);
    mentionTimeout = window.setTimeout(() => {
      void searchMentions(m.query);
    }, 100);
  } else {
    mention.value = null;
    mentionResults.value = [];
  }
}

function selectMention(index: number) {
  const mail = mentionResults.value[index];
  if (!mail || !mention.value) return;
  const el = inputRef.value;
  if (!el) return;
  const cursor = el.selectionStart ?? 0;
  const before = input.value.slice(0, mention.value.startIndex);
  const after = input.value.slice(cursor);
  const subject = mail.subject || t('mail.noSubject');
  input.value = `${before}@${subject} ${after}`;
  if (!mentionedMails.value.some((m) => m.id === mail.id)) {
    mentionedMails.value.push({ id: mail.id, subject });
  }
  mention.value = null;
  mentionResults.value = [];
  void nextTick(() => {
    const pos = before.length + 1 + subject.length + 1;
    el.setSelectionRange(pos, pos);
    el.focus();
  });
}

function handleMentionKeydown(event: KeyboardEvent) {
  if (!mention.value?.active) return;
  if (event.key === 'Escape') {
    mention.value = null;
    mentionResults.value = [];
    return;
  }
  if (event.key === 'Enter' || event.key === 'Tab') {
    if (mentionResults.value.length > 0) {
      event.preventDefault();
      selectMention(mentionIndex.value);
    } else {
      mention.value = null;
      mentionResults.value = [];
    }
    return;
  }
  if (mentionResults.value.length === 0) return;
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    mentionIndex.value = (mentionIndex.value + 1) % mentionResults.value.length;
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    mentionIndex.value =
      (mentionIndex.value - 1 + mentionResults.value.length) % mentionResults.value.length;
  }
}

function handleInputKeydown(event: KeyboardEvent) {
  handleMentionKeydown(event);
  if (event.defaultPrevented) return;
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    void handleSend();
  }
}

async function buildContentWithMentions(): Promise<string> {
  if (mentionedMails.value.length === 0) return input.value;
  const quotes: string[] = [];
  for (const mail of mentionedMails.value) {
    try {
      const detail = await invoke<MailDetail>('get_mail_detail', { mailId: mail.id });
      const body = detail.bodyText || detail.bodyHtml || '';
      const from = detail.fromName || detail.fromAddress || t('mail.unknownSender');
      quotes.push(`[${t('mail.quotedToAi')}]
${t('mail.subject')}: ${mail.subject}
${t('mail.from')}: ${from}

${body}`);
    } catch (e) {
      console.error('Failed to load quoted mail:', e);
    }
  }
  return quotes.length > 0 ? `${input.value}\n\n${quotes.join('\n\n---\n\n')}` : input.value;
}

async function ensureSession() {
  if (activeSessionId.value) return;
  const providerId = aiStore.resolveProviderId();
  if (!providerId) return;
  await createSession(providerId);
}

onMounted(() => {
  void (async () => {
    await loadSessions();
    await aiStore.loadProviders();
    await aiStore.loadDefaultProvider();
    if (aiStore.isPanelOpen) {
      await ensureSession();
    }
  })();
});

watch(
  () => aiStore.isPanelOpen,
  async (open) => {
    if (open) {
      await ensureSession();
    }
  }
);

async function handleSend() {
  if (!input.value.trim() || !activeSessionId.value) return;
  const content = await buildContentWithMentions();
  input.value = '';
  mentionedMails.value = [];
  try {
    await sendMessage(activeSessionId.value, content);
  } catch {
    // error is surfaced via useAiChat.error
  }
  await nextTick();
  scrollToBottom();
}

function clearError() {
  error.value = null;
}

function scrollToBottom() {
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

watch(
  messages,
  () => {
    void nextTick(scrollToBottom);
  },
  { deep: true }
);

async function handleSelectSession(id: string) {
  selectSession(id);
  sessionMenuOpen.value = false;
}

async function handleSetProvider(providerId: string) {
  if (!activeSessionId.value) return;
  await setSessionProvider(activeSessionId.value, providerId);
  providerMenuOpen.value = false;
}

async function handleDeleteSession(id: string) {
  await deleteSession(id);
  if (activeSessionId.value === id) {
    sessionMenuOpen.value = false;
  }
}

async function handleClearSession(id: string) {
  await clearSession(id);
}

function startRename(session: { id: string; title: string | null }) {
  renamingSessionId.value = session.id;
  renameDraft.value = session.title ?? '';
}

function cancelRename() {
  renamingSessionId.value = null;
  renameDraft.value = '';
}

async function commitRename(sessionId: string) {
  const title = renameDraft.value.trim();
  if (title) {
    await renameSession(sessionId, title);
  }
  cancelRename();
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-y-0 right-0 z-50 flex w-80 transform flex-col border-l border-border bg-elevated shadow-lg transition-transform duration-200 ease-out"
      :class="aiStore.isPanelOpen ? 'translate-x-0' : 'translate-x-full'"
    >
      <div class="flex h-12 items-center justify-between border-b border-border px-3">
        <div class="relative flex min-w-0 items-center">
          <button
            type="button"
            class="flex min-w-0 items-center gap-1 rounded-md px-1 py-1 text-sm font-medium text-primary transition-colors hover:bg-raised"
            :aria-label="$t('aiAssistant.selectSession')"
            @click="sessionMenuOpen = !sessionMenuOpen"
          >
            <span class="truncate">
              {{ currentSessionTitle || $t('aiAssistant.title') }}
            </span>
            <ChevronDown class="h-3.5 w-3.5 shrink-0 text-secondary" />
          </button>

          <div
            v-if="sessionMenuOpen"
            class="absolute left-0 top-full z-20 mt-1 w-64 rounded-md border border-border bg-elevated py-1 shadow-lg"
          >
            <div class="max-h-56 overflow-y-auto">
              <div
                v-for="s in sessions"
                :key="s.id"
                class="group flex items-center gap-1 px-2 py-1.5 hover:bg-raised"
              >
                <button
                  type="button"
                  class="min-w-0 flex-1 text-left text-xs text-primary"
                  @click="handleSelectSession(s.id)"
                >
                  <span v-if="renamingSessionId === s.id" class="block">
                    <input
                      v-model="renameDraft"
                      type="text"
                      class="w-full rounded border border-border bg-base px-1.5 py-0.5 text-xs text-primary outline-none focus:border-accent"
                      @keydown.enter.prevent="commitRename(s.id)"
                      @keydown.escape.prevent="cancelRename"
                      @blur="commitRename(s.id)"
                    />
                  </span>
                  <span v-else class="block truncate">
                    {{ s.title || $t('aiAssistant.unnamedSession') }}
                  </span>
                </button>

                <div class="flex shrink-0 items-center gap-0.5 opacity-60 group-hover:opacity-100">
                  <button
                    type="button"
                    class="rounded p-1 text-secondary transition-colors hover:text-primary"
                    :title="$t('aiAssistant.renameSession')"
                    @click="startRename(s)"
                  >
                    <Pencil class="h-3 w-3" />
                  </button>
                  <button
                    type="button"
                    class="rounded p-1 text-secondary transition-colors hover:text-primary"
                    :title="$t('aiAssistant.clearSession')"
                    @click="handleClearSession(s.id)"
                  >
                    <Eraser class="h-3 w-3" />
                  </button>
                  <button
                    type="button"
                    class="rounded p-1 text-secondary transition-colors hover:text-danger"
                    :title="$t('aiAssistant.deleteSession')"
                    @click="handleDeleteSession(s.id)"
                  >
                    <Trash2 class="h-3 w-3" />
                  </button>
                </div>
              </div>

              <div v-if="sessions.length === 0" class="px-3 py-2 text-xs text-tertiary">
                {{ $t('aiAssistant.noSessions') }}
              </div>
            </div>
          </div>

          <div v-if="activeSessionId" class="relative ml-2">
            <button
              type="button"
              class="flex min-w-0 items-center gap-1 rounded-md px-1 py-1 text-xs text-secondary transition-colors hover:bg-raised hover:text-primary"
              @click="providerMenuOpen = !providerMenuOpen"
            >
              <span class="truncate">{{ currentProvider?.name || currentProvider?.kind }}</span>
              <ChevronDown class="h-3 w-3 shrink-0" />
            </button>

            <div
              v-if="providerMenuOpen"
              class="absolute left-0 top-full z-20 mt-1 w-48 rounded-md border border-border bg-elevated py-1 shadow-lg"
            >
              <button
                v-for="provider in aiStore.providers"
                :key="provider.id"
                type="button"
                class="block w-full truncate px-2 py-1.5 text-left text-xs text-primary transition-colors hover:bg-raised"
                :class="provider.id === currentSession?.providerId ? 'text-accent' : ''"
                @click="handleSetProvider(provider.id)"
              >
                {{ provider.name }}
              </button>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-0.5">
          <button
            type="button"
            class="rounded-md p-1.5 text-secondary transition-colors hover:bg-raised hover:text-primary"
            :aria-label="$t('common.close')"
            @click="aiStore.togglePanel()"
          >
            <X class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div
        v-if="!activeSessionId"
        class="flex flex-1 items-center justify-center p-4 text-center text-sm text-secondary"
      >
        <span v-if="aiStore.resolveProviderId()">{{ $t('common.loading') }}</span>
        <span v-else>{{ $t('aiAssistant.selectProvider') }}</span>
      </div>

      <template v-else>
        <div ref="messagesContainer" class="flex-1 overflow-y-auto p-3">
          <AiMessageList :messages="messages" :is-loading="isLoading" />
        </div>
        <div class="border-t border-border p-3">
          <div
            v-if="error"
            class="mb-2 rounded-md bg-danger-subtle px-3 py-2 text-xs text-danger"
            role="alert"
          >
            <div class="flex items-start justify-between gap-2">
              <span class="break-words">{{ error }}</span>
              <button
                type="button"
                class="shrink-0 text-danger underline hover:text-danger-hover"
                @click="clearError"
              >
                {{ $t('common.close') }}
              </button>
            </div>
          </div>
          <div class="relative">
            <textarea
              ref="inputRef"
              v-model="input"
              class="w-full resize-none rounded-md border border-border bg-base p-2.5 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
              :rows="3"
              :placeholder="$t('aiAssistant.inputPlaceholder')"
              @input="
                clearError();
                handleMentionInput();
              "
              @keydown="handleInputKeydown"
            />
            <div
              v-if="mention?.active"
              class="absolute bottom-full left-0 z-30 mb-1 max-h-48 w-full overflow-y-auto rounded-md border border-border bg-elevated py-1 shadow-lg"
            >
              <template v-if="mentionResults.length > 0">
                <button
                  v-for="(mail, index) in mentionResults"
                  :key="mail.id"
                  type="button"
                  class="w-full px-3 py-2 text-left transition-colors hover:bg-raised"
                  :class="index === mentionIndex ? 'bg-raised' : ''"
                  @click="selectMention(index)"
                >
                  <div class="truncate text-xs font-medium text-primary">
                    {{ mail.subject || $t('mail.noSubject') }}
                  </div>
                  <div class="truncate text-[11px] text-secondary">
                    {{ $t('mail.from') }}:
                    {{ mail.fromName || mail.fromAddress || $t('mail.unknownSender') }}
                  </div>
                </button>
              </template>
              <div v-else class="px-3 py-2 text-xs text-secondary">
                {{ $t('aiAssistant.noMentionResults') }}
              </div>
            </div>
          </div>
          <button
            type="button"
            class="mt-2 flex h-8 w-full items-center justify-center rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="isLoading || !input.trim()"
            @click="handleSend"
          >
            {{ isLoading ? $t('aiAssistant.thinking') : $t('aiAssistant.send') }}
          </button>
        </div>
      </template>
    </div>
  </Teleport>
</template>
