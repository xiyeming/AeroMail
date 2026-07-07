import { ref, watch } from 'vue';
import { useSettingsStore } from '@/stores/settings';

export type Theme = 'dark' | 'light';

const SETTINGS_KEY = 'app.theme';

const theme = ref<Theme>('dark');

watch(
  theme,
  (value) => {
    document.documentElement.setAttribute('data-theme', value);
  },
  { immediate: true }
);

export function useTheme() {
  const { get, set } = useSettingsStore();

  function setTheme(value: Theme, persist = true) {
    theme.value = value;
    if (persist) {
      void set(SETTINGS_KEY, value);
    }
  }

  function toggleTheme() {
    setTheme(theme.value === 'dark' ? 'light' : 'dark');
  }

  async function initTheme() {
    const saved = await get(SETTINGS_KEY);
    if (saved === 'dark' || saved === 'light') {
      setTheme(saved, false);
    }
  }

  return {
    theme,
    setTheme,
    toggleTheme,
    initTheme,
  };
}
