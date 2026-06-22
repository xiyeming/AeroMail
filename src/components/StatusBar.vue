<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useStatusStore } from '@/stores/status';

const { t } = useI18n();
const statusStore = useStatusStore();

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return t('statusBar.syncing', { count: syncing, total: statusStore.syncStatus.length });
  }
  return t('statusBar.syncComplete');
});
</script>

<template>
  <div
    class="flex h-8 shrink-0 items-center border-t border-border bg-elevated px-4 text-xs text-secondary"
  >
    <div class="flex items-center gap-2">
      <span
        :class="[
          'h-2 w-2 rounded-full',
          statusStore.syncingAccounts > 0 ? 'animate-pulse bg-accent' : 'bg-success',
        ]"
        aria-hidden="true"
      />
      <span>{{ syncText }}</span>
    </div>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>{{ t('statusBar.unread', { count: statusStore.unreadCount }) }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span class="text-tertiary">{{
      t('statusBar.lastSync', { time: statusStore.lastSyncTime ?? t('statusBar.never') })
    }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span :class="statusStore.isOnline ? 'text-success' : 'text-warning'">
      {{ statusStore.isOnline ? t('statusBar.online') : t('statusBar.offline') }}
    </span>
    <span class="ml-auto text-tertiary">{{ t('statusBar.version', { version: '0.1.0' }) }}</span>
  </div>
</template>
