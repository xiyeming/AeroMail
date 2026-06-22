<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAccountStore } from '@/stores/account';
import type { AccountConfig, MailProvider } from '@/types/account';

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

const providerDefaults: Record<MailProvider, { imap: string; smtp: string }> = {
  Gmail: { imap: 'imap.gmail.com', smtp: 'smtp.gmail.com' },
  Outlook: { imap: 'outlook.office365.com', smtp: 'smtp.office365.com' },
  QQ: { imap: 'imap.qq.com', smtp: 'smtp.qq.com' },
  Netease163: { imap: 'imap.163.com', smtp: 'smtp.163.com' },
  Aliyun: { imap: 'imap.aliyun.com', smtp: 'smtp.aliyun.com' },
  TencentExmail: { imap: 'imap.exmail.qq.com', smtp: 'smtp.exmail.qq.com' },
  Custom: { imap: '', smtp: '' },
};

const config = ref<AccountConfig>({
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

const password = ref('');
const validationError = ref<string | null>(null);

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
  if (config.value.auth.type === 'Password' && !password.value) {
    validationError.value = t('account.errors.passwordRequired');
    return false;
  }
  validationError.value = null;
  return true;
}

async function handleSubmit() {
  if (!validate()) return;

  if (config.value.auth.type === 'Password') {
    config.value.auth.passwordEncrypted = Array.from(new TextEncoder().encode(password.value));
  }

  if (!config.value.email) {
    config.value.email = config.value.name;
  }
  await accountStore.addAccount(config.value);
  if (!accountStore.error) {
    password.value = '';
  }
}
</script>

<template>
  <form
    class="space-y-5 rounded-lg border border-border bg-elevated p-5"
    @submit.prevent="handleSubmit"
  >
    <h2 class="text-lg font-medium text-primary">{{ $t('account.addAccount') }}</h2>

    <div class="space-y-1.5">
      <label for="account-provider" class="text-sm text-secondary">{{ $t('account.provider') }}</label>
      <select
        id="account-provider"
        v-model="config.provider"
        class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        @change="updateProvider(config.provider)"
      >
        <option v-for="p in providers" :key="p" :value="p">{{ p }}</option>
      </select>
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
      <label for="account-email" class="text-sm text-secondary">{{ $t('account.emailAddress') }}</label>
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
        <label for="account-imap-host" class="text-sm text-secondary">{{ $t('account.imapHost') }}</label>
        <input
          id="account-imap-host"
          v-model="config.imap.host"
          type="text"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
      <div class="space-y-1.5">
        <label for="account-imap-port" class="text-sm text-secondary">{{ $t('account.imapPort') }}</label>
        <input
          id="account-imap-port"
          v-model.number="config.imap.port"
          type="number"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div class="space-y-1.5">
        <label for="account-smtp-host" class="text-sm text-secondary">{{ $t('account.smtpHost') }}</label>
        <input
          id="account-smtp-host"
          v-model="config.smtp.host"
          type="text"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
      <div class="space-y-1.5">
        <label for="account-smtp-port" class="text-sm text-secondary">{{ $t('account.smtpPort') }}</label>
        <input
          id="account-smtp-port"
          v-model.number="config.smtp.port"
          type="number"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
        />
      </div>
    </div>

    <div class="space-y-1.5">
      <label for="account-password" class="text-sm text-secondary">{{ $t('account.password') }}</label>
      <input
        id="account-password"
        v-model="password"
        type="password"
        class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
        :placeholder="t('account.passwordPlaceholder')"
      />
    </div>

    <p v-if="validationError" class="text-sm text-danger">{{ validationError }}</p>

    <button
      type="submit"
      class="h-9 w-full rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover"
    >
      {{ $t('account.addAccount') }}
    </button>

    <p v-if="accountStore.error" class="text-sm text-danger">{{ accountStore.error }}</p>
  </form>
</template>
