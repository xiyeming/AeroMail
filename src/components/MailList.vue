<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute, useRouter } from 'vue-router';
import { useDebounceFn } from '@vueuse/core';
import {
  Star,
  Archive,
  Paperclip,
  Check,
  MailOpen,
  Mail,
  Trash2,
  X,
  CheckSquare,
  Loader2,
} from '@lucide/vue';
import { useMailStore } from '@/stores/mail';
import { useAccountStore } from '@/stores/account';
import { useToastStore } from '@/stores/toast';
import { useStatusStore } from '@/stores/status';
import ContextMenu from '@/components/ContextMenu.vue';
import MoveMailDialog from '@/components/MoveMailDialog.vue';
import CustomScrollbar from '@/components/CustomScrollbar.vue';
import type { MailHeader } from '@/types/mail';

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const mailStore = useMailStore();
const accountStore = useAccountStore();
const toastStore = useToastStore();
const statusStore = useStatusStore();

const VIRTUAL_FOLDER_IDS = ['starred', 'sent', 'archived', 'spam'];

const currentFolderId = computed(() => {
  const folderId = route.params.folderId as string;
  if (folderId) return folderId;
  const inbox = mailStore.folders.find(
    (f) => f.path.toLowerCase() === 'inbox' || f.name.toLowerCase() === 'inbox'
  );
  return inbox?.id || 'inbox';
});

const currentFolderName = computed(() => {
  const folderId = route.params.folderId as string | undefined;
  if (!folderId) return t('folders.inbox');
  if (VIRTUAL_FOLDER_IDS.includes(folderId)) {
    return t(`folders.${folderId}`);
  }
  const folder = mailStore.folders.find((f) => f.id === folderId || f.path === folderId);
  return folder?.name || t('folders.inbox');
});

const contextMenu = ref<{
  show: boolean;
  x: number;
  y: number;
  mail: MailHeader | null;
}>({
  show: false,
  x: 0,
  y: 0,
  mail: null,
});

const showMoveDialog = ref(false);
const moveTargetMailId = ref<string | null>(null);
const searchQuery = ref('');
const debouncedSearchQuery = ref('');
const scrollbarRef = ref<InstanceType<typeof CustomScrollbar> | null>(null);
const sentinelRef = ref<HTMLElement | null>(null);
let scrollObserver: IntersectionObserver | null = null;
let sentinelObserved = false;

const updateDebouncedSearch = useDebounceFn((value: string) => {
  debouncedSearchQuery.value = value;
}, 300);

function getScrollElement(): HTMLElement | null {
  return scrollbarRef.value?.getScrollElement() ?? null;
}

function scrollToTop() {
  scrollbarRef.value?.scrollToTop();
}

onMounted(async () => {
  if (accountStore.accounts.length === 0) {
    await accountStore.loadAccounts();
  }
  for (const accountId of accountStore.selectedAccountIds) {
    await mailStore.loadFolders(accountId);
  }
});

watch(
  () => [route.path, accountStore.selectedAccountIds.join(',')],
  async ([path]) => {
    mailStore.closeReader();
    scrollToTop();
    if (path === '/') {
      await mailStore.loadInboxMails(accountStore.selectedAccountIds);
    } else if (currentFolderId.value) {
      await mailStore.loadMails(currentFolderId.value);
    }
    await checkLoadMore();
  },
  { immediate: true }
);

watch(
  () => statusStore.syncingAccounts,
  async (syncing, previous) => {
    if (previous && previous > 0 && syncing === 0) {
      const accountIds =
        accountStore.selectedAccountIds.length > 0
          ? accountStore.selectedAccountIds
          : accountStore.accounts.map((a) => a.id);
      for (const accountId of accountIds) {
        await mailStore.loadFolders(accountId);
      }
      if (route.path === '/') {
        await mailStore.refreshInboxMails(accountStore.selectedAccountIds);
      } else if (currentFolderId.value) {
        await mailStore.refreshMails(currentFolderId.value);
      }
    }
  }
);

const currentFolderUnread = computed(() => {
  if (route.path === '/') {
    return mailStore.folders
      .filter(
        (f) =>
          accountStore.selectedAccountIds.includes(f.accountId) &&
          (f.path.toLowerCase() === 'inbox' || f.name.toLowerCase() === 'inbox')
      )
      .reduce((sum, f) => sum + f.unreadCount, 0);
  }
  const folder = mailStore.folders.find((f) => f.id === currentFolderId.value);
  return folder?.unreadCount ?? 0;
});

watch(
  currentFolderUnread,
  (count) => {
    statusStore.setUnreadCount(count);
  },
  { immediate: true }
);

const displayedMails = computed(() => {
  const query = debouncedSearchQuery.value.trim();
  if (!query) return mailStore.mails;
  const q = query.toLowerCase();
  return mailStore.mails.filter(
    (m) =>
      (m.subject ?? '').toLowerCase().includes(q) ||
      (m.fromName ?? '').toLowerCase().includes(q) ||
      (m.fromAddress ?? '').toLowerCase().includes(q)
  );
});

const hasSearchResults = computed(() => {
  const query = debouncedSearchQuery.value.trim();
  if (!query) return true;
  return displayedMails.value.length > 0;
});

const selectedCount = computed(() => mailStore.selectedMailIds.length);
const hasSelection = computed(() => selectedCount.value > 0);

function formatDate(timestamp: number | null): string {
  if (!timestamp) return '';
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

  if (diffDays === 0) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } else if (diffDays === 1) {
    return t('mail.yesterday');
  } else if (diffDays < 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
  }
}

function getAccountName(accountId: string): string {
  return accountStore.accounts.find((a) => a.id === accountId)?.name ?? '';
}

function isReadingRoute(): boolean {
  return route.path === '/' || route.path.startsWith('/folder/');
}

function ensureReadingRoute() {
  if (!isReadingRoute()) {
    void router.push('/');
  }
}

function loadMoreMails() {
  if (!mailStore.hasMore || mailStore.loadingMore || mailStore.loading) return;
  void mailStore.loadMails(currentFolderId.value, false);
}

async function checkLoadMore() {
  await nextTick();
  const el = getScrollElement();
  if (!el) return;
  if (
    el.scrollHeight <= el.clientHeight + 50 &&
    mailStore.hasMore &&
    !mailStore.loading &&
    !mailStore.loadingMore
  ) {
    await loadMoreMails();
  }
}

function disconnectObserver() {
  if (scrollObserver) {
    scrollObserver.disconnect();
    scrollObserver = null;
    sentinelObserved = false;
  }
}

function setupInfiniteScroll() {
  const sentinel = sentinelRef.value;
  if (!sentinel) return;

  if (scrollObserver && sentinelObserved) {
    return;
  }

  disconnectObserver();

  scrollObserver = new IntersectionObserver(
    (entries) => {
      if (
        entries[0]?.isIntersecting &&
        mailStore.hasMore &&
        !mailStore.loadingMore &&
        !mailStore.loading
      ) {
        void loadMoreMails();
      }
    },
    {
      root: getScrollElement(),
      rootMargin: '0px 0px 200px 0px',
    }
  );
  scrollObserver.observe(sentinel);
  sentinelObserved = true;
}

onMounted(() => {
  void setupInfiniteScroll();
});

onUnmounted(() => {
  disconnectObserver();
});

watch(
  () => mailStore.mails.length,
  async () => {
    await nextTick();
    setupInfiniteScroll();
    await checkLoadMore();
  }
);

function handleContextMenu(e: MouseEvent, mail: MailHeader) {
  e.preventDefault();

  const offset = 2;
  let x = e.clientX + offset;
  let y = e.clientY + offset;

  const container = getScrollElement();
  if (container) {
    const rect = container.getBoundingClientRect();
    const menuHeight = 250;
    const padding = 12;
    const safeBottom = window.innerHeight - padding;

    // Keep the menu within the scroll container's horizontal bounds when possible;
    // the ContextMenu component will clamp against the viewport right edge.
    x = Math.max(rect.left + padding, x);

    // Anchor at cursor: open above when the menu would go off screen.
    if (y + menuHeight > safeBottom) {
      y = Math.max(padding, y - menuHeight);
    }
  }

  contextMenu.value = {
    show: true,
    x,
    y,
    mail,
  };
}

function handleRowClick(e: MouseEvent, mail: MailHeader) {
  if (e.ctrlKey || e.metaKey || e.shiftKey) {
    mailStore.toggleSelection(mail.id, e.ctrlKey || e.metaKey, e.shiftKey);
    return;
  }
  // 普通点击：若当前处于批量选择模式，先退出该模式，再打开邮件详情
  if (mailStore.selectedMailIds.length > 0) {
    mailStore.clearBulkSelection();
  }
  void mailStore.selectMail(mail.id);
  ensureReadingRoute();
}

function handleRowKeyDown(e: KeyboardEvent, mail: MailHeader) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    void mailStore.selectMail(mail.id);
    ensureReadingRoute();
  }
}

function handleCheckboxChange(mailId: string, e: Event) {
  const event = e as MouseEvent;
  if (event.shiftKey && mailStore.lastSelectedIndex >= 0) {
    mailStore.toggleSelection(mailId, false, true);
  } else if (event.ctrlKey || event.metaKey) {
    mailStore.toggleSelection(mailId, true, false);
  } else if (isSelected(mailId)) {
    mailStore.removeFromSelection(mailId);
  } else {
    mailStore.addToSelection(mailId);
  }
}

function toggleStar(mailId: string) {
  void mailStore.toggleStar(mailId);
}

function archiveMail(mailId: string) {
  void mailStore.archiveMail(mailId);
  toastStore.add({ type: 'success', message: t('mail.archived') });
}

function closeContextMenu() {
  contextMenu.value.show = false;
}

function handleContextOpen(mailId: string) {
  mailStore.selectMail(mailId);
  ensureReadingRoute();
}

function handleContextStar(mailId: string) {
  mailStore.toggleStar(mailId);
}

function handleContextToggleRead(mailId: string, isRead: boolean) {
  mailStore.markRead(mailId, isRead);
}

function handleContextArchive(mailId: string) {
  mailStore.archiveMail(mailId);
  toastStore.add({ type: 'success', message: t('mail.archived') });
}

function handleContextSpam(mailId: string) {
  mailStore.toggleSpam(mailId);
  toastStore.add({ type: 'success', message: t('mail.markedAsSpam') });
}

function handleContextDelete(mailId: string) {
  mailStore.deleteMail(mailId);
  toastStore.add({ type: 'success', message: t('mail.deleted') });
}

function handleContextMove(mailId: string) {
  moveTargetMailId.value = mailId;
  showMoveDialog.value = true;
}

async function handleMoveToFolder(folderId: string) {
  if (moveTargetMailId.value) {
    await mailStore.moveMail(moveTargetMailId.value, folderId);
    toastStore.add({ type: 'success', message: t('mail.moved') });
  }
  showMoveDialog.value = false;
  moveTargetMailId.value = null;
}

function isSelected(mailId: string): boolean {
  return mailStore.selectedMailIds.includes(mailId);
}

function deselectAll() {
  mailStore.clearBulkSelection();
}

function invertSelection() {
  mailStore.invertSelection();
}

async function bulkArchive() {
  for (const mailId of mailStore.selectedMailIds) {
    await mailStore.archiveMail(mailId);
  }
  mailStore.clearBulkSelection();
  toastStore.add({ type: 'success', message: t('mail.archived') });
}

async function bulkDelete() {
  await mailStore.bulkDelete();
  toastStore.add({ type: 'success', message: t('mail.deleted') });
}

async function bulkMarkRead(isRead: boolean) {
  await mailStore.bulkMarkRead(isRead);
  toastStore.add({
    type: 'success',
    message: isRead ? t('mail.markedAsRead') : t('mail.markedAsUnread'),
  });
}
</script>

<template>
  <div class="flex h-full flex-col bg-elevated">
    <!-- Header -->
    <div class="flex h-12 shrink-0 items-center justify-between border-b border-border px-4">
      <span class="font-medium text-primary">{{ currentFolderName }}</span>
      <form role="search" class="relative" @submit.prevent>
        <label for="mail-search" class="sr-only">{{ t('common.search') }}</label>
        <input
          id="mail-search"
          :value="searchQuery"
          type="search"
          :placeholder="t('commandPalette.placeholder')"
          class="h-8 w-44 rounded-md border border-border bg-base px-3 text-sm text-primary placeholder:text-tertiary outline-none focus:border-accent"
          @input="updateDebouncedSearch(($event.target as HTMLInputElement).value); searchQuery = ($event.target as HTMLInputElement).value"
        />
      </form>
    </div>

    <!-- Loading state -->
    <div
      v-if="mailStore.loading && mailStore.mails.length === 0"
      class="flex flex-1 items-center justify-center text-secondary"
    >
      <div class="flex items-center gap-2">
        <div
          class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent"
        />
        {{ t('mail.loading') }}
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="displayedMails.length === 0"
      class="flex flex-1 items-center justify-center text-secondary"
    >
      {{ hasSearchResults.value ? t('mail.noEmails') : t('mail.noSearchResults') }}
    </div>

    <!-- Mail list -->
    <CustomScrollbar v-else ref="scrollbarRef" class="flex-1">
      <div role="list">
        <div
          v-for="mail in displayedMails"
          :key="mail.id"
          v-memo="[mail.id, mail.isRead, mail.isStarred, mail.isArchived, mail.isSpam, mail.hasAttachments, mailStore.selectedMailId, mailStore.deletingMailIds.has(mail.id)]"
          role="button"
          tabindex="0"
          :aria-selected="mailStore.selectedMailId === mail.id"
          class="group relative cursor-pointer border-b border-border px-3 py-2.5 transition-colors hover:bg-raised focus:outline-none focus:ring-2 focus:ring-inset focus:ring-accent content-visibility-auto"
          :class="[
            !mail.isRead ? 'bg-accent/15' : '',
            isSelected(mail.id) ? 'bg-accent-subtle' : '',
            mailStore.selectedMailId === mail.id ? 'border-l-4 border-l-accent' : '',
            !mail.isRead && mailStore.selectedMailId !== mail.id
              ? 'border-l-4 border-l-accent'
              : '',
            mailStore.deletingMailIds.has(mail.id) ? 'pointer-events-none opacity-60' : '',
          ]"
          @click="handleRowClick($event, mail)"
          @keydown="handleRowKeyDown($event, mail)"
          @contextmenu="handleContextMenu($event, mail)"
        >
          <div class="flex items-center gap-3">
            <label
              class="relative flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center rounded border transition-colors hover:border-accent peer-focus-within:ring-2 peer-focus-within:ring-accent"
              :class="
                isSelected(mail.id)
                  ? 'border-accent bg-accent text-white'
                  : 'border-border bg-base text-transparent'
              "
            >
              <input
                type="checkbox"
                :checked="isSelected(mail.id)"
                class="peer sr-only"
                :aria-label="t('mail.select')"
                @click.stop
                @change="handleCheckboxChange(mail.id, $event)"
              />
              <Check class="h-3 w-3" />
            </label>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <span
                    class="truncate text-sm"
                    :class="!mail.isRead ? 'font-bold text-primary' : 'font-medium text-secondary'"
                  >
                    {{ mail.fromName || mail.fromAddress || t('mail.unknownSender') }}
                  </span>
                  <span
                    v-if="accountStore.accounts.length > 1"
                    class="shrink-0 rounded bg-elevated px-1.5 py-0.5 text-[10px] text-tertiary"
                  >
                    {{ getAccountName(mail.accountId) }}
                  </span>
                </div>
                <span class="shrink-0 text-xs text-tertiary">{{ formatDate(mail.date) }}</span>
              </div>
              <div
                class="mt-0.5 truncate text-sm"
                :class="!mail.isRead ? 'font-bold text-primary' : 'text-secondary'"
              >
                {{ mail.subject || t('mail.noSubject') }}
              </div>
              <div class="mt-0.5 flex items-center gap-2">
                <span class="truncate text-xs text-tertiary">
                  {{ mail.fromAddress }}
                </span>
                <Paperclip
                  v-if="mail.hasAttachments"
                  class="h-3.5 w-3.5 shrink-0 text-tertiary"
                  :aria-label="t('mail.hasAttachments')"
                />
              </div>
            </div>

            <!-- Hover quick actions -->
            <div class="hidden shrink-0 items-center gap-1 group-hover:flex">
              <template v-if="mailStore.deletingMailIds.has(mail.id)">
                <Loader2 class="h-4 w-4 animate-spin text-accent" :title="t('mail.deleting')" />
              </template>
              <template v-else>
                <button
                  type="button"
                  class="rounded p-1 text-secondary transition-colors hover:bg-raised hover:text-primary"
                  :title="mail.isStarred ? t('mail.unstar') : t('mail.star')"
                  @click.stop="toggleStar(mail.id)"
                >
                  <Star
                    class="h-4 w-4"
                    :class="mail.isStarred ? 'fill-warning text-warning' : ''"
                  />
                </button>
                <button
                  type="button"
                  class="rounded p-1 text-secondary transition-colors hover:bg-raised hover:text-primary"
                  :title="t('mail.archive')"
                  @click.stop="archiveMail(mail.id)"
                >
                  <Archive class="h-4 w-4" />
                </button>
              </template>
            </div>
          </div>
        </div>

        <!-- Infinite-scroll sentinel -->
        <div ref="sentinelRef" class="h-2 w-full" aria-hidden="true" />

        <!-- Load more indicator -->
        <div
          v-if="mailStore.loadingMore"
          class="flex items-center justify-center gap-2 py-4 text-secondary"
        >
          <div
            class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent"
          />
          <span class="text-xs">{{ t('common.loading') }}</span>
        </div>

        <!-- End of list -->
        <div
          v-else-if="!mailStore.hasMore && displayedMails.length > 0"
          class="py-4 text-center text-xs text-tertiary"
        >
          {{ t('mail.noMoreEmails') }}
        </div>
      </div>
    </CustomScrollbar>

    <!-- Bulk actions -->
    <div
      v-if="hasSelection"
      class="flex shrink-0 flex-wrap items-center justify-between gap-2 border-t border-border bg-raised px-3 py-2"
    >
      <span class="text-sm text-secondary whitespace-nowrap">{{
        t('mail.selectedCount', { count: selectedCount })
      }}</span>
      <div class="flex flex-wrap items-center gap-1.5">
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-secondary transition-colors hover:bg-elevated hover:text-primary"
          @click="bulkMarkRead(true)"
        >
          <MailOpen class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('mail.markAsRead') }}</span>
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-secondary transition-colors hover:bg-elevated hover:text-primary"
          @click="bulkMarkRead(false)"
        >
          <Mail class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('mail.markAsUnread') }}</span>
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-secondary transition-colors hover:bg-elevated hover:text-primary"
          @click="bulkArchive"
        >
          <Archive class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('mail.archive') }}</span>
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-danger transition-colors hover:bg-danger-subtle"
          @click="bulkDelete"
        >
          <Trash2 class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('common.delete') }}</span>
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-secondary transition-colors hover:bg-elevated hover:text-primary"
          @click="invertSelection"
        >
          <CheckSquare class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('mail.invertSelection') }}</span>
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1 rounded-md px-2 py-1.5 text-sm text-tertiary transition-colors hover:bg-elevated hover:text-primary"
          @click="deselectAll"
        >
          <X class="h-4 w-4" />
          <span class="hidden sm:inline">{{ t('mail.deselectAll') }}</span>
        </button>
      </div>
    </div>

    <!-- Context menu -->
    <ContextMenu
      v-if="contextMenu.show && contextMenu.mail"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :mail-id="contextMenu.mail.id"
      :is-read="contextMenu.mail.isRead"
      :is-starred="contextMenu.mail.isStarred"
      :is-archived="contextMenu.mail.isArchived"
      :is-spam="contextMenu.mail.isSpam"
      @close="closeContextMenu"
      @open="handleContextOpen"
      @star="handleContextStar"
      @toggle-read="handleContextToggleRead"
      @delete="handleContextDelete"
      @move="handleContextMove"
      @archive="handleContextArchive"
      @spam="handleContextSpam"
    />

    <!-- Move dialog -->
    <MoveMailDialog
      v-if="showMoveDialog"
      @close="showMoveDialog = false"
      @move="handleMoveToFolder"
    />
  </div>
</template>

<style scoped>
.content-visibility-auto {
  content-visibility: auto;
  contain-intrinsic-size: 0 80px;
}
</style>
