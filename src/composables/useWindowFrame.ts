import { ref, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useSettingsStore } from '@/stores/settings';

export type Decorations = 'system' | 'none';

const SETTINGS_KEY = 'app.systemTitleBar';
const decorations = ref<Decorations>('system');

export function useWindowFrame() {
  const settings = useSettingsStore();
  const win = getCurrentWindow();

  async function applyDecorations(value: Decorations) {
    try {
      await win.setDecorations(value === 'system');
    } catch (e) {
      console.error('Failed to apply window decorations:', e);
    }
  }

  async function setDecorations(value: Decorations, persist = true) {
    decorations.value = value;
    await applyDecorations(value);
    if (persist) {
      void settings.set(SETTINGS_KEY, value);
    }
  }

  async function initDecorations() {
    const saved = await settings.get(SETTINGS_KEY);
    const value: Decorations = saved === 'none' ? 'none' : 'system';
    await setDecorations(value, false);
  }

  watch(
    decorations,
    (value) => {
      document.documentElement.setAttribute('data-window-frame', value);
    },
    { immediate: true }
  );

  return {
    decorations,
    setDecorations,
    initDecorations,
  };
}
