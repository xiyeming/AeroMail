import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface LogConfig {
  enabled: boolean;
  dir: string;
}

const config = ref<LogConfig>({ enabled: false, dir: '' });
const loading = ref(false);

export function useLogConfig() {
  async function loadConfig() {
    loading.value = true;
    try {
      config.value = await invoke<LogConfig>('get_log_config');
    } finally {
      loading.value = false;
    }
  }

  async function saveConfig(value: LogConfig) {
    await invoke('set_log_config', { config: value });
    config.value = value;
  }

  return {
    config,
    loading,
    loadConfig,
    saveConfig,
  };
}
