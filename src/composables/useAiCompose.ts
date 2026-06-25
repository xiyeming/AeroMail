import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export function useAiCompose() {
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function assist(
    action: 'write' | 'polish' | 'optimize-en',
    content: string,
    providerId: string
  ) {
    isLoading.value = true;
    error.value = null;
    try {
      const result = await invoke<string>('ai_compose_assist', {
        action,
        content,
        providerId,
      });
      return result;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  return {
    isLoading,
    error,
    assist,
  };
}
