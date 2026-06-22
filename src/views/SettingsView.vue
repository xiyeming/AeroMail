<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useLocale, type Locale } from '@/composables/useLocale';
import { useAiStore } from '@/stores/ai';
import type { AiProviderKind } from '@/types/ai';
import type { TranslationProviderSummary, TraditionalProviderKind } from '@/types/translation';

const { locale, setLocale, supportedLocales } = useLocale();

const currentLocale = computed({
  get: () => locale.value as Locale,
  set: (value: Locale) => setLocale(value),
});

const localeLabels: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '简体中文',
};

// --- AI Providers ---
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
  void loadTranslationProviders();
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

// --- Translation Providers ---
const translationProviders = ref<TranslationProviderSummary[]>([]);
const showTranslationForm = ref(false);
const translationFormType = ref<'traditional' | 'ai'>('traditional');

// Traditional form fields
const tpName = ref('');
const tpKind = ref<TraditionalProviderKind>('google_translate');
const tpApiKey = ref('');
const tpEndpoint = ref('');

// AI form fields
const tpAiName = ref('');
const tpAiProviderId = ref('');

const traditionalKinds: TraditionalProviderKind[] = [
  'google_translate',
  'deep_l',
  'azure_translator',
  'baidu',
  'youdao',
  'tencent_translator',
  'aliyun_translator',
  'custom',
];

async function loadTranslationProviders() {
  translationProviders.value = await invoke<TranslationProviderSummary[]>(
    'list_translation_providers'
  );
}

function resetTranslationForm() {
  tpName.value = '';
  tpKind.value = 'google_translate';
  tpApiKey.value = '';
  tpEndpoint.value = '';
  tpAiName.value = '';
  tpAiProviderId.value = '';
}

async function addTranslationProvider() {
  if (translationFormType.value === 'traditional') {
    await invoke('upsert_translation_provider', {
      provider: {
        type: 'traditional',
        id: crypto.randomUUID(),
        name: tpName.value,
        kind: tpKind.value,
        api_key_encrypted: Array.from(new TextEncoder().encode(tpApiKey.value)),
        endpoint: tpEndpoint.value || undefined,
        extra: {},
      },
    });
  } else {
    await invoke('upsert_translation_provider', {
      provider: {
        type: 'ai',
        id: crypto.randomUUID(),
        name: tpAiName.value,
        ai_provider_id: tpAiProviderId.value,
        prompt_template: undefined,
      },
    });
  }
  await loadTranslationProviders();
  resetTranslationForm();
  showTranslationForm.value = false;
}

async function removeTranslationProvider(id: string) {
  await invoke('delete_translation_provider', { providerId: id });
  await loadTranslationProviders();
}
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-base p-6 text-primary">
    <h1 class="mb-6 text-2xl font-semibold">{{ $t('nav.settings') }}</h1>

    <section aria-labelledby="general-heading" class="space-y-4 rounded-lg border border-border bg-elevated p-5">
      <h2 id="general-heading" class="text-lg font-semibold">{{ $t('settings.language') }}</h2>
      <div class="flex items-center gap-3">
        <label for="locale-select" class="text-sm text-secondary">{{ $t('settings.language') }}</label>
        <select
          id="locale-select"
          v-model="currentLocale"
          class="h-8 rounded-md border border-border bg-raised px-2 text-sm text-primary outline-none focus:border-accent"
        >
          <option v-for="loc in supportedLocales" :key="loc" :value="loc">
            {{ localeLabels[loc] }}
          </option>
        </select>
      </div>
    </section>

    <section class="mt-6 rounded-lg border border-border bg-card p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-medium text-text">{{ $t('settings.aiProviders') }}</h2>
        <button
          class="rounded-md px-3 py-1.5 text-sm text-primary transition-colors hover:bg-primary/10"
          @click="showAddForm = !showAddForm"
        >
          {{ showAddForm ? $t('common.cancel') : $t('settings.addProvider') }}
        </button>
      </div>

      <div
        v-if="aiStore.providers.length === 0 && !showAddForm"
        class="py-4 text-center text-sm text-muted"
      >
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
        <button class="text-xs text-danger hover:text-danger-hover" @click="removeProvider(p.id)">
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

    <!-- Translation Providers -->
    <section class="mt-6 rounded-lg border border-border bg-card p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-medium text-text">{{ $t('settings.translationProviders') }}</h2>
        <button
          class="rounded-md px-3 py-1.5 text-sm text-primary transition-colors hover:bg-primary/10"
          @click="
            showTranslationForm = !showTranslationForm;
            resetTranslationForm();
          "
        >
          {{ showTranslationForm ? $t('common.cancel') : $t('settings.addTranslationProvider') }}
        </button>
      </div>

      <div
        v-if="translationProviders.length === 0 && !showTranslationForm"
        class="py-4 text-center text-sm text-muted"
      >
        {{ $t('settings.noTranslationProviders') }}
      </div>

      <div
        v-for="tp in translationProviders"
        :key="tp.id"
        class="flex items-center justify-between border-b border-border py-2 last:border-0"
      >
        <div class="flex items-center gap-2">
          <span class="text-sm text-text">{{ tp.name }}</span>
          <span class="rounded bg-panel px-1.5 py-0.5 text-xs text-muted">{{
            tp.providerType
          }}</span>
        </div>
        <button
          class="text-xs text-danger hover:text-danger-hover"
          @click="removeTranslationProvider(tp.id)"
        >
          {{ $t('account.delete') }}
        </button>
      </div>

      <div
        v-if="showTranslationForm"
        class="mt-4 space-y-3 rounded-md border border-border bg-panel p-4"
      >
        <!-- Type selector -->
        <div class="flex gap-2">
          <button
            class="rounded-md px-3 py-1.5 text-sm"
            :class="
              translationFormType === 'traditional'
                ? 'bg-primary text-white'
                : 'bg-card text-text border border-border'
            "
            @click="
              translationFormType = 'traditional';
              resetTranslationForm();
            "
          >
            {{ $t('settings.traditionalProvider') }}
          </button>
          <button
            class="rounded-md px-3 py-1.5 text-sm"
            :class="
              translationFormType === 'ai'
                ? 'bg-primary text-white'
                : 'bg-card text-text border border-border'
            "
            @click="
              translationFormType = 'ai';
              resetTranslationForm();
            "
          >
            {{ $t('settings.aiTranslationProvider') }}
          </button>
        </div>

        <!-- Traditional provider form -->
        <template v-if="translationFormType === 'traditional'">
          <div>
            <label class="mb-1 block text-xs text-muted">{{ $t('account.accountName') }}</label>
            <input
              v-model="tpName"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
              :placeholder="$t('account.namePlaceholder')"
            />
          </div>
          <div>
            <label class="mb-1 block text-xs text-muted">{{ $t('settings.providerKind') }}</label>
            <select
              v-model="tpKind"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            >
              <option v-for="kind in traditionalKinds" :key="kind" :value="kind">
                {{ kind }}
              </option>
            </select>
          </div>
          <div>
            <label class="mb-1 block text-xs text-muted">{{ $t('settings.apiKey') }}</label>
            <input
              v-model="tpApiKey"
              type="password"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
              placeholder="sk-..."
            />
          </div>
          <div>
            <label class="mb-1 block text-xs text-muted">{{ $t('settings.endpoint') }}</label>
            <input
              v-model="tpEndpoint"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
              placeholder="https://api.example.com/translate"
            />
          </div>
        </template>

        <!-- AI provider form -->
        <template v-else>
          <div>
            <label class="mb-1 block text-xs text-muted">{{ $t('account.accountName') }}</label>
            <input
              v-model="tpAiName"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
              :placeholder="$t('account.namePlaceholder')"
            />
          </div>
          <div>
            <label class="mb-1 block text-xs text-muted">{{
              $t('settings.selectAiProvider')
            }}</label>
            <select
              v-model="tpAiProviderId"
              class="w-full rounded-md border border-border bg-card px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
            >
              <option value="" disabled>{{ $t('settings.selectAiProvider') }}</option>
              <option v-for="ap in aiStore.providers" :key="ap.id" :value="ap.id">
                {{ ap.name }} ({{ ap.kind }})
              </option>
            </select>
          </div>
        </template>

        <button
          class="mt-2 flex h-8 w-full items-center justify-center rounded-md bg-primary text-sm font-medium text-white hover:bg-primary-hover disabled:opacity-50"
          :disabled="
            translationFormType === 'traditional'
              ? !tpName || !tpApiKey
              : !tpAiName || !tpAiProviderId
          "
          @click="addTranslationProvider"
        >
          {{ $t('settings.saveProvider') }}
        </button>
      </div>
    </section>
  </div>
</template>
