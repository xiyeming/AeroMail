<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { FolderInput } from '@lucide/vue';
import { useMailStore } from '@/stores/mail';

const { t } = useI18n();
const mailStore = useMailStore();

const emit = defineEmits<{
  close: [];
  move: [targetFolderId: string];
}>();

const dialogRef = ref<HTMLDivElement | null>(null);

const FOLDER_NAME_MAP: Record<string, string> = {
  inbox: 'nav.inbox',
  starred: 'nav.starred',
  sent: 'nav.sent',
  drafts: 'nav.drafts',
  archived: 'nav.archived',
  spam: 'nav.spam',
  trash: 'nav.trash',
};

function getFolderDisplayName(folder: { name: string; path: string }): string {
  const lower = folder.name.toLowerCase();
  const lowerPath = folder.path.toLowerCase();
  for (const [key, i18nKey] of Object.entries(FOLDER_NAME_MAP)) {
    if (lower === key || lowerPath === key || lowerPath.includes(key)) {
      return t(i18nKey);
    }
  }
  return folder.name;
}

const folders = computed(() => {
  return mailStore.folders.filter((f) => f.id !== mailStore.currentFolderId);
});

function handleMove(folderId: string) {
  emit('move', folderId);
}

function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  return Array.from(container.querySelectorAll(selector)).filter(
    (el) => !el.hasAttribute('disabled') && (el as HTMLElement).offsetParent !== null
  ) as HTMLElement[];
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    emit('close');
    return;
  }
  if (e.key !== 'Tab' || !dialogRef.value) return;

  const focusable = getFocusableElements(dialogRef.value);
  if (focusable.length === 0) return;
  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (e.shiftKey && document.activeElement === first) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && document.activeElement === last) {
    e.preventDefault();
    first.focus();
  }
}

watch(
  () => true,
  async () => {
    await nextTick();
    const focusable = dialogRef.value ? getFocusableElements(dialogRef.value) : [];
    focusable[0]?.focus();
  },
  { immediate: true }
);
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-overlay"
      @click="emit('close')"
    >
      <div
        ref="dialogRef"
        role="dialog"
        aria-modal="true"
        aria-labelledby="move-title"
        aria-describedby="move-desc"
        class="w-80 rounded-lg bg-elevated p-4 shadow-lg"
        tabindex="-1"
        @keydown="handleKeyDown"
        @click.stop
      >
        <h3 id="move-title" class="text-lg font-medium text-primary">{{ t('mail.moveTo') }}</h3>
        <p id="move-desc" class="mt-2 text-sm text-secondary">{{ t('mail.selectFolder') }}</p>

        <div class="mt-4 max-h-60 overflow-y-auto">
          <button
            v-for="folder in folders"
            :key="folder.id"
            type="button"
            class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
            @click="handleMove(folder.id)"
          >
            <FolderInput class="h-4 w-4" />
            <span>{{ getFolderDisplayName(folder) }}</span>
            <span v-if="folder.unreadCount > 0" class="ml-auto text-xs text-tertiary">{{
              folder.unreadCount
            }}</span>
          </button>
        </div>

        <div class="mt-4 flex justify-end">
          <button
            type="button"
            class="flex h-9 items-center justify-center rounded-md border border-border px-3 text-sm text-secondary transition-colors hover:bg-raised"
            @click="emit('close')"
          >
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
