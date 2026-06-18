<script setup lang="ts">
import { onMounted } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import { useAccountStore } from '@/stores/account';

const accountStore = useAccountStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

async function remove(id: string) {
  await accountStore.deleteAccount(id);
}
</script>

<template>
  <div class="rounded-lg border border-border bg-card p-5">
    <h2 class="mb-3 text-h1 font-semibold text-text">{{ $t('nav.accounts') }}</h2>
    <div v-if="accountStore.loading" class="py-4 text-center text-sm text-muted">{{ $t('common.loading') }}</div>
    <div v-else-if="accountStore.accounts.length === 0" class="py-4 text-center text-sm text-muted">
      {{ $t('account.noAccounts') }}
    </div>
    <ul v-else class="space-y-2">
      <li
        v-for="account in accountStore.accounts"
        :key="account.id"
        class="flex items-center justify-between rounded-md border border-border bg-panel px-4 py-3 transition-colors hover:border-border-hover"
      >
        <div class="flex items-center gap-3">
          <div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-medium text-primary">
            {{ account.name.charAt(0).toUpperCase() }}
          </div>
          <div>
            <div class="text-sm font-medium text-text">{{ account.name }}</div>
            <div class="text-xs text-muted">{{ account.imapHost }}</div>
          </div>
        </div>
        <button class="rounded-md p-1.5 text-muted transition-colors hover:bg-danger/10 hover:text-danger" @click="remove(account.id)">
          <Trash2 class="h-4 w-4" />
        </button>
      </li>
    </ul>
    <p v-if="accountStore.error" class="mt-2 text-sm text-danger">
      {{ accountStore.error }}
    </p>
  </div>
</template>
