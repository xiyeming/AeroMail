import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  async function set(key: string, value: string): Promise<void> {
    await invoke('set_setting', { key, value });
  }

  async function get(key: string): Promise<string | null> {
    return await invoke<string | null>('get_setting', { key });
  }

  return { set, get };
});
