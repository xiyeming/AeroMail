<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Languages } from 'lucide-vue-next';
import AiQuickActions from '@/components/AiQuickActions.vue';
import TranslatePanel from '@/components/TranslatePanel.vue';
import { useAiChat } from '@/composables/useAiChat';
import { useAiStore } from '@/stores/ai';

const { t } = useI18n();
const { createSession, sendMessage } = useAiChat();
const aiStore = useAiStore();

const currentMailId = ref<string | null>(null);
const translatedText = ref<string | null>(null);
const showTranslation = ref(false);
const showTranslatePanel = ref(false);

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
        <button
          class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
          :title="t('translation.translate')"
          @click="showTranslatePanel = !showTranslatePanel"
        >
          <Languages class="h-3.5 w-3.5" />
          {{ t('translation.translate') }}
        </button>
      </div>
      <div v-if="showTranslatePanel" class="border-b border-border px-4 py-2">
        <TranslatePanel :mail-id="currentMailId" @translated="handleTranslated" />
      </div>
      <div
        v-if="translatedText && showTranslation"
        class="flex items-center justify-between border-b border-border bg-primary/5 px-4 py-1.5"
      >
        <span class="text-xs text-primary">
          {{ t('translation.translatedTo', { lang: translatedText ? 'target' : '' }) }}
        </span>
        <button
          class="text-xs text-primary underline hover:text-primary-hover"
          @click="toggleTranslation"
        >
          {{ t('translation.showOriginal') }}
        </button>
      </div>
      <div
        v-else-if="translatedText && !showTranslation"
        class="flex items-center justify-between border-b border-border bg-muted/10 px-4 py-1.5"
      >
        <span class="text-xs text-muted">
          {{ t('mail.selectEmail') }}
        </span>
        <button
          class="text-xs text-primary underline hover:text-primary-hover"
          @click="toggleTranslation"
        >
          {{ t('translation.showOriginal') }}
        </button>
      </div>
    </template>
  </div>
</template>
