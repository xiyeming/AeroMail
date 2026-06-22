<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, type ComponentPublicInstance } from 'vue';
import { useI18n } from 'vue-i18n';
import { Star, Trash2, FolderInput, Eye, EyeOff, Archive, AlertTriangle } from 'lucide-vue-next';

const { t } = useI18n();

const props = defineProps<{
  x: number;
  y: number;
  mailId: string;
  isRead: boolean;
  isStarred: boolean;
  isArchived?: boolean;
  isSpam?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  open: [mailId: string];
  star: [mailId: string];
  toggleRead: [mailId: string, isRead: boolean];
  delete: [mailId: string];
  move: [mailId: string];
  archive: [mailId: string];
  spam: [mailId: string];
}>();

const menuRef = ref<HTMLDivElement | null>(null);
const itemRefs = ref<HTMLButtonElement[]>([]);

const menuStyle = computed(() => {
  const menuWidth = 200;
  const menuHeight = 250;
  const padding = 10;

  let x = props.x;
  let y = props.y;

  if (x + menuWidth > window.innerWidth - padding) {
    x = window.innerWidth - menuWidth - padding;
  }
  if (y + menuHeight > window.innerHeight - padding) {
    y = window.innerHeight - menuHeight - padding;
  }

  return {
    left: `${x}px`,
    top: `${y}px`,
  };
});

function handleClickOutside(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close');
  }
}

function handleKeydown(e: KeyboardEvent) {
  const items = itemRefs.value.filter((el) => !el.disabled && el.offsetParent !== null);
  const currentIndex = items.findIndex((el) => el === document.activeElement);

  if (e.key === 'Escape') {
    e.preventDefault();
    emit('close');
    return;
  }

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    const nextIndex = currentIndex < items.length - 1 ? currentIndex + 1 : 0;
    items[nextIndex]?.focus();
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    const prevIndex = currentIndex > 0 ? currentIndex - 1 : items.length - 1;
    items[prevIndex]?.focus();
  } else if (e.key === 'Home') {
    e.preventDefault();
    items[0]?.focus();
  } else if (e.key === 'End') {
    e.preventDefault();
    items[items.length - 1]?.focus();
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside, true);
  document.addEventListener('keydown', handleKeydown, true);
  void nextTick(() => {
    itemRefs.value[0]?.focus();
  });
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside, true);
  document.removeEventListener('keydown', handleKeydown, true);
});

function handleOpen() {
  emit('open', props.mailId);
  emit('close');
}

function handleStar() {
  emit('star', props.mailId);
  emit('close');
}

function handleToggleRead() {
  emit('toggleRead', props.mailId, !props.isRead);
  emit('close');
}

function handleArchive() {
  if (!props.isArchived) {
    emit('archive', props.mailId);
  }
  emit('close');
}

function handleSpam() {
  emit('spam', props.mailId);
  emit('close');
}

function handleDelete() {
  emit('delete', props.mailId);
  emit('close');
}

function handleMove() {
  emit('move', props.mailId);
  emit('close');
}

function _setItemRef(el: Element | ComponentPublicInstance | null, index: number) {
  if (el instanceof HTMLButtonElement) {
    itemRefs.value[index] = el;
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      role="menu"
      aria-orientation="vertical"
      class="fixed z-50 min-w-44 rounded-lg border border-border bg-elevated py-1 shadow-lg"
      :style="menuStyle"
    >
      <button
        ref="(el) => _setItemRef(el, 0)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleOpen"
      >
        <Eye class="h-4 w-4" />
        {{ t('contextMenu.open') }}
      </button>

      <button
        ref="(el) => _setItemRef(el, 1)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleStar"
      >
        <Star class="h-4 w-4" :class="{ 'text-warning': isStarred }" />
        {{ isStarred ? t('contextMenu.unstar') : t('contextMenu.star') }}
      </button>

      <button
        v-if="!isArchived"
        ref="(el) => _setItemRef(el, 2)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleArchive"
      >
        <Archive class="h-4 w-4" />
        {{ t('contextMenu.archive') }}
      </button>

      <button
        ref="(el) => _setItemRef(el, 3)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleSpam"
      >
        <AlertTriangle class="h-4 w-4" :class="{ 'text-danger': isSpam }" />
        {{ isSpam ? t('contextMenu.notSpam') : t('contextMenu.spam') }}
      </button>

      <button
        ref="(el) => _setItemRef(el, 4)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleToggleRead"
      >
        <EyeOff v-if="isRead" class="h-4 w-4" />
        <Eye v-else class="h-4 w-4" />
        {{ isRead ? t('contextMenu.markUnread') : t('contextMenu.markRead') }}
      </button>

      <div class="my-1 h-px bg-border" />

      <button
        ref="(el) => _setItemRef(el, 5)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-secondary transition-colors hover:bg-raised"
        @click="handleMove"
      >
        <FolderInput class="h-4 w-4" />
        {{ t('contextMenu.moveTo') }}
      </button>

      <div class="my-1 h-px bg-border" />

      <button
        ref="(el) => _setItemRef(el, 6)"
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-danger transition-colors hover:bg-danger-subtle"
        @click="handleDelete"
      >
        <Trash2 class="h-4 w-4" />
        {{ t('contextMenu.delete') }}
      </button>
    </div>
  </Teleport>
</template>
