import { createI18n } from 'vue-i18n';

export type Locale = 'en' | 'zh-CN';
export const SUPPORTED_LOCALES: Locale[] = ['en', 'zh-CN'];

export const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: {},
  missingWarn: false,
  fallbackWarn: false,
});

export async function loadLocaleMessages(locale: Locale): Promise<void> {
  const messages = await import(`./locales/${locale}.json`);
  i18n.global.setLocaleMessage(locale, messages.default);
}
