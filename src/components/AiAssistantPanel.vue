<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import { X } from 'lucide-vue-next';
import { useAiChat } from '@/composables/useAiChat';
import AiMessageList from '@/components/AiMessageList.vue';
import { useAiStore } from '@/stores/ai';

const aiStore = useAiStore();
const { messages, isLoading, error, activeSessionId, loadSessions, createSession, sendMessage } =
  useAiChat();

const input = ref('');
const messagesContainer = ref<HTMLElement>();

onMounted(() => {
  void loadSessions();
  void aiStore.loadProviders();
});

async function handleSend() {
  if (!input.value.trim() || !activeSessionId.value) return;
  const content = input.value;
  input.value = '';
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
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-y-0 right-0 z-50 flex w-80 transform flex-col border-l border-border bg-elevated shadow-lg transition-transform duration-200 ease-out"
      :class="aiStore.isPanelOpen ? 'translate-x-0' : 'translate-x-full'"
    >
      <div class="flex h-12 items-center justify-between border-b border-border px-4">
        <span class="text-sm font-medium text-primary">{{ $t('aiAssistant.title') }}</span>
        <button
          type="button"
          class="rounded-md p-1 text-secondary transition-colors hover:bg-raised hover:text-primary"
          :aria-label="$t('common.close')"
          @click="aiStore.togglePanel()"
        >
          <X class="h-4 w-4" />
        </button>
      </div>

      <div
        v-if="!activeSessionId"
        class="flex flex-1 flex-col items-center justify-center p-4 text-center"
      >
        <p class="mb-4 text-sm text-secondary">{{ $t('aiAssistant.selectProvider') }}</p>
        <div class="flex flex-wrap justify-center gap-2">
          <button
            v-for="provider in aiStore.providers"
            :key="provider.id"
            type="button"
            class="rounded-md border border-border bg-base px-3 py-2 text-xs text-secondary transition-colors hover:bg-raised"
            @click="createSession(provider.id)"
          >
            {{ provider.name }}
          </button>
        </div>
      </div>

      <template v-else>
        <div ref="messagesContainer" class="flex-1 overflow-y-auto p-3">
          <AiMessageList :messages="messages" />
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
          <textarea
            v-model="input"
            class="w-full resize-none rounded-md border border-border bg-base p-2.5 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :rows="3"
            :placeholder="$t('aiAssistant.inputPlaceholder')"
            @input="clearError"
            @keydown.enter.prevent="handleSend"
          />
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
