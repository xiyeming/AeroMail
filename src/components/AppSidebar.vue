<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
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
  ListTodo,
  ChevronDown,
} from '@lucide/vue';
import { useAccountStore } from '@/stores/account';
import { useMailStore } from '@/stores/mail';
import { useTodoStore } from '@/stores/todo';
import BaseCheckbox from '@/components/BaseCheckbox.vue';
import { decodeModifiedUtf7 } from '@/utils/imapFolderEncoding';
import SkeletonBlock from '@/components/SkeletonBlock.vue';

const route = useRoute();
const router = useRouter();

const { t } = useI18n();
const accountStore = useAccountStore();
const mailStore = useMailStore();
const todoStore = useTodoStore();

onMounted(async () => {
  initializing.value = true;
  await accountStore.loadAccounts();
  initializing.value = false;
});

const popoverOpen = ref(false);
const initializing = ref(true);

function getUnreadCount(folderName: string): number | null {
  const lower = folderName.toLowerCase();
  const matching = mailStore.folders.filter(
    (f) =>
      accountStore.selectedAccountIds.includes(f.accountId) &&
      (decodeModifiedUtf7(f.name).toLowerCase() === lower ||
        decodeModifiedUtf7(f.path).toLowerCase() === lower)
  );
  const total = matching.reduce((sum, f) => sum + f.unreadCount, 0);
  return total > 0 ? total : null;
}

const folders = computed(() => [
  { id: 'inbox', name: t('nav.inbox'), icon: Inbox, count: getUnreadCount('INBOX'), path: '/' },
  { id: 'starred', name: t('nav.starred'), icon: Star, count: null, path: '/folder/starred' },
  { id: 'sent', name: t('nav.sent'), icon: Send, count: null, path: '/folder/sent' },
  {
    id: 'drafts',
    name: t('nav.drafts'),
    icon: FileText,
    count: getUnreadCount('Drafts'),
    path: '/drafts',
  },
  {
    id: 'archived',
    name: t('nav.archived'),
    icon: Archive,
    count: null,
    path: '/folder/archived',
  },
  {
    id: 'spam',
    name: t('nav.spam'),
    icon: Trash2,
    count: getUnreadCount('[Gmail]/Spam') || getUnreadCount('Spam'),
    path: '/folder/spam',
  },
  {
    id: 'trash',
    name: t('nav.trash'),
    icon: Trash2,
    count: null,
    path: '/folder/trash',
  },
]);

const currentAccountLabel = computed(() => {
  if (accountStore.selectedAccountIds.length === 0) {
    return t('account.noAccounts');
  }
  if (accountStore.selectedAccountIds.length === 1) {
    const account = accountStore.accounts.find((a) => a.id === accountStore.selectedAccountIds[0]);
    return account?.name ?? t('account.noAccounts');
  }
  return t('account.selectedCount', { count: accountStore.selectedAccountIds.length });
});

const accountInitial = computed(() => {
  const first = accountStore.accounts.find((a) => accountStore.selectedAccountIds.includes(a.id));
  return first?.name.charAt(0).toUpperCase() ?? 'A';
});

function isActiveFolder(path: string) {
  if (path === '/') return route.path === '/';
  return route.path === path;
}

function handleCompose() {
  void router.push({ name: 'compose' });
}

function goToSettings() {
  popoverOpen.value = false;
  void router.push('/settings');
}

function closePopoverOnBlur(event: FocusEvent) {
  const target = event.relatedTarget as HTMLElement | null;
  const popover = document.getElementById('account-popover');
  if (popover && target && !popover.contains(target)) {
    popoverOpen.value = false;
  }
}
</script>

<template>
  <aside class="flex h-full flex-col bg-elevated">
    <div class="flex h-12 items-center px-4 text-lg font-semibold text-primary">
      {{ $t('app.name') }}
    </div>

    <div class="px-3 pb-3">
      <button
        type="button"
        class="flex h-9 w-full items-center justify-center gap-2 rounded-md bg-accent text-sm font-medium text-white transition-colors hover:bg-accent-hover active:bg-accent-active"
        @click="handleCompose"
      >
        <Plus class="h-4 w-4" />
        {{ $t('mail.newMail') }}
      </button>
    </div>

    <nav class="flex-1 overflow-y-auto px-2" aria-label="Mail folders">
      <ul class="space-y-0.5">
        <template v-if="initializing || (accountStore.loading && accountStore.accounts.length === 0)">
          <li v-for="i in 7" :key="i" class="flex items-center gap-3 px-3 py-2">
            <SkeletonBlock shape="circle" class="h-4 w-4 shrink-0" />
            <SkeletonBlock class="h-4 w-24" />
          </li>
        </template>
        <template v-else>
          <li v-for="folder in folders" :key="folder.id">
            <RouterLink
              :to="folder.path"
              class="flex h-9 items-center justify-between rounded-md px-3 text-sm transition-colors"
              :class="
                isActiveFolder(folder.path)
                  ? 'bg-raised text-primary'
                  : 'text-secondary hover:bg-raised/60'
              "
            >
              <div class="flex items-center gap-3">
                <component :is="folder.icon" class="h-4 w-4" />
                <span>{{ folder.name }}</span>
              </div>
              <span
                v-if="folder.count"
                class="flex h-5 min-w-5 items-center justify-center rounded-full bg-accent px-2 text-xs font-medium text-white"
              >
                {{ folder.count }}
              </span>
            </RouterLink>
          </li>
        </template>
      </ul>

      <div class="mt-2 border-t border-border pt-2">
        <button
          type="button"
          class="flex h-9 w-full items-center gap-3 rounded-md px-3 text-sm text-secondary transition-colors hover:bg-raised/60"
          :class="todoStore.isPanelOpen ? 'bg-raised text-primary' : ''"
          @click="todoStore.togglePanel()"
        >
          <ListTodo class="h-4 w-4" />
          <span>{{ $t('nav.todos') }}</span>
          <span
            v-if="todoStore.pendingItems.length > 0"
            class="ml-auto flex h-5 min-w-5 items-center justify-center rounded-full bg-accent px-2 text-xs font-medium text-white"
          >
            {{ todoStore.pendingItems.length }}
          </span>
        </button>
      </div>
    </nav>

    <div class="border-t border-border p-2">
      <div class="relative">
        <button
          id="account-switcher"
          type="button"
          aria-haspopup="true"
          :aria-expanded="popoverOpen"
          class="flex h-9 w-full items-center gap-2 rounded-md px-2 text-sm transition-colors hover:bg-raised"
          :class="popoverOpen ? 'bg-raised' : ''"
          @click="popoverOpen = !popoverOpen"
          @blur="closePopoverOnBlur"
        >
          <div
            class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-raised text-xs font-medium text-primary"
          >
            {{ accountInitial }}
          </div>
          <span class="flex-1 truncate text-left text-primary" tabindex="-1">{{
            currentAccountLabel
          }}</span>
          <ChevronDown class="h-4 w-4 text-secondary" />
        </button>

        <div
          v-if="popoverOpen"
          id="account-popover"
          class="absolute bottom-full left-0 right-0 mb-1 rounded-md border border-border bg-raised shadow-md"
          role="menu"
        >
          <ul class="py-1">
            <li
              v-for="account in accountStore.accounts"
              :key="account.id"
              class="flex cursor-pointer items-center justify-between gap-2 px-3 py-2 text-sm text-primary transition-colors hover:bg-elevated"
              role="menuitem"
              @click="accountStore.toggleAccountSelection(account.id)"
            >
              <div class="flex min-w-0 items-center gap-2">
                <BaseCheckbox
                  :model-value="accountStore.isAccountSelected(account.id)"
                  class="shrink-0"
                  @update:model-value="accountStore.toggleAccountSelection(account.id)"
                  @click.stop
                />
                <div
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-elevated text-xs font-medium"
                >
                  {{ account.name.charAt(0).toUpperCase() }}
                </div>
                <span class="truncate">{{ account.name }}</span>
              </div>
              <button
                type="button"
                class="shrink-0 text-xs text-secondary transition-colors hover:text-primary"
                @click.stop="accountStore.selectOnlyAccount(account.id)"
              >
                {{ $t('account.showOnly') }}
              </button>
            </li>
            <li v-if="!accountStore.allAccountsSelected" class="border-t border-border">
              <button
                type="button"
                class="w-full px-3 py-2 text-left text-sm text-secondary transition-colors hover:bg-elevated"
                role="menuitem"
                @click="accountStore.selectAllAccounts()"
              >
                {{ $t('account.showAll') }}
              </button>
            </li>
            <li class="border-t border-border">
              <button
                type="button"
                class="w-full px-3 py-2 text-left text-sm text-secondary transition-colors hover:bg-elevated"
                role="menuitem"
                @click="goToSettings"
              >
                {{ $t('settings.title') }}
              </button>
            </li>
          </ul>
        </div>
      </div>

      <RouterLink
        to="/settings"
        class="mt-1 flex h-9 items-center gap-2 rounded-md px-2 text-sm text-secondary transition-colors hover:bg-raised"
        :class="route.path === '/settings' ? 'bg-raised text-primary' : ''"
      >
        <Settings class="h-4 w-4" />
        {{ $t('nav.settings') }}
      </RouterLink>
    </div>
  </aside>
</template>
