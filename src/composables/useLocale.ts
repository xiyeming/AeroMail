import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Locale } from '@/i18n';
import { SUPPORTED_LOCALES, loadLocaleMessages } from '@/i18n';
import { useSettingsStore } from '@/stores/settings';

const SETTINGS_KEY = 'app.locale';

export function useLocale() {
  const { locale } = useI18n();
  const settings = useSettingsStore();
  const isReady = ref(false);

  async function initLocale() {
    const saved = await settings.get(SETTINGS_KEY);
    const target = resolveLocale(saved);
    await setLocale(target, false);
    isReady.value = true;
  }

  async function setLocale(value: Locale, persist = true) {
    if (!SUPPORTED_LOCALES.includes(value)) return;
    await loadLocaleMessages(value);
    locale.value = value;
    document.documentElement.setAttribute('lang', value);
    if (persist) {
      await settings.set(SETTINGS_KEY, value);
    }
  }

  function resolveLocale(raw: string | null): Locale {
    if (raw && SUPPORTED_LOCALES.includes(raw as Locale)) {
      return raw as Locale;
    }
    const osLang = navigator.language;
    if (osLang.startsWith('zh')) return 'zh-CN';
    return 'en';
  }

  return {
    locale,
    isReady,
    initLocale,
    setLocale,
    supportedLocales: SUPPORTED_LOCALES,
  };
}
