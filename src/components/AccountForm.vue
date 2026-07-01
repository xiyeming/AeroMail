<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { ChevronDown } from '@lucide/vue';
import { useAccountStore } from '@/stores/account';
import BaseCheckbox from '@/components/BaseCheckbox.vue';
import BaseSelect from '@/components/BaseSelect.vue';
import type { AccountConfig, MailProvider } from '@/types/account';

const props = withDefaults(
  defineProps<{
    mode?: 'add' | 'edit';
    initialConfig?: AccountConfig | null;
  }>(),
  {
    mode: 'add',
    initialConfig: null,
  }
);

const emit = defineEmits<{
  submit: [config: AccountConfig, password: string];
  test: [config: AccountConfig, password: string];
  cancel: [];
}>();

const { t } = useI18n();
const accountStore = useAccountStore();

const providers: MailProvider[] = [
  'Gmail',
  'Outlook',
  'QQ',
  'Netease163',
  'Aliyun',
  'TencentExmail',
  'Custom',
];

const providerOptions = computed(() =>
  providers.map((p) => ({ value: p, label: t(`account.providerLabels.${p}`) }))
);

const tlsOptions = computed(() => [
  { value: 'required', label: t('account.tlsModeRequired') },
  { value: 'starttls', label: t('account.tlsModeStarttls') },
  { value: 'none', label: t('account.tlsModeNone') },
]);

const providerDefaults: Record<MailProvider, { imap: string; smtp: string }> = {
  Gmail: { imap: 'imap.gmail.com', smtp: 'smtp.gmail.com' },
  Outlook: { imap: 'outlook.office365.com', smtp: 'smtp.office365.com' },
  QQ: { imap: 'imap.qq.com', smtp: 'smtp.qq.com' },
  Netease163: { imap: 'imap.163.com', smtp: 'smtp.163.com' },
  Aliyun: { imap: 'imap.aliyun.com', smtp: 'smtp.aliyun.com' },
  TencentExmail: { imap: 'imap.exmail.qq.com', smtp: 'smtp.exmail.qq.com' },
  Custom: { imap: '', smtp: '' },
};

const defaultConfig = (): AccountConfig => ({
  id: '',
  name: '',
  email: '',
  provider: 'Gmail',
  imap: { host: 'imap.gmail.com', port: 993, tlsMode: 'required' },
  smtp: { host: 'smtp.gmail.com', port: 465, tlsMode: 'required' },
  auth: { type: 'Password', passwordEncrypted: [] },
  advanced: {
    caCertPath: null,
    verifyCertificate: true,
    connectTimeoutSecs: 30,
    readTimeoutSecs: 30,
    keepalive: true,
  },
  syncIntervalSecs: 60,
  excludedFolders: [],
});

const config = ref<AccountConfig>(defaultConfig());
const password = ref('');
const validationError = ref<string | null>(null);
const testing = ref(false);
const testMessage = ref<string | null>(null);
const advancedOpen = ref(false);

const storedPasswordMissing = computed(() => {
  if (props.mode !== 'edit' || !props.initialConfig) return false;
  return (
    props.initialConfig.auth.type === 'Password' &&
    props.initialConfig.auth.passwordEncrypted.length === 0
  );
});

watch(
  () => props.initialConfig,
  (value) => {
    if (value) {
      config.value = JSON.parse(JSON.stringify(value));
      password.value = '';
    } else {
      config.value = defaultConfig();
      password.value = '';
    }
    validationError.value = null;
    testMessage.value = null;
  },
  { immediate: true }
);

function updateProvider(provider: MailProvider) {
  config.value.provider = provider;
  const defaults = providerDefaults[provider];
  config.value.imap.host = defaults.imap;
  config.value.smtp.host = defaults.smtp;
}

function validate(): boolean {
  if (!config.value.name.trim()) {
    validationError.value = t('account.errors.nameRequired');
    return false;
  }
  if (!config.value.email.trim()) {
    validationError.value = t('account.errors.emailRequired');
    return false;
  }
  if (!config.value.imap.host.trim() || config.value.imap.port === 0) {
    validationError.value = t('account.errors.imapRequired');
    return false;
  }
  if (!config.value.smtp.host.trim() || config.value.smtp.port === 0) {
    validationError.value = t('account.errors.smtpRequired');
    return false;
  }
  if (
    config.value.auth.type === 'Password' &&
    !password.value &&
    (props.mode === 'add' || storedPasswordMissing.value)
  ) {
    validationError.value = t('account.errors.passwordRequired');
    return false;
  }
  validationError.value = null;
  return true;
}

function buildConfig(): AccountConfig {
  const cloned: AccountConfig = JSON.parse(JSON.stringify(config.value));
  if (cloned.auth.type === 'Password' && password.value) {
    cloned.auth.passwordEncrypted = Array.from(new TextEncoder().encode(password.value));
  }
  if (!cloned.email) {
    cloned.email = cloned.name;
  }
  return cloned;
}

async function handleTest() {
  if (!validate()) return;
  testing.value = true;
  testMessage.value = null;
  accountStore.error = null;
  try {
    const result = await accountStore.testConnection(buildConfig());
    testMessage.value = result;
  } catch {
    testMessage.value = null;
  } finally {
    testing.value = false;
  }
}

async function handleSubmit() {
  if (!validate()) return;
  emit('submit', buildConfig(), password.value);
}

function handleCancel() {
  emit('cancel');
}
</script>

<template>
  <form
    class="space-y-5 rounded-lg border border-border bg-elevated p-5"
    @submit.prevent="handleSubmit"
  >
    <h2 class="text-lg font-medium text-primary">
      {{ mode === 'edit' ? $t('account.editAccount') : $t('account.addAccount') }}
    </h2>

    <div class="space-y-1.5">
      <label for="account-provider" class="text-sm text-secondary">{{
        $t('account.provider')
      }}</label>
      <BaseSelect
        id="account-provider"
        :model-value="config.provider"
        :options="providerOptions"
        @update:model-value="updateProvider($event as MailProvider)"
      />
    </div>

    <div class="space-y-1.5">
      <label for="account-name" class="text-sm text-secondary">{{ $t('account.label') }}</label>
      <input
        id="account-name"
        v-model="config.name"
        type="text"
        class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
        :placeholder="t('account.namePlaceholder')"
      />
    </div>

    <div class="space-y-1.5">
      <label for="account-email" class="text-sm text-secondary">{{
        $t('account.emailAddress')
      }}</label>
      <input
        id="account-email"
        v-model="config.email"
        type="email"
        class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
        :placeholder="t('account.emailPlaceholder')"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div class="space-y-1.5">
        <label for="account-imap-host" class="text-sm text-secondary">{{
          $t('account.imapHost')
        }}</label>
        <input
          id="account-imap-host"
          v-model="config.imap.host"
          type="text"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
      <div class="space-y-1.5">
        <label for="account-imap-port" class="text-sm text-secondary">{{
          $t('account.imapPort')
        }}</label>
        <input
          id="account-imap-port"
          v-model.number="config.imap.port"
          type="number"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
    </div>

    <div class="space-y-1.5">
      <label for="account-imap-tls" class="text-sm text-secondary">{{
        $t('account.imapTlsMode')
      }}</label>
      <BaseSelect id="account-imap-tls" v-model="config.imap.tlsMode" :options="tlsOptions" />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div class="space-y-1.5">
        <label for="account-smtp-host" class="text-sm text-secondary">{{
          $t('account.smtpHost')
        }}</label>
        <input
          id="account-smtp-host"
          v-model="config.smtp.host"
          type="text"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
      <div class="space-y-1.5">
        <label for="account-smtp-port" class="text-sm text-secondary">{{
          $t('account.smtpPort')
        }}</label>
        <input
          id="account-smtp-port"
          v-model.number="config.smtp.port"
          type="number"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
    </div>

    <div class="space-y-1.5">
      <label for="account-smtp-tls" class="text-sm text-secondary">{{
        $t('account.smtpTlsMode')
      }}</label>
      <BaseSelect id="account-smtp-tls" v-model="config.smtp.tlsMode" :options="tlsOptions" />
    </div>

    <div class="space-y-1.5">
      <label for="account-password" class="text-sm text-secondary">{{
        $t('account.password')
      }}</label>
      <input
        id="account-password"
        v-model="password"
        type="password"
        class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
        :placeholder="t('account.passwordPlaceholder')"
      />
      <p v-if="mode === 'edit'" class="text-xs text-tertiary">
        {{ $t('account.passwordUnchangedHint') }}
      </p>
      <p v-if="storedPasswordMissing" class="text-xs text-warning">
        {{ $t('account.passwordMissingHint') }}
      </p>
    </div>

    <div class="rounded-md border border-border bg-base">
      <button
        type="button"
        class="flex w-full items-center justify-between p-3 text-sm font-medium text-secondary transition-colors hover:text-primary"
        @click="advancedOpen = !advancedOpen"
      >
        {{ $t('account.advancedSettings') }}
        <ChevronDown
          class="h-4 w-4 text-tertiary transition-transform"
          :class="advancedOpen ? 'rotate-180' : ''"
        />
      </button>
      <div v-if="advancedOpen" class="space-y-4 p-3 pt-0">
        <BaseCheckbox id="account-verify-certificate" v-model="config.advanced.verifyCertificate">
          {{ $t('account.verifyCertificate') }}
        </BaseCheckbox>

        <div class="space-y-1.5">
          <label for="account-ca-cert" class="text-sm text-secondary">{{
            $t('account.caCertPath')
          }}</label>
          <input
            id="account-ca-cert"
            :value="config.advanced.caCertPath ?? ''"
            type="text"
            class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
            :placeholder="t('account.caCertPathPlaceholder')"
            @input="config.advanced.caCertPath = ($event.target as HTMLInputElement).value || null"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label for="account-connect-timeout" class="text-sm text-secondary">{{
              $t('account.connectTimeout')
            }}</label>
            <input
              id="account-connect-timeout"
              v-model.number="config.advanced.connectTimeoutSecs"
              type="number"
              min="1"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none focus:border-accent"
            />
          </div>
          <div class="space-y-1.5">
            <label for="account-read-timeout" class="text-sm text-secondary">{{
              $t('account.readTimeout')
            }}</label>
            <input
              id="account-read-timeout"
              v-model.number="config.advanced.readTimeoutSecs"
              type="number"
              min="1"
              class="h-9 w-full rounded-md border border-border bg-elevated px-3 text-sm text-primary outline-none focus:border-accent"
            />
          </div>
        </div>

        <BaseCheckbox id="account-keepalive" v-model="config.advanced.keepalive">
          {{ $t('account.keepalive') }}
        </BaseCheckbox>
      </div>
    </div>

    <p v-if="validationError || accountStore.error" class="text-sm text-danger">
      {{ validationError || accountStore.error }}
    </p>
    <p v-else-if="testMessage" class="text-sm text-success">{{ testMessage }}</p>

    <div class="flex gap-2">
      <button
        type="button"
        class="h-9 flex-1 rounded-md border border-border bg-base text-sm font-medium text-primary transition-colors hover:bg-raised disabled:opacity-50"
        :disabled="testing"
        @click="handleTest"
      >
        {{ testing ? $t('common.loading') : $t('account.testConnection') }}
      </button>
      <button
        v-if="mode === 'edit'"
        type="button"
        class="h-9 flex-1 rounded-md border border-border bg-base text-sm font-medium text-primary transition-colors hover:bg-raised"
        @click="handleCancel"
      >
        {{ $t('common.cancel') }}
      </button>
    </div>

    <button
      type="submit"
      class="h-9 w-full rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover"
    >
      {{ mode === 'edit' ? $t('account.saveAccount') : $t('account.addAccount') }}
    </button>
  </form>
</template>
