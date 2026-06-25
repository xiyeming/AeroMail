<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Editor } from '@tiptap/core';
import {
  Bold,
  Italic,
  Underline,
  Strikethrough,
  Code,
  Subscript,
  Superscript,
  AlignLeft,
  AlignCenter,
  AlignRight,
  AlignJustify,
  List,
  ListOrdered,
  Outdent,
  Indent,
  Link,
  Image,
  Table,
  Minus,
  Quote,
  Undo2,
  Redo2,
  Type,
  Sparkles,
  Loader2,
  Palette,
  Highlighter,
  Smile,
  PenLine,
  ChevronDown,
  Heading1,
  Heading2,
  Pilcrow,
  RemoveFormatting,
  Check,
} from 'lucide-vue-next';
import {
  PopoverContent,
  PopoverRoot,
  PopoverTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRoot,
  DropdownMenuTrigger,
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from 'radix-vue';
import LinkDialog from '@/components/compose/LinkDialog.vue';

const props = defineProps<{
  editor: Editor;
  aiLoading?: boolean;
}>();

const emit = defineEmits<{
  ai: [action: 'write' | 'polish' | 'optimize-en'];
  image: [];
  signature: [];
  emoji: [emoji: string];
}>();

const { t } = useI18n();

interface FontOption {
  value: string;
  label: string;
}

const fontFamilyOptions: FontOption[] = [
  { value: '', label: t('compose.fontDefault') },
  { value: 'Arial, sans-serif', label: 'Arial' },
  { value: '"Helvetica Neue", Helvetica, Arial, sans-serif', label: 'Helvetica' },
  { value: '"Times New Roman", Times, serif', label: 'Times New Roman' },
  { value: 'Georgia, serif', label: 'Georgia' },
  { value: '"Courier New", Courier, monospace', label: 'Courier New' },
  { value: '"Microsoft YaHei", "PingFang SC", sans-serif', label: '微软雅黑' },
];

const fontSizeOptions: FontOption[] = [
  { value: '', label: t('compose.fontSizeDefault') },
  { value: '12px', label: '12' },
  { value: '14px', label: '14' },
  { value: '16px', label: '16' },
  { value: '18px', label: '18' },
  { value: '20px', label: '20' },
  { value: '24px', label: '24' },
  { value: '32px', label: '32' },
];

const textColors = [
  '#000000',
  '#1f2937',
  '#374151',
  '#ef4444',
  '#f97316',
  '#f59e0b',
  '#84cc16',
  '#10b981',
  '#06b6d4',
  '#3b82f6',
  '#6366f1',
  '#8b5cf6',
  '#d946ef',
  '#f43f5e',
];

const highlightColors = [
  '#ffffff',
  '#fef3c7',
  '#fde68a',
  '#d1fae5',
  '#a7f3d0',
  '#bfdbfe',
  '#ddd6fe',
  '#fecaca',
  '#fbcfe8',
  '#e5e7eb',
];

const emojis = [
  '😀',
  '😂',
  '🥰',
  '😎',
  '🤔',
  '😢',
  '😡',
  '👍',
  '👎',
  '👏',
  '🙏',
  '💪',
  '❤️',
  '🎉',
  '🔥',
  '✨',
  '✅',
  '❌',
  '⭐',
  '📎',
  '📧',
  '☕',
  '🌹',
  '🤝',
];

const currentFontFamily = computed(() => {
  const attrs = props.editor.getAttributes('textStyle');
  return (attrs.fontFamily as string) ?? '';
});

const currentFontSize = computed(() => {
  const attrs = props.editor.getAttributes('textStyle');
  return (attrs.fontSize as string) ?? '';
});

const currentFontFamilyLabel = computed(() => {
  return (
    fontFamilyOptions.find((o) => o.value === currentFontFamily.value)?.label ??
    t('compose.fontDefault')
  );
});

const currentFontSizeLabel = computed(() => {
  return (
    fontSizeOptions.find((o) => o.value === currentFontSize.value)?.label ??
    t('compose.fontSizeDefault')
  );
});

const currentColor = computed(() => {
  const attrs = props.editor.getAttributes('textStyle');
  return (attrs.color as string) ?? '';
});

const currentHighlight = computed(() => {
  const attrs = props.editor.getAttributes('highlight');
  return (attrs.color as string) ?? '';
});

function setFontFamily(value: string) {
  if (value) {
    props.editor.chain().focus().setFontFamily(value).run();
  } else {
    props.editor.chain().focus().unsetFontFamily().run();
  }
}

function setFontSize(value: string) {
  if (value) {
    props.editor.chain().focus().setFontSize(value).run();
  } else {
    props.editor.chain().focus().unsetFontSize().run();
  }
}

function clearFormatting() {
  props.editor.chain().focus().unsetAllMarks().clearNodes().run();
}

function setColor(color: string) {
  props.editor.chain().focus().setColor(color).run();
}

function setHighlight(color: string) {
  props.editor.chain().focus().setHighlight({ color }).run();
}

const linkDialogOpen = ref(false);
const editingLinkUrl = ref('');
const editingLinkText = ref('');

function setLink() {
  // Get current link URL if cursor is on a link
  const attrs = props.editor.getAttributes('link');
  editingLinkUrl.value = attrs.href || '';
  // Get selected text or existing link text
  const { from, to } = props.editor.state.selection;
  editingLinkText.value = props.editor.state.doc.textBetween(from, to, '');
  linkDialogOpen.value = true;
}

function onLinkConfirm(url: string, text: string) {
  const { from, to } = props.editor.state.selection;
  const hasSelection = from !== to;

  if (hasSelection) {
    // Replace selected text with linked text
    props.editor
      .chain()
      .focus()
      .deleteRange({ from, to })
      .insertContentAt(from, `<a href="${url}">${text}</a>`)
      .run();
  } else {
    // Insert linked text at cursor
    props.editor
      .chain()
      .focus()
      .insertContent(`<a href="${url}">${text}</a>`)
      .run();
  }
}

function onLinkRemove() {
  props.editor.chain().focus().unsetLink().run();
}

function insertTable() {
  props.editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
}

function insertHorizontalRule() {
  props.editor.chain().focus().setHorizontalRule().run();
}

const aiOpen = ref(false);
const colorOpen = ref(false);
const highlightOpen = ref(false);
const emojiOpen = ref(false);

function runAi(action: 'write' | 'polish' | 'optimize-en') {
  aiOpen.value = false;
  emit('ai', action);
}

function runEmoji(emoji: string) {
  emojiOpen.value = false;
  emit('emoji', emoji);
}

function toolbarBtnClass(active = false) {
  return [
    'flex h-8 w-8 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised hover:text-primary',
    active ? 'bg-accent-subtle text-accent' : '',
  ];
}

function selectTriggerClass() {
  return 'group flex h-8 items-center justify-between gap-1.5 rounded-md border border-border bg-base px-2 text-xs text-primary outline-none transition-colors hover:bg-raised focus:border-accent';
}

function dividerClass() {
  return 'mx-1 h-5 w-px bg-border';
}
</script>

<template>
  <div
    class="flex flex-wrap items-center gap-0.5 border-b border-border bg-elevated/50 px-2 py-1.5"
  >
    <!-- AI dropdown -->
    <DropdownMenuRoot v-model:open="aiOpen">
      <DropdownMenuTrigger as-child>
        <button
          type="button"
          class="flex h-8 items-center gap-1 rounded-md px-2 text-xs font-medium text-accent transition-colors hover:bg-accent-subtle disabled:opacity-50"
          :title="t('compose.aiAssist')"
          :disabled="aiLoading"
        >
          <Loader2 v-if="aiLoading" class="h-4 w-4 animate-spin" />
          <Sparkles v-else class="h-4 w-4" />
          <span>{{ t('compose.aiSmartWrite') }}</span>
          <ChevronDown v-if="!aiLoading" class="h-3 w-3" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        class="z-50 min-w-32 rounded-md border border-border bg-elevated py-1 shadow-md"
      >
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="runAi('write')"
        >
          {{ t('compose.aiWrite') }}
        </DropdownMenuItem>
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="runAi('polish')"
        >
          {{ t('compose.aiPolish') }}
        </DropdownMenuItem>
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="runAi('optimize-en')"
        >
          {{ t('compose.aiOptimizeEn') }}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuRoot>

    <div :class="dividerClass()" />

    <!-- Font family -->
    <SelectRoot :model-value="currentFontFamily" @update:model-value="setFontFamily">
      <SelectTrigger :class="selectTriggerClass()" class="min-w-[6.5rem]">
        <SelectValue :placeholder="t('compose.fontDefault')">
          <span
            class="truncate"
            :style="currentFontFamily ? { fontFamily: currentFontFamily } : undefined"
            >{{ currentFontFamilyLabel }}</span
          >
        </SelectValue>
        <ChevronDown
          class="h-3.5 w-3.5 shrink-0 text-tertiary transition-transform group-data-[state=open]:rotate-180"
        />
      </SelectTrigger>
      <SelectContent
        position="popper"
        :side-offset="4"
        class="z-50 min-w-[var(--radix-select-trigger-width)] overflow-hidden rounded-md border border-border bg-elevated shadow-md"
      >
        <SelectViewport class="max-h-60 overflow-y-auto py-1">
          <SelectItem
            v-for="option in fontFamilyOptions"
            :key="option.value"
            :value="option.value"
            class="relative flex h-8 cursor-pointer select-none items-center px-3 text-sm text-secondary outline-none transition-colors data-[highlighted]:bg-raised data-[highlighted]:text-primary data-[state=checked]:bg-accent-subtle data-[state=checked]:text-accent"
          >
            <SelectItemText
              class="truncate"
              :style="option.value ? { fontFamily: option.value } : undefined"
            >
              {{ option.label }}
            </SelectItemText>
            <SelectItemIndicator class="ml-auto pl-2">
              <Check class="h-3.5 w-3.5" />
            </SelectItemIndicator>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectRoot>

    <!-- Font size -->
    <SelectRoot :model-value="currentFontSize" @update:model-value="setFontSize">
      <SelectTrigger :class="selectTriggerClass()" class="min-w-[4.5rem]">
        <SelectValue :placeholder="t('compose.fontSizeDefault')">
          <span class="truncate">{{
            currentFontSize ? currentFontSizeLabel : t('compose.fontSizeDefault')
          }}</span>
        </SelectValue>
        <ChevronDown
          class="h-3.5 w-3.5 shrink-0 text-tertiary transition-transform group-data-[state=open]:rotate-180"
        />
      </SelectTrigger>
      <SelectContent
        position="popper"
        :side-offset="4"
        class="z-50 min-w-[var(--radix-select-trigger-width)] overflow-hidden rounded-md border border-border bg-elevated shadow-md"
      >
        <SelectViewport class="max-h-60 overflow-y-auto py-1">
          <SelectItem
            v-for="option in fontSizeOptions"
            :key="option.value"
            :value="option.value"
            class="relative flex h-8 cursor-pointer select-none items-center px-3 text-sm text-secondary outline-none transition-colors data-[highlighted]:bg-raised data-[highlighted]:text-primary data-[state=checked]:bg-accent-subtle data-[state=checked]:text-accent"
          >
            <SelectItemText class="flex items-center gap-1">
              <span>{{ option.label }}</span>
              <span v-if="option.value" class="text-xs text-tertiary">px</span>
            </SelectItemText>
            <SelectItemIndicator class="ml-auto pl-2">
              <Check class="h-3.5 w-3.5" />
            </SelectItemIndicator>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectRoot>

    <div :class="dividerClass()" />

    <!-- Basic formatting -->
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('bold'))"
      :title="t('compose.bold')"
      @click="editor.chain().focus().toggleBold().run()"
    >
      <Bold class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('italic'))"
      :title="t('compose.italic')"
      @click="editor.chain().focus().toggleItalic().run()"
    >
      <Italic class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('underline'))"
      :title="t('compose.underline')"
      @click="editor.chain().focus().toggleUnderline().run()"
    >
      <Underline class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('strike'))"
      :title="t('compose.strike')"
      @click="editor.chain().focus().toggleStrike().run()"
    >
      <Strikethrough class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.clearFormatting')"
      @click="clearFormatting"
    >
      <RemoveFormatting class="h-4 w-4" />
    </button>

    <div :class="dividerClass()" />

    <!-- Text color -->
    <PopoverRoot v-model:open="colorOpen">
      <PopoverTrigger as-child>
        <button
          type="button"
          :class="toolbarBtnClass(!!currentColor)"
          :title="t('compose.textColor')"
        >
          <Palette class="h-4 w-4" :style="{ color: currentColor || undefined }" />
        </button>
      </PopoverTrigger>
      <PopoverContent
        side="bottom"
        :side-offset="4"
        align="start"
        class="z-50 w-48 rounded-lg border border-border bg-elevated p-2 shadow-md"
      >
        <div class="mb-2 text-xs font-medium text-secondary">{{ t('compose.textColor') }}</div>
        <div class="grid grid-cols-7 gap-1">
          <button
            v-for="color in textColors"
            :key="color"
            type="button"
            class="h-5 w-5 rounded-sm border border-border"
            :style="{ backgroundColor: color }"
            :title="color"
            @click="
              setColor(color);
              colorOpen = false;
            "
          />
        </div>
      </PopoverContent>
    </PopoverRoot>

    <!-- Highlight color -->
    <PopoverRoot v-model:open="highlightOpen">
      <PopoverTrigger as-child>
        <button
          type="button"
          :class="toolbarBtnClass(!!currentHighlight)"
          :title="t('compose.highlightColor')"
        >
          <Highlighter class="h-4 w-4" :style="{ color: currentHighlight || undefined }" />
        </button>
      </PopoverTrigger>
      <PopoverContent
        side="bottom"
        :side-offset="4"
        align="start"
        class="z-50 w-48 rounded-lg border border-border bg-elevated p-2 shadow-md"
      >
        <div class="mb-2 text-xs font-medium text-secondary">{{ t('compose.highlightColor') }}</div>
        <div class="grid grid-cols-7 gap-1">
          <button
            v-for="color in highlightColors"
            :key="color"
            type="button"
            class="h-5 w-5 rounded-sm border border-border"
            :style="{ backgroundColor: color }"
            :title="color"
            @click="
              setHighlight(color);
              highlightOpen = false;
            "
          />
        </div>
      </PopoverContent>
    </PopoverRoot>

    <div :class="dividerClass()" />

    <!-- Heading dropdown -->
    <DropdownMenuRoot>
      <DropdownMenuTrigger as-child>
        <button type="button" :class="toolbarBtnClass()" :title="t('compose.heading')">
          <Type class="h-4 w-4" />
          <ChevronDown class="ml-0.5 h-3 w-3" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        class="z-50 min-w-32 rounded-md border border-border bg-elevated py-1 shadow-md"
      >
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="editor.chain().focus().setParagraph().run()"
        >
          <Pilcrow class="mr-2 h-4 w-4" />
          {{ t('compose.paragraph') }}
        </DropdownMenuItem>
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="editor.chain().focus().toggleHeading({ level: 1 }).run()"
        >
          <Heading1 class="mr-2 h-4 w-4" />
          {{ t('compose.heading1') }}
        </DropdownMenuItem>
        <DropdownMenuItem
          class="cursor-pointer px-3 py-1.5 text-sm text-secondary outline-none transition-colors hover:bg-raised hover:text-primary"
          @click="editor.chain().focus().toggleHeading({ level: 2 }).run()"
        >
          <Heading2 class="mr-2 h-4 w-4" />
          {{ t('compose.heading2') }}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenuRoot>

    <div :class="dividerClass()" />

    <!-- Alignment -->
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive({ textAlign: 'left' }))"
      :title="t('compose.alignLeft')"
      @click="editor.chain().focus().setTextAlign('left').run()"
    >
      <AlignLeft class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive({ textAlign: 'center' }))"
      :title="t('compose.alignCenter')"
      @click="editor.chain().focus().setTextAlign('center').run()"
    >
      <AlignCenter class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive({ textAlign: 'right' }))"
      :title="t('compose.alignRight')"
      @click="editor.chain().focus().setTextAlign('right').run()"
    >
      <AlignRight class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive({ textAlign: 'justify' }))"
      :title="t('compose.alignJustify')"
      @click="editor.chain().focus().setTextAlign('justify').run()"
    >
      <AlignJustify class="h-4 w-4" />
    </button>

    <div :class="dividerClass()" />

    <!-- Lists -->
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('bulletList'))"
      :title="t('compose.bulletList')"
      @click="editor.chain().focus().toggleBulletList().run()"
    >
      <List class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('orderedList'))"
      :title="t('compose.orderedList')"
      @click="editor.chain().focus().toggleOrderedList().run()"
    >
      <ListOrdered class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.outdent')"
      @click="editor.chain().focus().outdent().run()"
    >
      <Outdent class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.indent')"
      @click="editor.chain().focus().indent().run()"
    >
      <Indent class="h-4 w-4" />
    </button>

    <div :class="dividerClass()" />

    <!-- Insert -->
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('link'))"
      :title="t('compose.link')"
      @click="setLink"
    >
      <Link class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.image')"
      @click="emit('image')"
    >
      <Image class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.table')"
      @click="insertTable"
    >
      <Table class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('blockquote'))"
      :title="t('compose.quote')"
      @click="editor.chain().focus().toggleBlockquote().run()"
    >
      <Quote class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.horizontalRule')"
      @click="insertHorizontalRule"
    >
      <Minus class="h-4 w-4" />
    </button>

    <div :class="dividerClass()" />

    <!-- Inline formatting -->
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('code'))"
      :title="t('compose.code')"
      @click="editor.chain().focus().toggleCode().run()"
    >
      <Code class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('subscript'))"
      :title="t('compose.subscript')"
      @click="editor.chain().focus().toggleSubscript().run()"
    >
      <Subscript class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass(editor.isActive('superscript'))"
      :title="t('compose.superscript')"
      @click="editor.chain().focus().toggleSuperscript().run()"
    >
      <Superscript class="h-4 w-4" />
    </button>

    <div :class="dividerClass()" />

    <!-- Signature -->
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.signature')"
      @click="emit('signature')"
    >
      <PenLine class="h-4 w-4" />
    </button>

    <!-- Emoji -->
    <PopoverRoot v-model:open="emojiOpen">
      <PopoverTrigger as-child>
        <button type="button" :class="toolbarBtnClass()" :title="t('compose.emoji')">
          <Smile class="h-4 w-4" />
        </button>
      </PopoverTrigger>
      <PopoverContent
        side="bottom"
        :side-offset="4"
        align="end"
        class="z-50 w-56 rounded-lg border border-border bg-elevated p-2 shadow-md"
      >
        <div class="mb-2 text-xs font-medium text-secondary">{{ t('compose.emoji') }}</div>
        <div class="grid grid-cols-8 gap-1">
          <button
            v-for="emoji in emojis"
            :key="emoji"
            type="button"
            class="flex h-7 w-7 items-center justify-center rounded-sm text-lg transition-colors hover:bg-raised"
            @click="runEmoji(emoji)"
          >
            {{ emoji }}
          </button>
        </div>
      </PopoverContent>
    </PopoverRoot>

    <div :class="dividerClass()" />

    <!-- History -->
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.undo')"
      @click="editor.chain().focus().undo().run()"
    >
      <Undo2 class="h-4 w-4" />
    </button>
    <button
      type="button"
      :class="toolbarBtnClass()"
      :title="t('compose.redo')"
      @click="editor.chain().focus().redo().run()"
    >
      <Redo2 class="h-4 w-4" />
    </button>
  </div>

  <LinkDialog
    :open="linkDialogOpen"
    :initial-url="editingLinkUrl"
    :initial-text="editingLinkText"
    @update:open="linkDialogOpen = $event"
    @confirm="onLinkConfirm"
    @remove="onLinkRemove"
  />
</template>
