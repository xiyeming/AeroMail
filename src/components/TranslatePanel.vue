<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTranslation } from '@/composables/useTranslation';
import type { TranslationProviderSummary } from '@/types/translation';

const props = defineProps<{
  mailId: string;
}>();

const emit = defineEmits<{
  translated: [payload: { text: string; lang: string }];
}>();

const { t } = useI18n();
const { translateMail, listProviders, getDefaultTargetLang, isTranslating } = useTranslation();

const providers = ref<TranslationProviderSummary[]>([]);
const selectedProviderId = ref('');
const targetLang = ref(getDefaultTargetLang());

const languages = [
  { value: 'en', label: 'translation.language.english' },
  { value: 'zh-CN', label: 'translation.language.chinese' },
  { value: 'ja', label: 'translation.language.japanese' },
  { value: 'ko', label: 'translation.language.korean' },
];

async function loadProviders() {
  providers.value = await listProviders();
  if (providers.value.length > 0 && !selectedProviderId.value) {
    selectedProviderId.value = providers.value[0].id;
  }
}

async function handleTranslate() {
  if (!selectedProviderId.value) return;
  const translated = await translateMail(props.mailId, targetLang.value, selectedProviderId.value);
  emit('translated', { text: translated, lang: targetLang.value });
}

loadProviders();
</script>

<template>
  <div class="flex flex-wrap items-center gap-2 rounded-lg border border-border bg-elevated p-2">
    <select
      v-model="targetLang"
      class="h-8 rounded-md border border-border bg-base px-2 text-xs text-primary outline-none focus:border-accent"
    >
      <option v-for="lang in languages" :key="lang.value" :value="lang.value">
        {{ $t(lang.label) }}
      </option>
    </select>
    <select
      v-model="selectedProviderId"
      class="h-8 rounded-md border border-border bg-base px-2 text-xs text-primary outline-none focus:border-accent"
    >
      <option v-for="p in providers" :key="p.id" :value="p.id">
        {{ p.name }}
      </option>
    </select>
    <button
      type="button"
      class="flex h-8 items-center rounded-md bg-accent px-3 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
      :disabled="isTranslating || !selectedProviderId"
      @click="handleTranslate"
    >
      {{ isTranslating ? t('translation.translating') : t('translation.translate') }}
    </button>
  </div>
</template>
