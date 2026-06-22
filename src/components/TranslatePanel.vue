<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTranslation } from '@/composables/useTranslation';
import BaseSelect from '@/components/BaseSelect.vue';
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
    <BaseSelect
      v-model="targetLang"
      size="sm"
      :options="languages.map((lang) => ({ value: lang.value, label: $t(lang.label) }))"
    />
    <BaseSelect
      v-model="selectedProviderId"
      size="sm"
      :options="providers.map((p) => ({ value: p.id, label: p.name }))"
    />
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
