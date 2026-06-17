<script setup lang="ts">
import { ref } from 'vue';
import { useAccountStore } from '@/stores/account';
import type { AccountConfig, MailProvider } from '@/types/account';

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
  await accountStore.addAccount(config.value);
}
</script>

<template>
  <form class="space-y-4 p-4" @submit.prevent="handleSubmit">
    <h2 class="text-lg font-semibold">Add Account</h2>

    <div>
      <label class="mb-1 block text-sm text-muted">Provider</label>
      <select
        v-model="config.provider"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        @change="updateProvider(config.provider)"
      >
        <option v-for="p in providers" :key="p" :value="p">{{ p }}</option>
      </select>
    </div>

    <div>
      <label class="mb-1 block text-sm text-muted">Account Name</label>
      <input
        v-model="config.name"
        type="text"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        placeholder="Work Gmail"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1 block text-sm text-muted">IMAP Host</label>
        <input
          v-model="config.imap.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
      <div>
        <label class="mb-1 block text-sm text-muted">IMAP Port</label>
        <input
          v-model.number="config.imap.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="mb-1 block text-sm text-muted">SMTP Host</label>
        <input
          v-model="config.smtp.host"
          type="text"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
      <div>
        <label class="mb-1 block text-sm text-muted">SMTP Port</label>
        <input
          v-model.number="config.smtp.port"
          type="number"
          class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        />
      </div>
    </div>

    <div>
      <label class="mb-1 block text-sm text-muted">Password</label>
      <input
        type="password"
        class="h-10 w-full rounded-md border border-border bg-card px-3 text-sm outline-none focus:border-primary"
        placeholder="App password"
      />
    </div>

    <button
      type="submit"
      class="h-10 w-full rounded-md bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover"
    >
      Add Account
    </button>

    <p v-if="accountStore.error" class="text-sm text-danger">{{ accountStore.error }}</p>
  </form>
</template>
