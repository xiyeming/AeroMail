<script setup lang="ts">
import { ref } from 'vue';
import AiQuickActions from '@/components/AiQuickActions.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';

const { createSession, sendMessage } = useAiChat();
const aiStore = useAiStore();

const currentMailId = ref<string | null>(null);

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
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <div
      v-if="!currentMailId"
      class="flex flex-1 items-center justify-center text-muted"
    >
      {{ $t('mail.selectEmail') }}
    </div>
    <template v-else>
      <div class="flex items-center justify-between border-b border-border px-4 py-2">
        <AiQuickActions
          @summarize="handleQuickAction('summarize')"
          @reply="handleQuickAction('reply')"
          @extract-todos="handleQuickAction('extractTodos')"
        />
      </div>
    </template>
  </div>
</template>
