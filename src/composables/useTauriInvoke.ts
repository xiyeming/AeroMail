import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';

export interface ErrorPayload {
  code: string;
  args: string[];
}

export function useTauriInvoke() {
  const { t } = useI18n();

  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (raw) {
      const payload = parseErrorPayload(raw);
      throw new Error(t(`errors.${payload.code}`, payload.args));
    }
  }

  return { call };
}

function parseErrorPayload(raw: unknown): ErrorPayload {
  if (typeof raw === 'string') {
    try {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed.code === 'string') {
        return {
          code: parsed.code,
          args: Array.isArray(parsed.args) ? parsed.args : [],
        };
      }
    } catch {
      // fall through
    }
    return { code: 'UNKNOWN_ERROR', args: [raw] };
  }

  if (raw && typeof raw === 'object') {
    const obj = raw as Record<string, unknown>;
    return {
      code: typeof obj.code === 'string' ? obj.code : 'UNKNOWN_ERROR',
      args: Array.isArray(obj.args) ? (obj.args as string[]) : [],
    };
  }

  return { code: 'UNKNOWN_ERROR', args: [] };
}
