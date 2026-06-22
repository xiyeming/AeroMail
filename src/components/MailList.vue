<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute } from 'vue-router';
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

const currentFolderId = computed(() => {
  const folderId = route.params.folderId as string;
  return folderId || 'inbox';
});

const currentFolderName = computed(() => {
  const folderId = route.params.folderId as string | undefined;
  if (!folderId) return t('nav.inbox');
  const folder = mailStore.folders.find((f) => f.id === folderId || f.path === folderId);
  return folder?.name || t('nav.inbox');
});

// Context menu state
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

// Single-mail move dialog state
const showMoveDialog = ref(false);
const moveTargetMailId = ref<string | null>(null);

// Load accounts on mount
onMounted(async () => {
  if (accountStore.accounts.length === 0) {
    await accountStore.loadAccounts();
  }
  // Load folders for the first account
  if (accountStore.accounts.length > 0) {
    const accountId = accountStore.accounts[0].id;
    await mailStore.loadFolders(accountId);
  }
});

// Load mails when folder changes
watch(
  currentFolderId,
  async (folderId) => {
    if (folderId) {
      await mailStore.loadMails(folderId);
    }
  },
  { immediate: true }
);

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

function handleClick(e: MouseEvent, mail: MailHeader) {
  mailStore.toggleSelection(mail.id, e.ctrlKey || e.metaKey, e.shiftKey);
}

function handleDoubleClick(mail: MailHeader) {
  mailStore.selectMail(mail.id);
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
</script>

<template>
  <div class="flex h-full flex-col bg-panel">
    <div class="flex h-12 items-center border-b border-border px-4 font-medium text-text">
      {{ currentFolderName }}
    </div>

    <!-- Loading state -->
    <div
      v-if="mailStore.loading && mailStore.mails.length === 0"
      class="flex flex-1 items-center justify-center text-muted"
    >
      <div class="flex items-center gap-2">
        <div
          class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
        />
        {{ t('mail.loading') }}
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="mailStore.mails.length === 0"
      class="flex flex-1 items-center justify-center text-muted"
    >
      {{ t('mail.noEmails') }}
    </div>

    <!-- Mail list -->
    <div v-else class="flex-1 overflow-y-auto" @scroll="handleScroll">
      <div
        v-for="mail in mailStore.mails"
        :key="mail.id"
        class="group relative cursor-pointer border-b border-border px-4 py-3 transition-colors hover:bg-card"
        :class="[
          !mail.isRead ? 'bg-card/30' : '',
          isSelected(mail.id) ? 'bg-primary/10' : '',
          mailStore.selectedMailId === mail.id ? 'border-l-2 border-l-primary' : '',
        ]"
        @click="handleClick($event, mail)"
        @dblclick="handleDoubleClick(mail)"
        @contextmenu="handleContextMenu($event, mail)"
      >
        <div class="flex items-center gap-2">
          <!-- Checkbox -->
          <input
            type="checkbox"
            :checked="isSelected(mail.id)"
            class="h-4 w-4 rounded border-border text-primary focus:ring-primary"
            @click.stop
            @change="
              mailStore.toggleSelection(mail.id, $event.ctrlKey || $event.metaKey, $event.shiftKey)
            "
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <span
                  v-if="!mail.isRead"
                  class="h-1.5 w-1.5 flex-shrink-0 rounded-full bg-primary"
                />
                <span
                  class="text-sm font-medium"
                  :class="!mail.isRead ? 'text-text' : 'text-text-secondary'"
                >
                  {{ mail.fromName || mail.fromAddress || t('mail.unknownSender') }}
                </span>
              </div>
              <span class="flex-shrink-0 text-xs text-muted">{{ formatDate(mail.date) }}</span>
            </div>
            <div
              class="mt-1 truncate text-sm"
              :class="!mail.isRead ? 'text-text' : 'text-text-secondary'"
            >
              {{ mail.subject || t('mail.noSubject') }}
            </div>
            <div class="mt-0.5 flex items-center gap-2">
              <span class="truncate text-xs text-muted">
                {{ mail.fromAddress }}
              </span>
              <span
                v-if="mail.hasAttachments"
                class="flex-shrink-0 text-muted"
                :title="t('mail.hasAttachments')"
              >
                📎
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Load more indicator -->
      <div v-if="mailStore.loadingMore" class="flex items-center justify-center py-4 text-muted">
        <div
          class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
        />
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
      @close="closeContextMenu"
      @open="handleContextOpen"
      @star="handleContextStar"
      @toggle-read="handleContextToggleRead"
      @delete="handleContextDelete"
      @move="handleContextMove"
    />

    <!-- Move dialog -->
    <MoveMailDialog
      v-if="showMoveDialog"
      @close="showMoveDialog = false"
      @move="handleMoveToFolder"
    />
  </div>
</template>
