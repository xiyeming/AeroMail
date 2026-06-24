import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { TranslationProviderSummary } from '@/types/translation';
import { useLocale } from '@/composables/useLocale';

export function useTranslation() {
  const { locale: appLocale } = useLocale();
  const isTranslating = ref(false);
  const error = ref<string | null>(null);

  function getDefaultTargetLang(): string {
    return appLocale.value === 'zh-CN' ? 'en' : 'zh-CN';
  }

  async function translateMail(
    mailId: string,
    targetLang: string,
    providerId: string
  ): Promise<string> {
    isTranslating.value = true;
    error.value = null;
    try {
      return await invoke<string>('translate_mail_text', {
        mailId,
        targetLang,
        providerId,
      });
    } catch (raw) {
      error.value = String(raw);
      throw raw;
    } finally {
      isTranslating.value = false;
    }
  }

  async function translateText(
    text: string,
    targetLang: string,
    providerId: string
  ): Promise<string> {
    isTranslating.value = true;
    error.value = null;
    try {
      return await invoke<string>('translate_text', {
        text,
        targetLang,
        providerId,
      });
    } catch (raw) {
      error.value = String(raw);
      throw raw;
    } finally {
      isTranslating.value = false;
    }
  }

  async function listProviders(): Promise<TranslationProviderSummary[]> {
    return await invoke<TranslationProviderSummary[]>('list_translation_providers');
  }

  return {
    isTranslating,
    error,
    translateMail,
    translateText,
    listProviders,
    getDefaultTargetLang,
  };
}
