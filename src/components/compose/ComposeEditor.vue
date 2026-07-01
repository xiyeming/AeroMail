<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <div
      v-if="editor"
      class="flex flex-wrap items-center gap-1 border-b border-border bg-elevated/50 p-2"
    >
      <ComposeToolbar
        :editor="editor"
        :ai-loading="aiCompose.isLoading.value"
        @ai="handleAiCompose"
        @image="handleInsertImage"
        @signature="handleInsertSignature"
        @emoji="handleInsertEmoji"
      />
    </div>

    <div class="relative flex-1 overflow-hidden">
      <EditorContent :editor="editor" class="h-full overflow-auto bg-base p-4" />

      <TableFloatingToolbar v-if="editor" :editor="editor" />

      <div
        v-if="aiCompose.isLoading.value"
        class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-base/80 backdrop-blur-sm"
      >
        <Loader2 class="h-8 w-8 animate-spin text-accent" />
        <p class="text-sm font-medium text-primary">{{ t('compose.aiGenerating') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Loader2 } from '@lucide/vue';
import { useTiptap } from '@/composables/useTiptap';
import { useAiCompose } from '@/composables/useAiCompose';
import { useAiStore } from '@/stores/ai';
import { useToastStore } from '@/stores/toast';
import ComposeToolbar from '@/components/compose/ComposeToolbar.vue';
import TableFloatingToolbar from '@/components/compose/TableFloatingToolbar.vue';

const { t } = useI18n();

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'change', value: { html: string; text: string }): void;
  (e: 'image-pasted', file: File): void;
}>();

const aiStore = useAiStore();
const toastStore = useToastStore();
const aiCompose = useAiCompose();

const richHtml = ref(props.modelValue || '');

function emitUpdate(html: string, text: string) {
  emit('update:modelValue', html);
  emit('change', { html, text });
}

const { editor, EditorContent, setContent, insertContent } = useTiptap({
  content: computed(() => richHtml.value),
  placeholder: t('compose.editorPlaceholder'),
  onUpdate: (html, text) => {
    richHtml.value = html;
    emitUpdate(html, text);
  },
  onImagePasted: (file) => emit('image-pasted', file),
});

watch(
  () => props.modelValue,
  (value) => {
    const html = value || '';
    if (richHtml.value === html) return;
    richHtml.value = html;
    setContent(html);
  }
);

watch(
  () => aiCompose.isLoading.value,
  (loading) => {
    editor.value?.setEditable(!loading);
  }
);

async function handleAiCompose(action: 'write' | 'polish' | 'optimize-en') {
  await aiStore.loadProviders();
  await aiStore.loadDefaultProvider();
  const providerId = aiStore.resolveProviderId();
  if (!providerId) {
    toastStore.add({ type: 'error', message: t('compose.aiNoProvider') });
    return;
  }

  const selectedText = editor.value?.state.selection.empty
    ? ''
    : (editor.value?.state.doc.textBetween(
        editor.value.state.selection.from,
        editor.value.state.selection.to,
        ' '
      ) ?? '');
  const content = selectedText || editor.value?.getText() || '';

  if (!content.trim()) {
    toastStore.add({ type: 'warning', message: t('compose.aiEmptyContent') });
    return;
  }

  try {
    const result = await aiCompose.assist(action, content, providerId);
    if (editor.value) {
      if (selectedText) {
        editor.value.chain().focus().insertContent(result).run();
      } else {
        editor.value.commands.setContent(`<p>${result.replace(/\n/g, '</p><p>')}</p>`, {
          emitUpdate: true,
        });
      }
    }
  } catch {
    toastStore.add({ type: 'error', message: t('compose.aiFailed') });
  }
}

function handleInsertImage() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'image/*';
  // 隐藏 input 以兼容 WebKit（Tauri Linux）
  input.style.position = 'fixed';
  input.style.opacity = '0';
  input.style.pointerEvents = 'none';
  input.style.left = '-9999px';
  document.body.appendChild(input);
  input.onchange = () => {
    const file = input.files?.[0];
    if (file) {
      emit('image-pasted', file);
    }
    document.body.removeChild(input);
  };
  input.click();
}

function handleInsertSignature() {
  toastStore.add({ type: 'info', message: t('compose.signatureComingSoon') });
}

function handleInsertEmoji(emoji: string) {
  editor.value?.chain().focus().insertContent(emoji).run();
}

function insertHtml(html: string) {
  insertContent(html);
}

defineExpose({ insertHtml });
</script>

<style scoped>
:deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: var(--text-tertiary);
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

:deep(.ProseMirror table) {
  border-collapse: collapse;
  table-layout: fixed;
  width: fit-content;
  margin: 0;
  min-width: 200px;
}

:deep(.ProseMirror th),
:deep(.ProseMirror td) {
  border: 1px solid var(--border);
  padding: 0.5em;
  text-align: left;
}

:deep(.ProseMirror th) {
  background-color: var(--bg-raised);
  font-weight: 600;
}

:deep(.ProseMirror .selectedCell) {
  background-color: var(--accent-subtle);
}

:deep(.ProseMirror .column-resize-handle) {
  background-color: var(--accent);
  width: 3px;
  opacity: 0.5;
  cursor: col-resize;
  position: relative;
}

:deep(.ProseMirror .column-resize-handle:hover) {
  opacity: 1;
  background-color: var(--accent-hover, var(--accent));
  width: 4px;
  box-shadow: 0 0 6px rgba(59, 130, 246, 0.4);
}

:deep(.ProseMirror .resize-cursor) {
  cursor: col-resize;
}

/* 拖拽进行中：高亮整个表格边框和当前列 */
:deep(.ProseMirror table.column-resize-active) {
  box-shadow: 0 0 0 2px var(--accent);
  border-radius: 4px;
}

:deep(.ProseMirror table.column-resize-active col) {
  transition: none;
}

:deep(.ProseMirror table.column-resize-active td),
:deep(.ProseMirror table.column-resize-active th) {
  transition: background-color 0.15s ease;
}

:deep(.ProseMirror table.column-resize-active .column-resize-handle) {
  opacity: 1;
  background-color: var(--accent);
  width: 4px;
  box-shadow: 0 0 8px rgba(59, 130, 246, 0.5);
}

/* 表格整体样式增强 */
:deep(.ProseMirror table:hover) {
  outline: 1px solid var(--border);
  outline-offset: 2px;
  border-radius: 2px;
}

/* tableWrapper 定位支持 */
:deep(.ProseMirror .tableWrapper) {
  overflow-x: auto;
  margin: 0.5em 0;
  transition: margin 0.2s ease;
}

:deep(.ProseMirror a) {
  color: var(--accent);
  text-decoration: underline;
  cursor: pointer;
}

:deep(.ProseMirror a:hover) {
  color: var(--accent-hover);
}

:deep(.ProseMirror img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
  margin: 0.25em 0;
}

:deep(.ProseMirror img.ProseMirror-selectednode) {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

:deep(.ProseMirror code) {
  background-color: var(--bg-raised);
  border-radius: 3px;
  padding: 0.15em 0.3em;
  font-size: 0.9em;
  font-family: 'JetBrains Mono', ui-monospace, monospace;
}

:deep(.ProseMirror pre) {
  background-color: var(--bg-raised);
  border-radius: 6px;
  padding: 0.75em 1em;
  margin: 0.5em 0;
  overflow-x: auto;
}

:deep(.ProseMirror pre code) {
  background: none;
  padding: 0;
  font-size: inherit;
}
</style>
