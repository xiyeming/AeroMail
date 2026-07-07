import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AiMcpServer, AiProviderSummary, AiSkill } from '@/types/ai';

const DEFAULT_PROVIDER_KEY = 'app.ai.defaultProviderId';

export const useAiStore = defineStore('ai', () => {
  const { call } = useTauriInvoke();
  const providers = ref<AiProviderSummary[]>([]);
  const mcpServers = ref<AiMcpServer[]>([]);
  const skills = ref<AiSkill[]>([]);
  const defaultProviderId = ref<string | null>(null);
  const isPanelOpen = ref(false);

  async function loadProviders() {
    providers.value = await call<AiProviderSummary[]>('list_ai_providers');
  }

  async function loadDefaultProvider() {
    defaultProviderId.value = await call<string | null>('get_setting', {
      key: DEFAULT_PROVIDER_KEY,
    });
  }

  async function setDefaultProvider(providerId: string | null) {
    if (providerId) {
      await call('set_setting', { key: DEFAULT_PROVIDER_KEY, value: providerId });
    } else {
      await call('set_setting', { key: DEFAULT_PROVIDER_KEY, value: '' });
    }
    defaultProviderId.value = providerId;
  }

  function resolveProviderId(): string | null {
    return defaultProviderId.value ?? providers.value[0]?.id ?? null;
  }

  async function loadMcpServers() {
    mcpServers.value = await call<AiMcpServer[]>('list_ai_mcp_servers');
  }

  async function loadSkills() {
    skills.value = await call<AiSkill[]>('list_ai_skills');
  }

  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value;
  }

  return {
    providers,
    mcpServers,
    skills,
    defaultProviderId,
    isPanelOpen,
    loadProviders,
    loadDefaultProvider,
    setDefaultProvider,
    resolveProviderId,
    loadMcpServers,
    loadSkills,
    togglePanel,
  };
});
