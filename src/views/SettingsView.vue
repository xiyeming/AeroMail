<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Trash2 } from 'lucide-vue-next';
import { useLocale, type Locale } from '@/composables/useLocale';
import { useLogConfig } from '@/composables/useLogConfig';
import { useTheme, type Theme } from '@/composables/useTheme';
import { useWindowFrame, type Decorations } from '@/composables/useWindowFrame';
import { useAccountStore } from '@/stores/account';
import { useAiStore } from '@/stores/ai';
import AccountForm from '@/components/AccountForm.vue';
import BaseSelect from '@/components/BaseSelect.vue';
import type { AiProviderKind } from '@/types/ai';
import type { TranslationProviderSummary, TraditionalProviderKind } from '@/types/translation';

const { locale, setLocale, supportedLocales } = useLocale();
const { theme, setTheme } = useTheme();
const { decorations, setDecorations } = useWindowFrame();

const currentLocale = computed({
  get: () => locale.value as Locale,
  set: (value: Locale) => void setLocale(value),
});

const currentTheme = computed({
  get: () => theme.value,
  set: (value: Theme) => void setTheme(value),
});

const systemTitleBarEnabled = computed({
  get: () => decorations.value === 'system',
  set: (value: boolean) => void setDecorations(value ? 'system' : ('none' as Decorations)),
});

const { config: logConfig, loadConfig: loadLogConfig, saveConfig: saveLogConfig } = useLogConfig();

const logEnabled = computed({
  get: () => logConfig.value.enabled,
  set: (value: boolean) => void saveLogConfig({ ...logConfig.value, enabled: value }),
});

async function handleLogDirChange(dir: string) {
  await saveLogConfig({ ...logConfig.value, dir });
}

const localeLabels: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '简体中文',
};

// --- AI Providers ---
const aiStore = useAiStore();
const accountStore = useAccountStore();
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
  void accountStore.loadAccounts();
  void aiStore.loadProviders();
  void loadTranslationProviders();
  void loadLogConfig();
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
    <h1 class="mb-6 text-2xl font-semibold">{{ $t('settings.title') }}</h1>

    <section
      aria-labelledby="general-heading"
      class="space-y-4 rounded-lg border border-border bg-elevated p-5"
    >
      <h2 id="general-heading" class="text-lg font-medium">{{ $t('settings.general') }}</h2>
      <div class="grid gap-4 sm:grid-cols-2">
        <div class="flex flex-col gap-1.5">
          <label for="locale-select" class="text-sm text-secondary">{{
            $t('settings.language')
          }}</label>
          <BaseSelect
            id="locale-select"
            v-model="currentLocale"
            :options="supportedLocales.map((loc) => ({ value: loc, label: localeLabels[loc] }))"
          />
        </div>
        <div class="flex flex-col gap-1.5">
          <label for="theme-select" class="text-sm text-secondary">{{
            $t('settings.theme')
          }}</label>
          <BaseSelect
            id="theme-select"
            v-model="currentTheme"
            :options="[
              { value: 'dark', label: $t('settings.themeDark') },
              { value: 'light', label: $t('settings.themeLight') },
            ]"
          />
        </div>
        <div class="mt-4 flex items-center gap-2">
          <input
            id="system-title-bar"
            v-model="systemTitleBarEnabled"
            type="checkbox"
            class="h-4 w-4 rounded border-border bg-base text-accent outline-none focus:ring-2 focus:ring-accent focus:ring-offset-0"
          />
          <label for="system-title-bar" class="text-sm text-secondary">
            {{ $t('settings.systemTitleBar') }}
          </label>
        </div>
      </div>
    </section>

    <section
      aria-labelledby="log-heading"
      class="mt-6 rounded-lg border border-border bg-elevated p-5"
    >
      <h2 id="log-heading" class="mb-4 text-lg font-medium">{{ $t('settings.log') }}</h2>
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <input
            id="log-enabled"
            v-model="logEnabled"
            type="checkbox"
            class="h-4 w-4 rounded border-border bg-base text-accent outline-none focus:ring-2 focus:ring-accent focus:ring-offset-0"
          />
          <label for="log-enabled" class="text-sm text-secondary">
            {{ $t('settings.logEnabled') }}
          </label>
        </div>
        <div>
          <label for="log-dir" class="mb-1 block text-sm text-secondary">
            {{ $t('settings.logDirectory') }}
          </label>
          <input
            id="log-dir"
            type="text"
            :value="logConfig.dir"
            :placeholder="$t('settings.logDirectoryPlaceholder')"
            class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            @change="handleLogDirChange(($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>
    </section>

    <section
      aria-labelledby="accounts-heading"
      class="mt-6 rounded-lg border border-border bg-elevated p-5"
    >
      <h2 id="accounts-heading" class="mb-4 text-lg font-medium">{{ $t('settings.accounts') }}</h2>

      <div v-if="accountStore.loading" class="py-6 text-center text-sm text-secondary">
        {{ $t('common.loading') }}
      </div>
      <div
        v-else-if="accountStore.accounts.length === 0"
        class="py-6 text-center text-sm text-secondary"
      >
        {{ $t('account.noAccounts') }}
      </div>
      <ul v-else class="mb-5 divide-y divide-border">
        <li
          v-for="account in accountStore.accounts"
          :key="account.id"
          class="flex items-center justify-between py-2.5"
        >
          <div class="flex items-center gap-3">
            <div
              class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-raised text-sm font-medium text-primary"
            >
              {{ account.name.charAt(0).toUpperCase() }}
            </div>
            <div class="min-w-0">
              <div class="truncate text-sm text-primary">{{ account.name }}</div>
              <div class="truncate text-xs text-secondary">
                {{ account.email || account.imapHost }}
              </div>
            </div>
          </div>
          <button
            type="button"
            class="rounded-md p-1.5 text-secondary transition-colors hover:bg-danger-subtle hover:text-danger"
            :aria-label="$t('common.delete')"
            @click="accountStore.deleteAccount(account.id)"
          >
            <Trash2 class="h-4 w-4" />
          </button>
        </li>
      </ul>

      <AccountForm />
    </section>

    <section class="mt-6 rounded-lg border border-border bg-elevated p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-medium">{{ $t('settings.aiProviders') }}</h2>
        <button
          type="button"
          class="flex h-9 items-center justify-center rounded-md border border-border bg-base px-3 text-sm text-primary transition-colors hover:bg-raised"
          @click="showAddForm = !showAddForm"
        >
          {{ showAddForm ? $t('common.cancel') : $t('settings.addProvider') }}
        </button>
      </div>

      <div
        v-if="aiStore.providers.length === 0 && !showAddForm"
        class="py-6 text-center text-sm text-secondary"
      >
        {{ $t('settings.noProviders') }}
      </div>

      <ul v-else-if="aiStore.providers.length > 0" class="divide-y divide-border">
        <li
          v-for="p in aiStore.providers"
          :key="p.id"
          class="flex items-center justify-between py-2.5"
        >
          <div class="flex items-center gap-2">
            <span class="text-sm text-primary">{{ p.name }}</span>
            <span class="rounded bg-raised px-2 py-0.5 text-xs text-secondary">{{ p.kind }}</span>
            <span class="text-xs text-tertiary">{{ p.model }}</span>
          </div>
          <button
            type="button"
            class="text-xs text-danger transition-colors hover:text-danger-hover"
            @click="removeProvider(p.id)"
          >
            {{ $t('common.delete') }}
          </button>
        </li>
      </ul>

      <div v-if="showAddForm" class="mt-4 space-y-3 rounded-md border border-border bg-base p-4">
        <div>
          <label class="mb-1 block text-sm text-secondary">{{ $t('settings.providerName') }}</label>
          <input
            v-model="newName"
            type="text"
            class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="$t('settings.providerNamePlaceholder')"
          />
        </div>
        <div>
          <label class="mb-1 block text-sm text-secondary">{{ $t('settings.providerType') }}</label>
          <BaseSelect
            v-model="selectedKind"
            variant="elevated"
            :options="providerKinds.map((kind) => ({ value: kind, label: kind }))"
          />
        </div>
        <div>
          <label class="mb-1 block text-sm text-secondary">{{ $t('settings.accessKey') }}</label>
          <input
            v-model="newApiKey"
            type="password"
            class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="$t('settings.accessKeyPlaceholder')"
          />
        </div>
        <div>
          <label class="mb-1 block text-sm text-secondary">{{
            $t('settings.serverAddress')
          }}</label>
          <input
            v-model="newBaseUrl"
            type="text"
            class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="$t('settings.serverAddressPlaceholder')"
          />
        </div>
        <div>
          <label class="mb-1 block text-sm text-secondary">{{ $t('settings.model') }}</label>
          <input
            v-model="newModel"
            type="text"
            class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="$t('settings.modelPlaceholder')"
          />
        </div>
        <button
          type="button"
          class="mt-2 flex h-9 w-full items-center justify-center rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="!newName || !newApiKey || !newModel"
          @click="addProvider"
        >
          {{ $t('settings.saveProvider') }}
        </button>
      </div>
    </section>

    <section class="mt-6 rounded-lg border border-border bg-elevated p-5">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-lg font-medium">{{ $t('settings.translationProviders') }}</h2>
        <button
          type="button"
          class="flex h-9 items-center justify-center rounded-md border border-border bg-base px-3 text-sm text-primary transition-colors hover:bg-raised"
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
        class="py-6 text-center text-sm text-secondary"
      >
        {{ $t('settings.noTranslationProviders') }}
      </div>

      <ul v-else-if="translationProviders.length > 0" class="divide-y divide-border">
        <li
          v-for="tp in translationProviders"
          :key="tp.id"
          class="flex items-center justify-between py-2.5"
        >
          <div class="flex items-center gap-2">
            <span class="text-sm text-primary">{{ tp.name }}</span>
            <span class="rounded bg-raised px-2 py-0.5 text-xs text-secondary">{{
              tp.providerType
            }}</span>
          </div>
          <button
            type="button"
            class="text-xs text-danger transition-colors hover:text-danger-hover"
            @click="removeTranslationProvider(tp.id)"
          >
            {{ $t('common.delete') }}
          </button>
        </li>
      </ul>

      <div
        v-if="showTranslationForm"
        class="mt-4 space-y-3 rounded-md border border-border bg-base p-4"
      >
        <div class="flex gap-2">
          <button
            type="button"
            class="flex h-9 items-center justify-center rounded-md px-3 text-sm transition-colors"
            :class="
              translationFormType === 'traditional'
                ? 'bg-accent text-white'
                : 'border border-border bg-elevated text-primary hover:bg-raised'
            "
            @click="
              translationFormType = 'traditional';
              resetTranslationForm();
            "
          >
            {{ $t('settings.traditionalProvider') }}
          </button>
          <button
            type="button"
            class="flex h-9 items-center justify-center rounded-md px-3 text-sm transition-colors"
            :class="
              translationFormType === 'ai'
                ? 'bg-accent text-white'
                : 'border border-border bg-elevated text-primary hover:bg-raised'
            "
            @click="
              translationFormType = 'ai';
              resetTranslationForm();
            "
          >
            {{ $t('settings.aiTranslationProvider') }}
          </button>
        </div>

        <template v-if="translationFormType === 'traditional'">
          <div>
            <label class="mb-1 block text-sm text-secondary">{{
              $t('settings.providerName')
            }}</label>
            <input
              v-model="tpName"
              type="text"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
              :placeholder="$t('settings.providerNamePlaceholder')"
            />
          </div>
          <div>
            <label class="mb-1 block text-sm text-secondary">{{
              $t('settings.providerKind')
            }}</label>
            <BaseSelect
              v-model="tpKind"
              variant="elevated"
              :options="traditionalKinds.map((kind) => ({ value: kind, label: kind }))"
            />
          </div>
          <div>
            <label class="mb-1 block text-sm text-secondary">{{ $t('settings.accessKey') }}</label>
            <input
              v-model="tpApiKey"
              type="password"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
              :placeholder="$t('settings.accessKeyPlaceholder')"
            />
          </div>
          <div>
            <label class="mb-1 block text-sm text-secondary">{{ $t('settings.endpoint') }}</label>
            <input
              v-model="tpEndpoint"
              type="text"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
              :placeholder="$t('settings.endpointPlaceholder')"
            />
          </div>
        </template>

        <template v-else>
          <div>
            <label class="mb-1 block text-sm text-secondary">{{
              $t('settings.providerName')
            }}</label>
            <input
              v-model="tpAiName"
              type="text"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
              :placeholder="$t('settings.providerNamePlaceholder')"
            />
          </div>
          <div>
            <label class="mb-1 block text-sm text-secondary">{{
              $t('settings.selectAiProvider')
            }}</label>
            <BaseSelect
              v-model="tpAiProviderId"
              variant="elevated"
              :placeholder="$t('settings.selectAiProvider')"
              :options="
                aiStore.providers.map((ap) => ({
                  value: ap.id,
                  label: `${ap.name} (${ap.kind})`,
                }))
              "
            />
          </div>
        </template>

        <button
          type="button"
          class="mt-2 flex h-9 w-full items-center justify-center rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
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
