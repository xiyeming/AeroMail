<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useStatusStore } from '@/stores/status';
import { useLocale, type Locale } from '@/composables/useLocale';

const { t } = useI18n();
const { locale, setLocale, supportedLocales } = useLocale();
const statusStore = useStatusStore();

const showLangMenu = ref(false);

const labels: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '简体中文',
};

const syncText = computed(() => {
  const syncing = statusStore.syncingAccounts;
  if (syncing > 0) {
    return t('statusBar.syncing', { count: syncing, total: statusStore.syncStatus.length });
  }
  return t('statusBar.syncComplete');
});

function selectLanguage(lang: Locale) {
  setLocale(lang);
  showLangMenu.value = false;
}

function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest('.lang-switcher')) {
    showLangMenu.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
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
    <span>{{ t('statusBar.unread', { count: statusStore.unreadCount }) }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span>{{ t('statusBar.lastSync', { time: statusStore.lastSyncTime ?? t('statusBar.never') }) }}</span>
    <div class="mx-3 h-3 w-px bg-border" />
    <span :class="statusStore.isOnline ? 'text-success' : 'text-warning'">
      {{ statusStore.isOnline ? t('statusBar.online') : t('statusBar.offline') }}
    </span>
    <div class="relative ml-auto lang-switcher">
      <button
        class="text-xs text-muted hover:text-text"
        @click="showLangMenu = !showLangMenu"
      >
        {{ t('statusBar.language', { lang: labels[locale as Locale] }) }}
      </button>
      <div
        v-if="showLangMenu"
        class="absolute bottom-full right-0 mb-1 rounded-lg border border-border bg-card py-1 shadow-modal"
      >
        <button
          v-for="loc in supportedLocales"
          :key="loc"
          class="block w-full px-4 py-1 text-left text-sm text-text hover:bg-panel"
          @click="selectLanguage(loc)"
        >
          {{ labels[loc] }}
        </button>
      </div>
    </div>
    <span>v0.1.0</span>
  </div>
</template>
