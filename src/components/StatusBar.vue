<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { RefreshCw, Upload } from '@lucide/vue';
import { useStatusStore } from '@/stores/status';
import { useAccountStore } from '@/stores/account';
import { useToastStore } from '@/stores/toast';
import { useTauriInvoke } from '@/composables/useTauriInvoke';

const { t, locale } = useI18n();
const statusStore = useStatusStore();
const accountStore = useAccountStore();
const toastStore = useToastStore();
const { call: invokeCommand } = useTauriInvoke();

const appVersion = ref('0.0.0');

onMounted(async () => {
  try {
    const version = await invokeCommand<string>('get_app_version');
    appVersion.value = version;
  } catch (e) {
    console.error('Failed to get app version:', e);
  }
});

const isSyncing = computed(() => statusStore.syncingAccounts > 0);

async function handleRefresh() {
  console.warn('[StatusBar] refresh clicked, isSyncing:', isSyncing.value);
  if (isSyncing.value) return;

  if (accountStore.accounts.length === 0) {
    try {
      await accountStore.loadAccounts();
    } catch (e) {
      console.error('[StatusBar] failed to load accounts:', e);
      toastStore.add({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
      });
      return;
    }
  }

  const accountId = accountStore.accounts[0]?.id;
  if (!accountId) {
    toastStore.add({ type: 'error', message: t('account.noAccounts') });
    return;
  }

  try {
    await invokeCommand('start_sync', { accountId });
  } catch (e) {
    console.error('[StatusBar] start_sync failed:', e);
    toastStore.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
    });
  }
}

const isSyncingReadFlags = ref(false);

async function handleSyncReadFlags() {
  if (isSyncingReadFlags.value) return;
  console.log('[StatusBar] sync_read_flags: starting');
  isSyncingReadFlags.value = true;
  try {
    console.log('[StatusBar] sync_read_flags: calling backend');
    const count = await invokeCommand<number>('sync_read_flags_to_server');
    console.log('[StatusBar] sync_read_flags: done, count =', count);
    toastStore.add({
      type: 'success',
      message: t('statusBar.readFlagsSynced', { count }),
    });
  } catch (e) {
    console.error('[StatusBar] sync_read_flags: error', e);
    toastStore.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
    });
  } finally {
    isSyncingReadFlags.value = false;
  }
}

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return t('statusBar.syncing', { count: syncing, total: statusStore.syncStatus.length });
  }
  if (statusStore.errorAccounts > 0) {
    return t('statusBar.syncFailed', { count: statusStore.errorAccounts });
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
          statusStore.errorAccounts > 0
            ? 'bg-danger'
            : statusStore.syncingAccounts > 0
              ? 'animate-pulse bg-accent'
              : 'bg-success',
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
    <div class="ml-auto flex items-center gap-2">
      <button
        type="button"
        class="flex h-6 w-6 items-center justify-center rounded text-secondary transition-colors hover:bg-raised hover:text-primary disabled:opacity-50"
        :disabled="isSyncingReadFlags"
        :title="t('statusBar.syncReadFlags')"
        @click="handleSyncReadFlags"
      >
        <Upload class="h-3.5 w-3.5" :class="{ 'animate-pulse': isSyncingReadFlags }" />
      </button>
      <button
        type="button"
        class="flex h-6 w-6 items-center justify-center rounded text-secondary transition-colors hover:bg-raised hover:text-primary disabled:opacity-50"
        :disabled="isSyncing"
        :title="t('statusBar.refresh')"
        @click="handleRefresh"
      >
        <RefreshCw class="h-3.5 w-3.5" :class="{ 'animate-spin': isSyncing }" />
      </button>
      <span class="text-tertiary">{{ t('statusBar.version', { version: appVersion }) }}</span>
    </div>
  </div>
</template>
