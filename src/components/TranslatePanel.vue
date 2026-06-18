<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTranslation } from '@/composables/useTranslation';
import type { TranslationProviderSummary } from '@/types/translation';

const props = defineProps<{
  mailId: string;
}>();

const emit = defineEmits<{
  translated: [text: string];
}>();

const { t } = useI18n();
const { translateMail, listProviders, getDefaultTargetLang, isTranslating } =
  useTranslation();

const providers = ref<TranslationProviderSummary[]>([]);
const selectedProviderId = ref('');
const targetLang = ref(getDefaultTargetLang());

async function loadProviders() {
  providers.value = await listProviders();
  if (providers.value.length > 0 && !selectedProviderId.value) {
    selectedProviderId.value = providers.value[0].id;
  }
}

async function handleTranslate() {
  if (!selectedProviderId.value) return;
  const translated = await translateMail(
    props.mailId,
    targetLang.value,
    selectedProviderId.value,
  );
  emit('translated', translated);
}

// Load providers on mount
loadProviders();
</script>

<template>
  <div class="flex items-center gap-2 rounded-lg border border-border bg-card p-2">
    <select
      v-model="targetLang"
      class="h-7 rounded border border-border bg-panel px-2 text-xs text-text"
    >
      <option value="en">English</option>
      <option value="zh-CN">简体中文</option>
      <option value="ja">日本語</option>
      <option value="ko">한국어</option>
    </select>
    <select
      v-model="selectedProviderId"
      class="h-7 rounded border border-border bg-panel px-2 text-xs text-text"
    >
      <option v-for="p in providers" :key="p.id" :value="p.id">
        {{ p.name }}
      </option>
    </select>
    <button
      class="flex h-7 items-center rounded bg-primary px-2 text-xs text-white hover:bg-primary-hover disabled:opacity-50"
      :disabled="isTranslating || !selectedProviderId"
      @click="handleTranslate"
    >
      {{ isTranslating ? t('translation.translating') : t('translation.translate') }}
    </button>
  </div>
</template>
