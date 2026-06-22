<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { RouterLink, useRoute, useRouter } from 'vue-router';
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
import { useMailStore } from '@/stores/mail';

const aiStore = useAiStore();
const route = useRoute();
const router = useRouter();

const { t } = useI18n();
const accountStore = useAccountStore();
const mailStore = useMailStore();

onMounted(() => {
  void accountStore.loadAccounts();
});

// Get unread count for a folder by name
function getUnreadCount(folderName: string): number | null {
  const folder = mailStore.folders.find((f) => f.name === folderName || f.path === folderName);
  return folder?.unreadCount || null;
}

const folders = computed(() => [
  { id: 'inbox', name: t('folders.inbox'), icon: Inbox, count: getUnreadCount('INBOX'), path: '/' },
  { id: 'starred', name: t('folders.starred'), icon: Star, count: null, path: '/folder/starred' },
  { id: 'sent', name: t('folders.sent'), icon: Send, count: null, path: '/folder/sent' },
  {
    id: 'drafts',
    name: t('folders.drafts'),
    icon: FileText,
    count: getUnreadCount('Drafts'),
    path: '/drafts',
  },
  {
    id: 'archived',
    name: t('folders.archived'),
    icon: Archive,
    count: null,
    path: '/folder/archived',
  },
  {
    id: 'spam',
    name: t('folders.spam'),
    icon: Trash2,
    count: getUnreadCount('[Gmail]/Spam'),
    path: '/folder/spam',
  },
]);

function isActiveFolder(path: string) {
  if (path === '/') return route.path === '/';
  return route.path === path;
}

function isActivePage(path: string) {
  return route.path === path;
}

function handleCompose() {
  void router.push({ name: 'compose' });
}
</script>

<template>
  <aside class="flex h-full flex-col border-r border-border bg-panel">
    <div class="flex h-12 items-center px-4 text-lg font-semibold">
      {{ $t('app.name') }}
    </div>

    <div class="px-3 pb-2">
      <button
        type="button"
        class="flex h-10 w-full items-center justify-center gap-2 rounded-lg bg-primary text-sm font-medium text-white transition-colors hover:bg-primary-hover active:bg-primary-active"
        @click="handleCompose"
      >
        <Plus class="h-4 w-4" />
        {{ $t('mail.newMail') }}
      </button>
    </div>

    <nav class="flex-1 overflow-y-auto px-2">
      <ul class="space-y-0.5">
        <li v-for="folder in folders" :key="folder.id">
          <RouterLink
            :to="folder.path"
            class="flex h-9 items-center justify-between rounded-md px-3 text-sm transition-colors"
            :class="
              isActiveFolder(folder.path)
                ? 'bg-card text-text'
                : 'text-text-secondary hover:bg-card/50'
            "
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
          </RouterLink>
        </li>
      </ul>

      <div class="my-3 h-px bg-border" />

      <div class="px-3 pb-2 text-xs font-medium text-muted">{{ $t('sidebar.accounts') }}</div>
      <ul class="space-y-0.5">
        <li
          v-for="account in accountStore.accounts"
          :key="account.id"
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-card/50"
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
        <li>
          <RouterLink
            to="/accounts"
            class="flex h-9 items-center gap-3 rounded-md px-3 text-sm transition-colors"
            :class="
              isActivePage('/accounts')
                ? 'bg-card text-text'
                : 'text-text-secondary hover:bg-card/50'
            "
          >
            <Users class="h-4 w-4" />
            <span>{{ $t('nav.accounts') }}</span>
          </RouterLink>
        </li>
        <li>
          <RouterLink
            to="/settings"
            class="flex h-9 items-center gap-3 rounded-md px-3 text-sm transition-colors"
            :class="
              isActivePage('/settings')
                ? 'bg-card text-text'
                : 'text-text-secondary hover:bg-card/50'
            "
          >
            <Settings class="h-4 w-4" />
            <span>{{ $t('nav.settings') }}</span>
          </RouterLink>
        </li>
        <li
          class="flex h-9 cursor-pointer items-center gap-3 rounded-md px-3 text-sm text-text-secondary transition-colors hover:bg-card/50"
          @click="aiStore.togglePanel()"
        >
          <Bot class="h-4 w-4" />
          <span>{{ $t('aiAssistant.title') }}</span>
        </li>
      </ul>
    </nav>
  </aside>
</template>
