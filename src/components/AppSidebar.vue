<script setup lang="ts">
import { onMounted } from 'vue';
import {
  Inbox,
  Star,
  Send,
  FileText,
  Archive,
  Trash2,
  Settings,
  Plus,
} from 'lucide-vue-next';
import { useAccountStore } from '@/stores/account';

const accountStore = useAccountStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

const folders = [
  { name: 'Inbox', icon: Inbox, count: 128 },
  { name: 'Starred', icon: Star, count: 12 },
  { name: 'Sent', icon: Send, count: null },
  { name: 'Drafts', icon: FileText, count: 3 },
  { name: 'Archived', icon: Archive, count: null },
  { name: 'Spam', icon: Trash2, count: null },
];
</script>

<template>
  <aside class="flex h-full flex-col bg-panel">
    <div class="flex h-12 items-center px-4 text-lg font-semibold">
      AeroMail
    </div>

    <div class="px-3 pb-2">
      <button
        class="flex h-10 w-full items-center justify-center gap-2 rounded-lg bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover"
      >
        <Plus class="h-4 w-4" />
        New Mail
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

      <div class="px-3 pb-2 text-xs font-medium text-muted">ACCOUNTS</div>
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
          <Settings class="h-4 w-4" />
          <span>Settings</span>
        </li>
      </ul>
    </nav>
  </aside>
</template>
