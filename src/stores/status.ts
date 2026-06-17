import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface SyncStatusItem {
  accountId: string;
  status: 'idle' | 'syncing' | 'error' | 'completed';
  message?: string;
}

export const useStatusStore = defineStore('status', () => {
  const syncStatus = ref<SyncStatusItem[]>([]);
  const unreadCount = ref(0);
  const lastSyncTime = ref<string | null>(null);
  const isOnline = ref(true);

  const syncingAccounts = computed(
    () => syncStatus.value.filter((s) => s.status === 'syncing').length
  );

  function updateSyncStatus(
    accountId: string,
    status: SyncStatusItem['status'],
    message?: string
  ) {
    const idx = syncStatus.value.findIndex((s) => s.accountId === accountId);
    if (idx >= 0) {
      syncStatus.value[idx] = { accountId, status, message };
    } else {
      syncStatus.value.push({ accountId, status, message });
    }
  }

  function setOnline(value: boolean) {
    isOnline.value = value;
  }

  function setLastSyncTime(value: string | null) {
    lastSyncTime.value = value;
  }

  return {
    syncStatus,
    unreadCount,
    lastSyncTime,
    isOnline,
    syncingAccounts,
    updateSyncStatus,
    setOnline,
    setLastSyncTime,
  };
});
