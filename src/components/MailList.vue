<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
import { Star, Archive, Paperclip } from 'lucide-vue-next';
import { useMailStore } from '@/stores/mail';
import { useAccountStore } from '@/stores/account';
import { useToastStore } from '@/stores/toast';
import ContextMenu from '@/components/ContextMenu.vue';
import MoveMailDialog from '@/components/MoveMailDialog.vue';
import type { MailHeader } from '@/types/mail';

const { t } = useI18n();
const route = useRoute();
const mailStore = useMailStore();
const accountStore = useAccountStore();
const toastStore = useToastStore();

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

onMounted(async () => {
  if (accountStore.accounts.length === 0) {
    await accountStore.loadAccounts();
  }
  if (accountStore.accounts.length > 0) {
    const accountId = accountStore.accounts[0].id;
    await mailStore.loadFolders(accountId);
  }
});

watch(
  currentFolderId,
  async (folderId) => {
    if (folderId) {
      await mailStore.loadMails(folderId);
    }
  },
  { immediate: true }
);

const displayedMails = computed(() => {
  if (!searchQuery.value.trim()) return mailStore.mails;
  const q = searchQuery.value.toLowerCase();
  return mailStore.mails.filter(
    (m) =>
      (m.subject ?? '').toLowerCase().includes(q) ||
      (m.fromName ?? '').toLowerCase().includes(q) ||
      (m.fromAddress ?? '').toLowerCase().includes(q)
  );
});

const selectedCount = computed(() => mailStore.selectedMailIds.size);
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

function handleScroll(event: Event) {
  const target = event.target as HTMLElement;
  if (target.scrollTop + target.clientHeight >= target.scrollHeight - 100) {
    if (mailStore.hasMore && !mailStore.loadingMore) {
      mailStore.loadMails(currentFolderId.value, false);
    }
  }
}

function handleContextMenu(e: MouseEvent, mail: MailHeader) {
  e.preventDefault();
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    mail,
  };
}

function handleRowClick(e: MouseEvent, mail: MailHeader) {
  if (e.ctrlKey || e.metaKey || e.shiftKey) {
    mailStore.toggleSelection(mail.id, e.ctrlKey || e.metaKey, e.shiftKey);
    return;
  }
  if (mailStore.selectedMailIds.size > 0) {
    mailStore.toggleSelection(mail.id, false, false);
    return;
  }
  void mailStore.selectMail(mail.id);
}

function handleRowKeyDown(e: KeyboardEvent, mail: MailHeader) {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault();
    void mailStore.selectMail(mail.id);
  }
}

function handleCheckboxChange(mailId: string, e: Event) {
  const event = e as MouseEvent;
  mailStore.toggleSelection(mailId, event.ctrlKey || event.metaKey, event.shiftKey);
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
  return mailStore.selectedMailIds.has(mailId);
}

function selectAll() {
  mailStore.selectAll();
}

function deselectAll() {
  mailStore.clearBulkSelection();
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
      <search class="relative">
        <label for="mail-search" class="sr-only">{{ t('common.search') }}</label>
        <input
          id="mail-search"
          v-model="searchQuery"
          type="search"
          :placeholder="t('commandPalette.placeholder')"
          class="h-8 w-44 rounded-md border border-border bg-base px-3 text-sm text-primary placeholder:text-tertiary outline-none focus:border-accent"
        />
      </search>
    </div>

    <!-- Loading state -->
    <div
      v-if="mailStore.loading && mailStore.mails.length === 0"
      class="flex flex-1 items-center justify-center text-secondary"
    >
      <div class="flex items-center gap-2">
        <div class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent" />
        {{ t('mail.loading') }}
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="displayedMails.length === 0"
      class="flex flex-1 items-center justify-center text-secondary"
    >
      {{ t('mail.noEmails') }}
    </div>

    <!-- Mail list -->
    <div v-else class="flex-1 overflow-y-auto" role="list" @scroll="handleScroll">
      <div
        v-for="mail in displayedMails"
        :key="mail.id"
        role="button"
        tabindex="0"
        :aria-selected="mailStore.selectedMailId === mail.id"
        class="group relative cursor-pointer border-b border-border px-3 py-2.5 transition-colors hover:bg-raised focus:outline-none focus:ring-2 focus:ring-inset focus:ring-accent"
        :class="[
          !mail.isRead ? 'bg-raised/30' : '',
          isSelected(mail.id) ? 'bg-accent-subtle' : '',
          mailStore.selectedMailId === mail.id ? 'border-l-2 border-l-accent' : '',
        ]"
        @click="handleRowClick($event, mail)"
        @keydown="handleRowKeyDown($event, mail)"
        @contextmenu="handleContextMenu($event, mail)"
      >
        <div class="flex items-center gap-3">
          <input
            type="checkbox"
            :checked="isSelected(mail.id)"
            class="h-4 w-4 rounded border-border text-accent focus:ring-accent"
            :aria-label="t('mail.select')"
            @click.stop
            @change="handleCheckboxChange(mail.id, $event)"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2">
              <div class="flex items-center gap-2 min-w-0">
                <span
                  v-if="!mail.isRead"
                  class="h-1.5 w-1.5 shrink-0 rounded-full bg-accent"
                  aria-hidden="true"
                />
                <span
                  class="truncate text-sm font-medium"
                  :class="!mail.isRead ? 'text-primary' : 'text-secondary'"
                >
                  {{ mail.fromName || mail.fromAddress || t('mail.unknownSender') }}
                </span>
              </div>
              <span class="shrink-0 text-xs text-tertiary">{{ formatDate(mail.date) }}</span>
            </div>
            <div
              class="mt-0.5 truncate text-sm"
              :class="!mail.isRead ? 'text-primary' : 'text-secondary'"
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
            <button
              type="button"
              class="rounded p-1 text-secondary transition-colors hover:bg-raised hover:text-primary"
              :title="mail.isStarred ? t('mail.unstar') : t('mail.star')"
              @click.stop="toggleStar(mail.id)"
            >
              <Star class="h-4 w-4" :class="mail.isStarred ? 'fill-warning text-warning' : ''" />
            </button>
            <button
              type="button"
              class="rounded p-1 text-secondary transition-colors hover:bg-raised hover:text-primary"
              :title="t('mail.archive')"
              @click.stop="archiveMail(mail.id)"
            >
              <Archive class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>

      <!-- Load more indicator -->
      <div v-if="mailStore.loadingMore" class="flex items-center justify-center py-4 text-secondary">
        <div class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent" />
      </div>
    </div>

    <!-- Bulk actions -->
    <div
      v-if="hasSelection"
      class="flex shrink-0 items-center justify-between border-t border-border bg-raised px-4 py-2"
    >
      <span class="text-sm text-secondary">{{ t('mail.selectedCount', { count: selectedCount }) }}</span>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="rounded px-2 py-1 text-sm text-secondary transition-colors hover:bg-elevated"
          @click="bulkMarkRead(true)"
        >
          {{ t('mail.markAsRead') }}
        </button>
        <button
          type="button"
          class="rounded px-2 py-1 text-sm text-secondary transition-colors hover:bg-elevated"
          @click="bulkMarkRead(false)"
        >
          {{ t('mail.markAsUnread') }}
        </button>
        <button
          type="button"
          class="rounded px-2 py-1 text-sm text-secondary transition-colors hover:bg-elevated"
          @click="bulkArchive"
        >
          {{ t('mail.archive') }}
        </button>
        <button
          type="button"
          class="rounded px-2 py-1 text-sm text-danger transition-colors hover:bg-danger-subtle"
          @click="bulkDelete"
        >
          {{ t('common.delete') }}
        </button>
        <button
          type="button"
          class="rounded px-2 py-1 text-sm text-tertiary transition-colors hover:bg-elevated"
          @click="deselectAll"
        >
          {{ t('mail.deselectAll') }}
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
