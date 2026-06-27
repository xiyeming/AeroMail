import { createI18n } from 'vue-i18n';

export type Locale = 'en' | 'zh-CN';
export const SUPPORTED_LOCALES: Locale[] = ['en', 'zh-CN'];

/**
 * Custom runtime-only message compiler that treats messages as plain strings
 * with simple `{key}` / `{0}` interpolation. This avoids the default ICU
 * compiler mis-parsing `@` in values like `your.email@example.com` as a
 * linked-message prefix.
 */
function runtimeMessageCompiler(
  message: string,
  _ctx: { locale?: string; key?: string; onError?: (err: Error) => void },
) {
  return (ctx: { values?: Record<string, unknown> }) =>
    message.replace(/\{([^}]+)}/g, (_, key: string) => {
      const val = ctx.values?.[key.trim()];
      return val == null ? '' : String(val);
    });
}

export const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: {},
  missingWarn: false,
  fallbackWarn: false,
  messageCompiler: runtimeMessageCompiler,
});

export async function loadLocaleMessages(locale: Locale): Promise<void> {
  const messages = await import(`./locales/${locale}.json`);
  i18n.global.setLocaleMessage(locale, messages.default);
}
