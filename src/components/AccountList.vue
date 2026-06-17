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
  <div class="p-4">
    <h2 class="mb-3 text-lg font-semibold">Accounts</h2>
    <div v-if="accountStore.loading">Loading...</div>
    <ul v-else class="space-y-2">
      <li
        v-for="account in accountStore.accounts"
        :key="account.id"
        class="flex items-center justify-between rounded-md bg-card px-3 py-2"
      >
        <div>
          <div class="text-sm font-medium">{{ account.name }}</div>
          <div class="text-xs text-muted">{{ account.imapHost }}</div>
        </div>
        <button class="text-muted hover:text-danger" @click="remove(account.id)">
          <Trash2 class="h-4 w-4" />
        </button>
      </li>
    </ul>
    <p v-if="accountStore.error" class="mt-2 text-sm text-danger">
      {{ accountStore.error }}
    </p>
  </div>
</template>
