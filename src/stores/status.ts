import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { SyncProgress } from '@/types/mail';

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

  function updateSyncStatus(accountId: string, status: SyncStatusItem['status'], message?: string) {
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

  function setUnreadCount(value: number) {
    unreadCount.value = value;
  }

  // Listen for sync progress events from the backend
  function initEventListeners() {
    listen<SyncProgress>('sync:progress', (event) => {
      const progress = event.payload;
      updateSyncStatus(
        progress.accountId,
        progress.status,
        progress.status === 'error' ? progress.status : undefined
      );
      if (progress.status === 'completed') {
        setLastSyncTime(progress.lastSyncTime);
      }
    });

    // Listen for online/offline events
    window.addEventListener('online', () => setOnline(true));
    window.addEventListener('offline', () => setOnline(false));
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
    setUnreadCount,
    initEventListeners,
  };
});
