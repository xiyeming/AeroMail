<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { X, Trash2, MailOpen, Mail, FolderInput } from 'lucide-vue-next';
import { useMailStore } from '@/stores/mail';
import { useToastStore } from '@/stores/toast';
import MoveMailDialog from '@/components/MoveMailDialog.vue';

const { t } = useI18n();
const mailStore = useMailStore();
const toastStore = useToastStore();

const showMoveDialog = ref(false);

const selectedCount = computed(() => mailStore.selectedMailIds.size);

function handleBulkMarkRead(isRead: boolean) {
  mailStore.bulkMarkRead(isRead);
  toastStore.add({
    type: 'success',
    message: isRead ? t('mail.markedAsRead') : t('mail.markedAsUnread'),
  });
}

function handleBulkDelete() {
  mailStore.bulkDelete();
  toastStore.add({ type: 'success', message: t('mail.deleted') });
}

function handleMove(targetFolderId: string) {
  mailStore.bulkMove(targetFolderId);
  showMoveDialog.value = false;
  toastStore.add({ type: 'success', message: t('mail.moved') });
}
</script>

<template>
  <div
    v-if="selectedCount > 0"
    class="flex items-center gap-4 border-b border-border bg-primary/5 px-4 py-2"
  >
    <span class="text-sm text-text-secondary">
      {{ t('mail.selectedCount', { count: selectedCount }) }}
    </span>
    <div class="flex items-center gap-2">
      <button
        class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
        :title="t('mail.markAsRead')"
        @click="handleBulkMarkRead(true)"
      >
        <MailOpen class="h-3.5 w-3.5" />
      </button>
      <button
        class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
        :title="t('mail.markAsUnread')"
        @click="handleBulkMarkRead(false)"
      >
        <Mail class="h-3.5 w-3.5" />
      </button>
      <button
        class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
        :title="t('mail.moveTo')"
        @click="showMoveDialog = true"
      >
        <FolderInput class="h-3.5 w-3.5" />
      </button>
      <button
        class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-red-500 hover:bg-red-500/10"
        :title="t('mail.delete')"
        @click="handleBulkDelete"
      >
        <Trash2 class="h-3.5 w-3.5" />
      </button>
      <div class="h-4 w-px bg-border" />
      <button
        class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
        :title="t('mail.deselectAll')"
        @click="mailStore.clearBulkSelection()"
      >
        <X class="h-3.5 w-3.5" />
      </button>
    </div>
  </div>

  <!-- Move dialog -->
  <MoveMailDialog v-if="showMoveDialog" @close="showMoveDialog = false" @move="handleMove" />
</template>
