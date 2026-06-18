<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { RouterLink } from 'vue-router';
import {
  Inbox,
  Star,
  Send,
  FileText,
  Archive,
  Trash2,
  Settings,
  Plus,
  Users,
  Bot,
} from 'lucide-vue-next';
import { useAccountStore } from '@/stores/account';
import { useAiStore } from '@/stores/ai';

const aiStore = useAiStore();

const { t } = useI18n();
const accountStore = useAccountStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

const folders = computed(() => [
  { name: t('folders.inbox'), icon: Inbox, count: 128 },
  { name: t('folders.starred'), icon: Star, count: 12 },
  { name: t('folders.sent'), icon: Send, count: null },
  { name: t('folders.drafts'), icon: FileText, count: 3 },
  { name: t('folders.archived'), icon: Archive, count: null },
  { name: t('folders.spam'), icon: Trash2, count: null },
]);
</script>

<template>
  <aside class="flex h-full flex-col bg-panel">
    <div class="flex h-12 items-center px-4 text-lg font-semibold">
      {{ $t('app.name') }}
    </div>

    <div class="px-3 pb-2">
      <button
        class="flex h-10 w-full items-center justify-center gap-2 rounded-lg bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover"
      >
        <Plus class="h-4 w-4" />
        {{ $t('mail.newMail') }}
      </button>
    </div>

    <nav class="flex-1 overflow-y-auto px-2">
      <ul class="space-y-0.5">
        <li
          v-for="folder in folders"
          :key="folder.name"
          class="flex h-9 cursor-pointer items-center justify-between rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
        >
          <div class="flex items-center gap-3">
            <component :is="folder.icon" class="h-4 w-4" />
            <span>{{ folder.name }}</span>
          </div>
          <span
            v-if="folder.count"
            class="flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-xs font-medium text-white"
          >
            {{ folder.count }}
          </span>
        </li>
      </ul>

      <div class="my-3 h-px bg-border" />

      <div class="px-3 pb-2 text-xs font-medium text-muted">{{ $t('sidebar.accounts') }}</div>
      <ul class="space-y-0.5">
        <li
          v-for="account in accountStore.accounts"
          :key="account.id"
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
        >
          <div
            class="flex h-6 w-6 items-center justify-center rounded-full bg-card text-xs font-medium"
          >
            {{ account.name.charAt(0).toUpperCase() }}
          </div>
          <span class="truncate">{{ account.name }}</span>
        </li>
      </ul>

      <div class="my-3 h-px bg-border" />

      <ul class="space-y-0.5">
        <li
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
        >
          <RouterLink
            to="/accounts"
            class="flex items-center gap-3"
          >
            <Users class="h-4 w-4" />
            <span>{{ $t('nav.accounts') }}</span>
          </RouterLink>
        </li>
        <li
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
        >
          <RouterLink
            to="/settings"
            class="flex items-center gap-3"
          >
            <Settings class="h-4 w-4" />
            <span>{{ $t('nav.settings') }}</span>
          </RouterLink>
        </li>
        <li
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-white/5"
          @click="aiStore.togglePanel()"
        >
          <Bot class="h-4 w-4" />
          <span>{{ $t('aiAssistant.title') }}</span>
        </li>
      </ul>
    </nav>
  </aside>
</template>
