<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Star, Trash2, FolderInput, Eye, EyeOff } from 'lucide-vue-next';

const { t } = useI18n();

const props = defineProps<{
  x: number;
  y: number;
  mailId: string;
  isRead: boolean;
  isStarred: boolean;
}>();

const emit = defineEmits<{
  close: [];
  open: [mailId: string];
  star: [mailId: string];
  toggleRead: [mailId: string, isRead: boolean];
  delete: [mailId: string];
  move: [mailId: string];
}>();

const menuRef = ref<HTMLDivElement | null>(null);

// 调整菜单位置避免超出屏幕
const menuStyle = computed(() => {
  const menuWidth = 200;
  const menuHeight = 250;
  const padding = 10;

  let x = props.x;
  let y = props.y;

  // 右侧超出
  if (x + menuWidth > window.innerWidth - padding) {
    x = window.innerWidth - menuWidth - padding;
  }
  // 底部超出
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
  if (e.key === 'Escape') {
    emit('close');
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside, true);
  document.addEventListener('keydown', handleKeydown, true);
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

function handleDelete() {
  emit('delete', props.mailId);
  emit('close');
}

function handleMove() {
  emit('move', props.mailId);
  emit('close');
}
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="fixed z-50 min-w-[180px] rounded-lg border border-border bg-panel py-1 shadow-lg"
      :style="menuStyle"
    >
      <button
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-text-secondary hover:bg-card"
        @click="handleOpen"
      >
        <Eye class="h-4 w-4" />
        {{ t('contextMenu.open') }}
      </button>

      <button
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-text-secondary hover:bg-card"
        @click="handleStar"
      >
        <Star class="h-4 w-4" :class="{ 'text-yellow-500': isStarred }" />
        {{ isStarred ? t('contextMenu.unstar') : t('contextMenu.star') }}
      </button>

      <button
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-text-secondary hover:bg-card"
        @click="handleToggleRead"
      >
        <EyeOff v-if="isRead" class="h-4 w-4" />
        <Eye v-else class="h-4 w-4" />
        {{ isRead ? t('contextMenu.markUnread') : t('contextMenu.markRead') }}
      </button>

      <div class="my-1 h-px bg-border" />

      <button
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-text-secondary hover:bg-card"
        @click="handleMove"
      >
        <FolderInput class="h-4 w-4" />
        {{ t('contextMenu.moveTo') }}
      </button>

      <div class="my-1 h-px bg-border" />

      <button
        class="flex w-full items-center gap-3 px-3 py-2 text-sm text-red-500 hover:bg-red-500/10"
        @click="handleDelete"
      >
        <Trash2 class="h-4 w-4" />
        {{ t('contextMenu.delete') }}
      </button>
    </div>
  </Teleport>
</template>
