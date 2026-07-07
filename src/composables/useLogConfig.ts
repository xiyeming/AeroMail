import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';

export interface LogConfig {
  enabled: boolean;
  dir: string;
}

const config = ref<LogConfig>({ enabled: false, dir: '' });
const loading = ref(false);

export function useLogConfig() {
  const { call } = useTauriInvoke();
  async function loadConfig() {
    loading.value = true;
    try {
      config.value = await call<LogConfig>('get_log_config');
    } finally {
      loading.value = false;
    }
  }

  async function saveConfig(value: LogConfig) {
    await call('set_log_config', { config: value });
    config.value = value;
  }

  return {
    config,
    loading,
    loadConfig,
    saveConfig,
  };
}
