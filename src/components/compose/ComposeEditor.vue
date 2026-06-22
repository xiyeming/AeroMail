<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <div class="flex flex-wrap items-center gap-1 border-b border-border bg-panel/50 p-2">
      <button
        v-for="btn in toolbarButtons"
        :key="btn.key"
        type="button"
        class="rounded-md px-2 py-1 text-sm text-text-secondary transition-colors hover:bg-card hover:text-text"
        :class="{ 'bg-primary/20 text-text': btn.active }"
        :title="btn.title"
        @click="btn.action"
      >
        {{ btn.label }}
      </button>
    </div>
    <EditorContent :editor="editor" class="flex-1 overflow-auto bg-background p-4" />
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTiptap } from '@/composables/useTiptap';

const { t } = useI18n();

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'change', value: { html: string; text: string }): void;
  (e: 'image-pasted', file: File): void;
}>();

const { editor, EditorContent, isActive, toolbarActions } = useTiptap({
  content: computed(() => props.modelValue),
  placeholder: t('compose.editorPlaceholder'),
  onUpdate: (html, text) => {
    emit('update:modelValue', html);
    emit('change', { html, text });
  },
  onImagePasted: (file) => emit('image-pasted', file),
});

watch(
  () => props.modelValue,
  (value) => {
    if (editor.value && editor.value.getHTML() !== value) {
      editor.value.commands.setContent(value, false);
    }
  }
);

const toolbarButtons = computed(() => [
  {
    key: 'bold',
    label: 'B',
    title: 'Bold',
    active: isActive('bold'),
    action: toolbarActions.value.bold,
  },
  {
    key: 'italic',
    label: 'I',
    title: 'Italic',
    active: isActive('italic'),
    action: toolbarActions.value.italic,
  },
  {
    key: 'underline',
    label: 'U',
    title: 'Underline',
    active: isActive('underline'),
    action: toolbarActions.value.underline,
  },
  {
    key: 'strike',
    label: 'S',
    title: 'Strike',
    active: isActive('strike'),
    action: toolbarActions.value.strike,
  },
  {
    key: 'h1',
    label: 'H1',
    title: 'Heading 1',
    active: isActive('heading', { level: 1 }),
    action: toolbarActions.value.h1,
  },
  {
    key: 'h2',
    label: 'H2',
    title: 'Heading 2',
    active: isActive('heading', { level: 2 }),
    action: toolbarActions.value.h2,
  },
  {
    key: 'bulletList',
    label: '• List',
    title: 'Bullet List',
    active: isActive('bulletList'),
    action: toolbarActions.value.bulletList,
  },
  {
    key: 'orderedList',
    label: '1. List',
    title: 'Ordered List',
    active: isActive('orderedList'),
    action: toolbarActions.value.orderedList,
  },
  {
    key: 'blockquote',
    label: '"',
    title: 'Quote',
    active: isActive('blockquote'),
    action: toolbarActions.value.blockquote,
  },
  { key: 'undo', label: '↶', title: 'Undo', active: false, action: toolbarActions.value.undo },
  { key: 'redo', label: '↷', title: 'Redo', active: false, action: toolbarActions.value.redo },
]);
</script>

<style scoped>
:deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: var(--muted);
  pointer-events: none;
  height: 0;
}

:deep(.ProseMirror) {
  outline: none;
  min-height: 100%;
}

:deep(.ProseMirror p) {
  margin: 0.5em 0;
}

:deep(.ProseMirror ul),
:deep(.ProseMirror ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

:deep(.ProseMirror blockquote) {
  border-left: 3px solid var(--border);
  padding-left: 1em;
  margin: 0.5em 0;
  color: var(--text-secondary);
}
</style>
