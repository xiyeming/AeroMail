import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AccountConfig, AccountSummary } from '@/types/account';

const SELECTED_ACCOUNTS_KEY = 'aeromail.selectedAccountIds';

export const useAccountStore = defineStore('account', () => {
  const { call } = useTauriInvoke();
  const accounts = ref<AccountSummary[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedAccountIds = ref<string[]>(loadSelectedAccounts());

  const accountCount = computed(() => accounts.value.length);
  const selectedAccountCount = computed(() => selectedAccountIds.value.length);
  const allAccountsSelected = computed(
    () =>
      accounts.value.length > 0 &&
      selectedAccountIds.value.length === accounts.value.length
  );

  function loadSelectedAccounts(): string[] {
    try {
      const raw = localStorage.getItem(SELECTED_ACCOUNTS_KEY);
      if (raw) {
        const parsed = JSON.parse(raw) as string[];
        return Array.isArray(parsed) ? parsed : [];
      }
    } catch {
      // ignore
    }
    return [];
  }

  function saveSelectedAccounts() {
    localStorage.setItem(SELECTED_ACCOUNTS_KEY, JSON.stringify(selectedAccountIds.value));
  }

  function selectAllAccounts() {
    selectedAccountIds.value = accounts.value.map((a) => a.id);
    saveSelectedAccounts();
  }

  function selectOnlyAccount(accountId: string) {
    selectedAccountIds.value = [accountId];
    saveSelectedAccounts();
  }

  function toggleAccountSelection(accountId: string) {
    const index = selectedAccountIds.value.indexOf(accountId);
    if (index === -1) {
      selectedAccountIds.value.push(accountId);
    } else {
      selectedAccountIds.value.splice(index, 1);
    }
    saveSelectedAccounts();
  }

  function isAccountSelected(accountId: string): boolean {
    return selectedAccountIds.value.includes(accountId);
  }

  async function loadAccounts() {
    loading.value = true;
    error.value = null;
    try {
      accounts.value = await call<AccountSummary[]>('list_accounts');
      if (selectedAccountIds.value.length === 0 && accounts.value.length > 0) {
        selectAllAccounts();
      }
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
      selectAllAccounts();
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
    selectedAccountIds,
    selectedAccountCount,
    allAccountsSelected,
    loadAccounts,
    addAccount,
    getAccountConfig,
    updateAccount,
    deleteAccount,
    testConnection,
    selectAllAccounts,
    selectOnlyAccount,
    toggleAccountSelection,
    isAccountSelected,
  };
});
