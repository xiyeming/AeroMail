import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AccountConfig, AccountSummary } from '@/types/account';

export const useAccountStore = defineStore('account', () => {
  const { call } = useTauriInvoke();
  const accounts = ref<AccountSummary[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const accountCount = computed(() => accounts.value.length);

  async function loadAccounts() {
    loading.value = true;
    error.value = null;
    try {
      accounts.value = await call<AccountSummary[]>('list_accounts');
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function addAccount(config: AccountConfig) {
    error.value = null;
    try {
      await call<string>('add_account', { config });
      await loadAccounts();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function getAccountConfig(accountId: string) {
    return await call<AccountConfig>('get_account_config', { accountId });
  }

  async function updateAccount(config: AccountConfig, password?: string) {
    error.value = null;
    try {
      const passwordBytes = password ? Array.from(new TextEncoder().encode(password)) : undefined;
      await call<void>('update_account', { config, password: passwordBytes });
      await loadAccounts();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function deleteAccount(accountId: string) {
    error.value = null;
    try {
      await call<void>('delete_account', { accountId });
      await loadAccounts();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function testConnection(config: AccountConfig) {
    return await call<string>('test_account_connection', { config });
  }

  return {
    accounts,
    loading,
    error,
    accountCount,
    loadAccounts,
    addAccount,
    getAccountConfig,
    updateAccount,
    deleteAccount,
    testConnection,
  };
});
