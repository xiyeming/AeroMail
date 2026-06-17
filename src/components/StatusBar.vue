<script setup lang="ts">
import { computed } from 'vue';
import { useStatusStore } from '@/stores/status';

const statusStore = useStatusStore();

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return `Syncing... ${syncing}/${statusStore.syncStatus.length} accounts`;
  }
  return 'Sync complete';
});
</script>

<template>
  <div
    class="flex h-7 flex-shrink-0 items-center border-t border-border bg-panel px-4 text-tiny text-muted"
  >
    <div class="flex items-center gap-2">
      <span
        :class="[
          'h-2 w-2 rounded-full',
          statusStore.syncingAccounts > 0 ? 'animate-pulse bg-primary' : 'bg-success',
        ]"
      />
      <span>{{ syncText }}</span>
    </div>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>{{ statusStore.unreadCount }} unread</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>Last sync: {{ statusStore.lastSyncTime ?? 'Never' }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span :class="statusStore.isOnline ? 'text-success' : 'text-warning'">
      {{ statusStore.isOnline ? 'Online' : 'Offline' }}
    </span>
    <span class="ml-auto">v0.1.0</span>
  </div>
</template>
