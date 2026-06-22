<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { FolderInput } from 'lucide-vue-next';
import { useMailStore } from '@/stores/mail';

const { t } = useI18n();
const mailStore = useMailStore();

const emit = defineEmits<{
  close: [];
  move: [targetFolderId: string];
}>();

const folders = computed(() => {
  return mailStore.folders.filter((f) => f.id !== mailStore.currentFolderId);
});

function handleMove(folderId: string) {
  emit('move', folderId);
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-overlay"
      @click="emit('close')"
    >
      <div class="w-80 rounded-lg bg-panel p-4 shadow-lg" @click.stop>
        <h3 class="text-lg font-medium text-text">{{ t('mail.moveTo') }}</h3>
        <p class="mt-2 text-sm text-text-secondary">{{ t('mail.selectFolder') }}</p>

        <div class="mt-4 max-h-60 overflow-y-auto">
          <button
            v-for="folder in folders"
            :key="folder.id"
            class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm text-text-secondary hover:bg-card"
            @click="handleMove(folder.id)"
          >
            <FolderInput class="h-4 w-4" />
            <span>{{ folder.name }}</span>
            <span class="ml-auto text-xs text-muted">{{ folder.unreadCount }}</span>
          </button>
        </div>

        <div class="mt-4 flex justify-end">
          <button
            class="rounded-md border border-border px-3 py-1.5 text-sm text-text-secondary hover:bg-card"
            @click="emit('close')"
          >
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
