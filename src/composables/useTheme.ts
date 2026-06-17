import { ref, watch } from 'vue';

type Theme = 'dark' | 'light';

const theme = ref<Theme>('dark');

export function useTheme() {
  function setTheme(value: Theme) {
    theme.value = value;
    document.documentElement.setAttribute('data-theme', value);
  }

  function toggleTheme() {
    setTheme(theme.value === 'dark' ? 'light' : 'dark');
  }

  watch(
    theme,
    (value) => {
      document.documentElement.setAttribute('data-theme', value);
    },
    { immediate: true }
  );

  return {
    theme,
    toggleTheme,
    setTheme,
  };
}
