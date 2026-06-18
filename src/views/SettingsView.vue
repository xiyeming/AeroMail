<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import LocaleSwitch from '@/components/LocaleSwitch.vue';
import { useAiStore } from '@/stores/ai';
import type { AiProviderKind } from '@/types/ai';

const aiStore = useAiStore();
const showAddForm = ref(false);
const newName = ref('');
const newApiKey = ref('');
const newBaseUrl = ref('');
const newModel = ref('');
const selectedKind = ref<AiProviderKind>('deepseek');

const providerKinds: AiProviderKind[] = [
  'openai',
  'anthropic',
  'gemini',
  'azure_openai',
  'deepseek',
  'moonshot',
  'qwen',
  'zhipu',
  'minimax',
  'baichuan',
  'custom_openai_compatible',
];

onMounted(() => {
  void aiStore.loadProviders();
});

function resetForm() {
  newName.value = '';
  newApiKey.value = '';
  newBaseUrl.value = '';
  newModel.value = '';
  selectedKind.value = 'deepseek';
}

async function addProvider() {
  await invoke('upsert_ai_provider', {
    provider: {
      id: crypto.randomUUID(),
      name: newName.value,
      kind: selectedKind.value,
      apiKeyEncrypted: Array.from(new TextEncoder().encode(newApiKey.value)),
      baseUrl: newBaseUrl.value || undefined,
      model: newModel.value,
      maxTokens: 2048,
    },
  });
  await aiStore.loadProviders();
  resetForm();
  showAddForm.value = false;
}

async function removeProvider(id: string) {
  await invoke('delete_ai_provider', { providerId: id });
  await aiStore.loadProviders();
}
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-panel p-6">
    <h1 class="text-h1 mb-6 font-semibold text-text">{{ $t('nav.settings') }}</h1>

    <section class="rounded-lg border border-border bg-card p-4">
      <h2 class="mb-4 text-lg font-medium text-text">{{ $t('settings.language') }}</h2>
      <LocaleSwitch />
    </section>

    <section class="mt-6 rounded-lg border border-border bg-card p-4">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-medium text-text">{{ $t('settings.aiProviders') }}</h2>
        <button
          class="text-sm text-primary hover:text-primary-hover"
          @click="showAddForm = !showAddForm"
        >
          {{ showAddForm ? $t('common.cancel') : $t('settings.addProvider') }}
        </button>
      </div>

      <div v-if="aiStore.providers.length === 0 && !showAddForm" class="py-4 text-center text-sm text-muted">
        {{ $t('settings.noProviders') }}
      </div>

      <div
        v-for="p in aiStore.providers"
        :key="p.id"
        class="flex items-center justify-between border-b border-border py-2 last:border-0"
      >
        <div class="flex items-center gap-2">
          <span class="text-sm text-text">{{ p.name }}</span>
          <span class="rounded bg-panel px-1.5 py-0.5 text-xs text-muted">{{ p.kind }}</span>
          <span class="text-xs text-muted">{{ p.model }}</span>
        </div>
        <button
          class="text-xs text-danger hover:text-danger-hover"
          @click="removeProvider(p.id)"
        >
          {{ $t('account.delete') }}
        </button>
      </div>

      <div v-if="showAddForm" class="mt-4 space-y-3 rounded-md border border-border bg-panel p-4">
        <div>
          <label class="mb-1 block text-xs text-muted">{{ $t('account.accountName') }}</label>
          <input
            v-model="newName"
            class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            :placeholder="$t('account.namePlaceholder')"
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-muted">{{ $t('settings.providerType') }}</label>
          <select
            v-model="selectedKind"
            class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
          >
            <option v-for="kind in providerKinds" :key="kind" :value="kind">
              {{ kind }}
            </option>
          </select>
        </div>
        <div>
          <label class="mb-1 block text-xs text-muted">{{ $t('settings.apiKey') }}</label>
          <input
            v-model="newApiKey"
            type="password"
            class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            placeholder="sk-..."
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-muted">{{ $t('settings.baseUrl') }}</label>
          <input
            v-model="newBaseUrl"
            class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            placeholder="https://api.example.com/v1"
          />
        </div>
        <div>
          <label class="mb-1 block text-xs text-muted">{{ $t('settings.model') }}</label>
          <input
            v-model="newModel"
            class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            placeholder="deepseek-chat"
          />
        </div>
        <button
          class="mt-2 flex h-8 w-full items-center justify-center rounded-md bg-primary text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-50"
          :disabled="!newName || !newApiKey || !newModel"
          @click="addProvider"
        >
          {{ $t('settings.saveProvider') }}
        </button>
      </div>
    </section>
  </div>
</template>
