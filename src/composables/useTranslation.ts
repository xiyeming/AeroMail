import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { TranslationProviderSummary } from '@/types/translation';
import { useLocale } from '@/composables/useLocale';
import { useSettingsStore } from '@/stores/settings';

const DEFAULT_TARGET_LANG_KEY = 'translation.defaultTargetLang';

export function useTranslation() {
  const { call } = useTauriInvoke();
  const { locale: appLocale } = useLocale();
  const settingsStore = useSettingsStore();
  const isTranslating = ref(false);
  const error = ref<string | null>(null);

  function getDefaultTargetLang(): string {
    return appLocale.value === 'zh-CN' ? 'en' : 'zh-CN';
  }

  async function loadDefaultTargetLang(): Promise<string> {
    try {
      const saved = await settingsStore.get(DEFAULT_TARGET_LANG_KEY);
      if (saved) return saved;
    } catch (e) {
      console.error('Failed to load default target language:', e);
    }
    return getDefaultTargetLang();
  }

  async function saveDefaultTargetLang(lang: string): Promise<void> {
    try {
      await settingsStore.set(DEFAULT_TARGET_LANG_KEY, lang);
    } catch (e) {
      console.error('Failed to save default target language:', e);
    }
  }

  async function translateMail(
    mailId: string,
    targetLang: string,
    providerId: string
  ): Promise<string> {
    isTranslating.value = true;
    error.value = null;
    try {
      return await call<string>('translate_mail_text', {
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
      return await call<string>('translate_text', {
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
    return await call<TranslationProviderSummary[]>('list_translation_providers');
  }

  return {
    isTranslating,
    error,
    translateMail,
    translateText,
    listProviders,
    getDefaultTargetLang,
    loadDefaultTargetLang,
    saveDefaultTargetLang,
  };
}
