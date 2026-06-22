<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import { X } from 'lucide-vue-next';
import { useAiChat } from '@/composables/useAiChat';
import AiMessageList from '@/components/AiMessageList.vue';
import { useAiStore } from '@/stores/ai';

const aiStore = useAiStore();
const { messages, isLoading, activeSessionId, loadSessions, createSession, sendMessage } =
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
  await sendMessage(activeSessionId.value, content);
  await nextTick();
  scrollToBottom();
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
  <div class="flex h-full w-[360px] flex-shrink-0 flex-col border-l border-border bg-panel">
    <div class="flex h-12 items-center justify-between border-b border-border px-4">
      <span class="text-sm font-medium text-text">{{ $t('aiAssistant.title') }}</span>
      <button
        class="rounded-md p-1 text-muted transition-colors hover:bg-card hover:text-text"
        @click="aiStore.togglePanel()"
      >
        <X class="h-4 w-4" />
      </button>
    </div>

    <div
      v-if="!activeSessionId"
      class="flex flex-1 flex-col items-center justify-center p-4 text-center"
    >
      <p class="mb-4 text-sm text-muted">{{ $t('aiAssistant.selectProvider') }}</p>
      <div class="flex flex-wrap justify-center gap-2">
        <button
          v-for="provider in aiStore.providers"
          :key="provider.id"
          class="rounded-md border border-border bg-card px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-border-hover hover:text-text"
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
        <textarea
          v-model="input"
          class="w-full resize-none rounded-md border border-border bg-card p-2.5 text-sm text-text outline-none transition-colors placeholder:text-disabled focus:border-primary focus:ring-1 focus:ring-primary/20"
          :rows="3"
          :placeholder="$t('aiAssistant.inputPlaceholder')"
          @keydown.enter.prevent="handleSend"
        />
        <button
          class="mt-2 flex h-8 w-full items-center justify-center rounded-md bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover active:bg-primary-active disabled:opacity-50"
          :disabled="isLoading || !input.trim()"
          @click="handleSend"
        >
          {{ isLoading ? $t('aiAssistant.thinking') : $t('aiAssistant.send') }}
        </button>
      </div>
    </template>
  </div>
</template>
