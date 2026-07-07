import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';

export function useAiCompose() {
  const { call } = useTauriInvoke();
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
      const result = await call<string>('ai_compose_assist', {
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
