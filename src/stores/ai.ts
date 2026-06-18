import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AiProviderSummary } from '@/types/ai';

export const useAiStore = defineStore('ai', () => {
  const providers = ref<AiProviderSummary[]>([]);
  const isPanelOpen = ref(false);

  async function loadProviders() {
    providers.value = await invoke<AiProviderSummary[]>('list_ai_providers');
  }

  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value;
  }

  return { providers, isPanelOpen, loadProviders, togglePanel };
});
