<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from 'lucide-vue-next';
import { useTranslation } from '@/composables/useTranslation';
import { useToastStore } from '@/stores/toast';
import BaseSelect from '@/components/BaseSelect.vue';
import type { TranslationProviderSummary } from '@/types/translation';

const props = defineProps<{
  mailId: string;
}>();

const emit = defineEmits<{
  translated: [payload: { text: string; lang: string }];
}>();

const { t } = useI18n();
const toast = useToastStore();
const { translateMail, listProviders, getDefaultTargetLang, isTranslating } = useTranslation();

const providers = ref<TranslationProviderSummary[]>([]);
const selectedProviderId = ref('');
const targetLang = ref(getDefaultTargetLang());
const isLoadingProviders = ref(false);
const translateError = ref<string | null>(null);

const languages = [
  { value: 'en', label: 'translation.language.english' },
  { value: 'zh-CN', label: 'translation.language.chinese' },
  { value: 'ja', label: 'translation.language.japanese' },
  { value: 'ko', label: 'translation.language.korean' },
];

async function loadProviders() {
  isLoadingProviders.value = true;
  try {
    providers.value = await listProviders();
    if (providers.value.length > 0 && !selectedProviderId.value) {
      selectedProviderId.value = providers.value[0].id;
    }
  } catch (e) {
    console.error('Failed to load translation providers:', e);
  } finally {
    isLoadingProviders.value = false;
  }
}

async function handleTranslate() {
  translateError.value = null;
  if (providers.value.length === 0) {
    toast.add({
      type: 'warning',
      message: t('translation.noProviders'),
      duration: 4000,
    });
    return;
  }
  if (!selectedProviderId.value) {
    toast.add({
      type: 'warning',
      message: t('translation.noProviders'),
      duration: 4000,
    });
    return;
  }
  try {
    const translated = await translateMail(props.mailId, targetLang.value, selectedProviderId.value);
    emit('translated', { text: translated, lang: targetLang.value });
  } catch (e) {
    translateError.value = e instanceof Error ? e.message : String(e);
    toast.add({
      type: 'error',
      message: translateError.value,
      duration: 5000,
    });
  }
}

loadProviders();
</script>

<template>
  <div class="flex flex-col gap-2 rounded-lg border border-border bg-elevated p-2">
    <div class="flex flex-wrap items-center gap-2">
      <BaseSelect
        v-model="targetLang"
        size="sm"
        :options="languages.map((lang) => ({ value: lang.value, label: $t(lang.label) }))"
        :disabled="isTranslating || providers.length === 0"
      />
      <BaseSelect
        v-model="selectedProviderId"
        size="sm"
        :options="providers.map((p) => ({ value: p.id, label: p.name }))"
        :disabled="isTranslating || providers.length === 0"
      />
      <button
        type="button"
        class="flex h-8 items-center gap-1 rounded-md bg-accent px-3 text-xs font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="isTranslating || providers.length === 0"
        @click="handleTranslate"
      >
        <Loader2 v-if="isTranslating" class="h-3.5 w-3.5 animate-spin" />
        {{ isTranslating ? t('translation.translating') : t('translation.translate') }}
      </button>
    </div>

    <div
      v-if="providers.length === 0 && !isLoadingProviders"
      class="text-xs text-warning"
      role="alert"
    >
      {{ t('translation.noProviders') }}
    </div>

    <div
      v-if="translateError"
      class="text-xs text-danger"
      role="alert"
    >
      {{ translateError }}
    </div>
  </div>
</template>
