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

function updateProvider(provider: MailProvider) {
  config.value.provider = provider;
  const defaults = providerDefaults[provider];
  config.value.imap.host = defaults.imap;
  config.value.smtp.host = defaults.smtp;
}

async function handleSubmit() {
  if (!config.value.email) {
    config.value.email = config.value.name;
  }
  await accountStore.addAccount(config.value);
}
</script>

<template>
  <form
    class="space-y-4 rounded-lg border border-border bg-card p-5"
    @submit.prevent="handleSubmit"
  >
    <h2 class="text-h1 font-semibold text-text">{{ $t('account.addAccount') }}</h2>

    <div>
      <label class="mb-1.5 block text-xs font-medium text-muted">{{
        $t('account.provider')
      }}</label>
      <select
        v-model="config.provider"
        class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors focus:border-primary focus:ring-1 focus:ring-primary/20"
        @change="updateProvider(config.provider)"
      >
        <option v-for="p in providers" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>

    <div>
      <label class="mb-1.5 block text-xs font-medium text-muted">{{
        $t('account.accountName')
      }}</label>
      <input
        v-model="config.name"
        type="text"
        class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors placeholder:text-disabled focus:border-primary focus:ring-1 focus:ring-primary/20"
        :placeholder="t('account.namePlaceholder')"
      />
    </div>

    <div>
      <label class="mb-1.5 block text-xs font-medium text-muted">{{
        $t('account.emailAddress')
      }}</label>
      <input
        v-model="config.email"
        type="email"
        class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors placeholder:text-disabled focus:border-primary focus:ring-1 focus:ring-primary/20"
        :placeholder="t('account.emailPlaceholder')"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1.5 block text-xs font-medium text-muted">{{
          $t('account.imapHost')
        }}</label>
        <input
          v-model="config.imap.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors focus:border-primary focus:ring-1 focus:ring-primary/20"
        />
      </div>
      <div>
        <label class="mb-1.5 block text-xs font-medium text-muted">{{
          $t('account.imapPort')
        }}</label>
        <input
          v-model.number="config.imap.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors focus:border-primary focus:ring-1 focus:ring-primary/20"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1.5 block text-xs font-medium text-muted">{{
          $t('account.smtpHost')
        }}</label>
        <input
          v-model="config.smtp.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors focus:border-primary focus:ring-1 focus:ring-primary/20"
        />
      </div>
      <div>
        <label class="mb-1.5 block text-xs font-medium text-muted">{{
          $t('account.smtpPort')
        }}</label>
        <input
          v-model.number="config.smtp.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors focus:border-primary focus:ring-1 focus:ring-primary/20"
        />
      </div>
    </div>

    <div>
      <label class="mb-1.5 block text-xs font-medium text-muted">{{
        $t('account.password')
      }}</label>
      <input
        type="password"
        class="h-10 w-full rounded-md border border-border bg-panel px-3 text-sm text-text outline-none transition-colors placeholder:text-disabled focus:border-primary focus:ring-1 focus:ring-primary/20"
        :placeholder="t('account.passwordPlaceholder')"
      />
    </div>

    <button
      type="submit"
      class="h-10 w-full rounded-md bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover active:bg-primary-active"
    >
      {{ $t('account.addAccount') }}
    </button>

    <p v-if="accountStore.error" class="text-sm text-danger">{{ accountStore.error }}</p>
  </form>
</template>
