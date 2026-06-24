<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <div class="flex flex-wrap items-center gap-1 border-b border-border bg-elevated/50 p-2">
      <!-- Mode switcher -->
      <div class="mr-2 inline-flex rounded-md border border-border bg-base p-0.5">
        <button
          v-for="modeOption in modeOptions"
          :key="modeOption.value"
          type="button"
          class="rounded px-2 py-1 text-xs transition-colors"
          :class="
            mode === modeOption.value
              ? 'bg-accent-subtle text-primary'
              : 'text-secondary hover:bg-raised hover:text-primary'
          "
          @click="setMode(modeOption.value)"
        >
          {{ modeOption.label }}
        </button>
      </div>

      <template v-if="mode === 'rich'">
        <button
          v-for="btn in richToolbarButtons"
          :key="btn.key"
          type="button"
          class="rounded-md px-2 py-1 text-sm text-secondary transition-colors hover:bg-raised hover:text-primary"
          :class="{ 'bg-accent-subtle text-primary': btn.active }"
          :title="btn.title"
          @click="btn.action"
        >
          {{ btn.label }}
        </button>
      </template>

      <template v-if="mode === 'markdown'">
        <button
          v-for="btn in markdownToolbarButtons"
          :key="btn.key"
          type="button"
          class="rounded-md px-2 py-1 text-sm text-secondary transition-colors hover:bg-raised hover:text-primary"
          :title="btn.title"
          @click="btn.action"
        >
          {{ btn.label }}
        </button>
      </template>
    </div>

    <EditorContent
      v-if="mode === 'rich'"
      :editor="editor"
      class="flex-1 overflow-auto bg-base p-4"
    />

    <textarea
      v-else-if="mode === 'markdown'"
      ref="textRef"
      v-model="markdownText"
      class="flex-1 resize-none overflow-auto bg-base p-4 text-sm text-primary outline-none"
      :placeholder="t('compose.editorPlaceholder')"
      @input="onMarkdownInput"
    />

    <textarea
      v-else
      ref="textRef"
      v-model="plainText"
      class="flex-1 resize-none overflow-auto bg-base p-4 text-sm text-primary outline-none"
      :placeholder="t('compose.editorPlaceholder')"
      @input="onPlainInput"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useTiptap } from '@/composables/useTiptap';
import { renderMarkdown, plainToHtml, htmlToPlain, htmlToMarkdown } from '@/utils/markdown';

type EditorMode = 'rich' | 'markdown' | 'plain';

const { t } = useI18n();

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'change', value: { html: string; text: string }): void;
  (e: 'image-pasted', file: File): void;
}>();

const mode = ref<EditorMode>('rich');
const richHtml = ref('');
const markdownText = ref('');
const plainText = ref('');
const textRef = ref<HTMLTextAreaElement | null>(null);

function emitUpdate(html: string, text: string) {
  emit('update:modelValue', html);
  emit('change', { html, text });
}

function initializeFromHtml(html: string) {
  richHtml.value = html;
  markdownText.value = htmlToMarkdown(html);
  plainText.value = htmlToPlain(html);
}

initializeFromHtml(props.modelValue || '');

watch(
  () => props.modelValue,
  (value) => {
    const html = value || '';
    if (richHtml.value === html) return;
    initializeFromHtml(html);
    if (mode.value === 'rich' && editor.value) {
      editor.value.commands.setContent(html, false);
    }
  }
);

function setMode(next: EditorMode) {
  if (mode.value === next) return;

  if (next === 'rich') {
    const html = mode.value === 'markdown'
      ? renderMarkdown(markdownText.value)
      : plainToHtml(plainText.value);
    mode.value = next;
    richHtml.value = html;
    nextTick(() => {
      editor.value?.commands.setContent(html, false);
      const text = editor.value?.getText() ?? htmlToPlain(html);
      emitUpdate(html, text);
    });
    return;
  }

  if (next === 'markdown') {
    const md = mode.value === 'plain' ? plainText.value : htmlToMarkdown(richHtml.value);
    markdownText.value = md;
    mode.value = next;
    const html = renderMarkdown(md);
    richHtml.value = html;
    emitUpdate(html, md);
    return;
  }

  // next === 'plain'
  const text = mode.value === 'markdown' ? markdownText.value : htmlToPlain(richHtml.value);
  plainText.value = text;
  mode.value = next;
  const html = plainToHtml(text);
  richHtml.value = html;
  emitUpdate(html, text);
}

function onMarkdownInput() {
  const html = renderMarkdown(markdownText.value);
  richHtml.value = html;
  emitUpdate(html, markdownText.value);
}

function onPlainInput() {
  const html = plainToHtml(plainText.value);
  richHtml.value = html;
  emitUpdate(html, plainText.value);
}

const { editor, EditorContent, isActive, toolbarActions } = useTiptap({
  content: computed(() => richHtml.value),
  placeholder: t('compose.editorPlaceholder'),
  onUpdate: (html, text) => {
    richHtml.value = html;
    emitUpdate(html, text);
  },
  onImagePasted: (file) => emit('image-pasted', file),
});

const modeOptions = computed(() => [
  { value: 'rich' as EditorMode, label: t('compose.richText') },
  { value: 'markdown' as EditorMode, label: t('compose.markdown') },
  { value: 'plain' as EditorMode, label: t('compose.plainText') },
]);

const richToolbarButtons = computed(() => [
  {
    key: 'bold',
    label: 'B',
    title: t('compose.bold'),
    active: isActive('bold'),
    action: toolbarActions.value.bold,
  },
  {
    key: 'italic',
    label: 'I',
    title: t('compose.italic'),
    active: isActive('italic'),
    action: toolbarActions.value.italic,
  },
  {
    key: 'underline',
    label: 'U',
    title: t('compose.underline'),
    active: isActive('underline'),
    action: toolbarActions.value.underline,
  },
  {
    key: 'strike',
    label: 'S',
    title: t('compose.strike'),
    active: isActive('strike'),
    action: toolbarActions.value.strike,
  },
  {
    key: 'code',
    label: '</>',
    title: t('compose.code'),
    active: isActive('code'),
    action: toolbarActions.value.code,
  },
  {
    key: 'h1',
    label: 'H1',
    title: t('compose.heading1'),
    active: isActive('heading', { level: 1 }),
    action: toolbarActions.value.h1,
  },
  {
    key: 'h2',
    label: 'H2',
    title: t('compose.heading2'),
    active: isActive('heading', { level: 2 }),
    action: toolbarActions.value.h2,
  },
  {
    key: 'bulletList',
    label: '• List',
    title: t('compose.bulletList'),
    active: isActive('bulletList'),
    action: toolbarActions.value.bulletList,
  },
  {
    key: 'orderedList',
    label: '1. List',
    title: t('compose.orderedList'),
    active: isActive('orderedList'),
    action: toolbarActions.value.orderedList,
  },
  {
    key: 'blockquote',
    label: '"',
    title: t('compose.quote'),
    active: isActive('blockquote'),
    action: toolbarActions.value.blockquote,
  },
  {
    key: 'link',
    label: t('compose.link'),
    title: t('compose.link'),
    active: isActive('link'),
    action: () => {
      const url = window.prompt(t('compose.linkPrompt'));
      if (url) toolbarActions.value.link(url);
    },
  },
  {
    key: 'undo',
    label: '↶',
    title: t('compose.undo'),
    active: false,
    action: toolbarActions.value.undo,
  },
  {
    key: 'redo',
    label: '↷',
    title: t('compose.redo'),
    active: false,
    action: toolbarActions.value.redo,
  },
]);

interface MarkdownToolbarButton {
  key: string;
  label: string;
  title: string;
  wrap?: string;
  prefix?: string;
  action: () => void;
}

const markdownToolbarButtons = computed<MarkdownToolbarButton[]>(() => [
  { key: 'bold', label: 'B', title: t('compose.bold'), wrap: '**', action: () => insertMarkdownSyntax('bold') },
  { key: 'italic', label: 'I', title: t('compose.italic'), wrap: '*', action: () => insertMarkdownSyntax('italic') },
  { key: 'strike', label: 'S', title: t('compose.strike'), wrap: '~~', action: () => insertMarkdownSyntax('strike') },
  { key: 'code', label: '</>', title: t('compose.code'), wrap: '`', action: () => insertMarkdownSyntax('code') },
  { key: 'h1', label: 'H1', title: t('compose.heading1'), prefix: '# ', action: () => insertMarkdownSyntax('h1') },
  { key: 'h2', label: 'H2', title: t('compose.heading2'), prefix: '## ', action: () => insertMarkdownSyntax('h2') },
  { key: 'quote', label: '"', title: t('compose.quote'), prefix: '> ', action: () => insertMarkdownSyntax('quote') },
  { key: 'bulletList', label: '• List', title: t('compose.bulletList'), prefix: '- ', action: () => insertMarkdownSyntax('bulletList') },
  { key: 'orderedList', label: '1. List', title: t('compose.orderedList'), prefix: '1. ', action: () => insertMarkdownSyntax('orderedList') },
]);

function insertMarkdownSyntax(key: string) {
  const button = markdownToolbarButtons.value.find((b) => b.key === key);
  if (!button) return;
  const textarea = textRef.value;
  if (!textarea) return;

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const value = markdownText.value;
  const selected = value.slice(start, end);

  if (button.wrap) {
    const before = value.slice(0, start);
    const after = value.slice(end);
    const wrapped = `${button.wrap}${selected}${button.wrap}`;
    markdownText.value = before + wrapped + after;
    nextTick(() => {
      textarea.selectionStart = start + button.wrap!.length;
      textarea.selectionEnd = end + button.wrap!.length;
      textarea.focus();
    });
    onMarkdownInput();
    return;
  }

  if (button.prefix) {
    const before = value.slice(0, start);
    const after = value.slice(end);
    const lines = selected.split('\n');
    const prefixed = lines.map((line) => `${button.prefix}${line}`).join('\n');
    markdownText.value = before + prefixed + after;
    nextTick(() => {
      textarea.selectionStart = start;
      textarea.selectionEnd = start + prefixed.length;
      textarea.focus();
    });
    onMarkdownInput();
  }
}
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
</style>
