<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useStatusStore } from '@/stores/status';

const { t, locale } = useI18n();
const statusStore = useStatusStore();

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return t('statusBar.syncing', { count: syncing, total: statusStore.syncStatus.length });
  }
  return t('statusBar.syncComplete');
});

const formattedLastSync = computed(() => {
  const raw = statusStore.lastSyncTime;
  if (!raw) return null;

  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) return raw;

  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();

  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  const isYesterday = date.toDateString() === yesterday.toDateString();

  const time = date.toLocaleTimeString(locale.value, {
    hour: '2-digit',
    minute: '2-digit',
  });

  if (isToday) {
    return t('statusBar.todayAt', { time });
  }
  if (isYesterday) {
    return t('statusBar.yesterdayAt', { time });
  }

  const datePart = date.toLocaleDateString(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
  return `${datePart} ${time}`;
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
      t('statusBar.lastSync', { time: formattedLastSync ?? t('statusBar.never') })
    }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span :class="statusStore.isOnline ? 'text-success' : 'text-warning'">
      {{ statusStore.isOnline ? t('statusBar.online') : t('statusBar.offline') }}
    </span>
    <span class="ml-auto text-tertiary">{{ t('statusBar.version', { version: '0.1.0' }) }}</span>
  </div>
</template>
