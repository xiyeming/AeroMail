import { defineStore } from 'pinia';
import { useTauriInvoke } from '@/composables/useTauriInvoke';

export const useSettingsStore = defineStore('settings', () => {
  const { call } = useTauriInvoke();

  async function set(key: string, value: string): Promise<void> {
    await call('set_setting', { key, value });
  }

  async function get(key: string): Promise<string | null> {
    return await call<string | null>('get_setting', { key });
  }

  return { set, get };
});
