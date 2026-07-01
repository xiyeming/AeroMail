<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Editor } from '@tiptap/core';
import {
  Trash2,
  Minus,
  TableCellsMerge,
  TableCellsSplit,
  Table2,
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  AlignLeft,
  AlignCenter,
  AlignRight,
  PaintBucket,
} from '@lucide/vue';

const props = defineProps<{
  editor: Editor;
}>();

const { t } = useI18n();

const isVisible = ref(false);
const toolbarRef = ref<HTMLDivElement | null>(null);
const position = ref({ top: 0, left: 0 });

const cellBgColor = ref('#ffffff');
const showCellColorPicker = ref(false);
const currentTableAlign = ref<'left' | 'center' | 'right'>('left');

const cellColors = [
  '#ffffff', '#fef3c7', '#fde68a', '#d1fae5', '#a7f3d0',
  '#bfdbfe', '#ddd6fe', '#fecaca', '#fbcfe8', '#e5e7eb',
  '#fef9c3', '#fed7aa', '#fde047', '#bbf7d0', '#99f6e4',
  '#bae6fd', '#c7d2fe', '#fca5a5', '#f9a8d4', '#d4d4d4',
];

function detectTableAlign() {
  const { state } = props.editor;
  const { selection } = state;
  state.doc.nodesBetween(selection.from, selection.to, (node) => {
    if (node.type.name === 'table') {
      currentTableAlign.value = (node.attrs.align as string) || 'left';
      return false;
    }
  });
}

function isClickInsideToolbar(target: EventTarget | null): boolean {
  return !!(toolbarRef.value && target instanceof Node && toolbarRef.value.contains(target));
}

async function showAt(x: number, y: number) {
  detectTableAlign();
  // Render off-screen first so we can measure the real toolbar size, then clamp
  // to the viewport to avoid overflow.
  position.value = { top: -9999, left: -9999 };
  isVisible.value = true;
  await nextTick();
  const rect = toolbarRef.value?.getBoundingClientRect();
  const toolbarWidth = rect?.width ?? 360;
  const toolbarHeight = rect?.height ?? 44;
  const padding = 8;
  const top = Math.min(window.innerHeight - toolbarHeight - padding, y + padding);
  const left = Math.min(window.innerWidth - toolbarWidth - padding, Math.max(padding, x - toolbarWidth / 2));
  position.value = { top, left };
}

function hide() {
  isVisible.value = false;
  showCellColorPicker.value = false;
}

function handleContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  const isTableTarget = target?.closest('.ProseMirror table') !== null;
  if (!isTableTarget) return;

  e.preventDefault();

  // 将光标/选择设置到右键位置，确保后续表格命令作用于正确位置
  const view = props.editor.view;
  const coords = { left: e.clientX, top: e.clientY };
  const posInfo = view.posAtCoords(coords);
  if (posInfo) {
    props.editor.commands.focus();
    props.editor.commands.setTextSelection(posInfo.pos);
  }

  showAt(e.clientX, e.clientY);
}

function handleDocumentClick(e: MouseEvent) {
  if (isClickInsideToolbar(e.target)) return;
  hide();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') hide();
}

function addRowAbove() {
  props.editor.chain().focus().addRowBefore().run();
  hide();
}

function addRowBelow() {
  props.editor.chain().focus().addRowAfter().run();
  hide();
}

function deleteRow() {
  props.editor.chain().focus().deleteRow().run();
  hide();
}

function addColumnLeft() {
  props.editor.chain().focus().addColumnBefore().run();
  hide();
}

function addColumnRight() {
  props.editor.chain().focus().addColumnAfter().run();
  hide();
}

function deleteColumn() {
  props.editor.chain().focus().deleteColumn().run();
  hide();
}

function mergeCells() {
  props.editor.chain().focus().mergeCells().run();
  hide();
}

function splitCell() {
  props.editor.chain().focus().splitCell().run();
  hide();
}

function deleteTable() {
  props.editor.chain().focus().deleteTable().run();
  hide();
}

function toggleHeaderRow() {
  props.editor.chain().focus().toggleHeaderRow().run();
  hide();
}

function mergeCellStyle(existingStyle: string | undefined, newDecl: string): string {
  const styleMap = new Map<string, string>();
  if (existingStyle) {
    existingStyle.split(';').forEach((decl) => {
      const separatorIndex = decl.indexOf(':');
      if (separatorIndex > 0) {
        const prop = decl.slice(0, separatorIndex).trim();
        const value = decl.slice(separatorIndex + 1).trim();
        if (prop) styleMap.set(prop, value);
      }
    });
  }
  const separatorIndex = newDecl.indexOf(':');
  if (separatorIndex > 0) {
    const prop = newDecl.slice(0, separatorIndex).trim();
    const value = newDecl.slice(separatorIndex + 1).trim();
    if (prop) styleMap.set(prop, value);
  }
  return Array.from(styleMap.entries())
    .map(([k, v]) => `${k}: ${v}`)
    .join('; ');
}

function setCellBgColor(color: string) {
  cellBgColor.value = color;
  showCellColorPicker.value = false;
  const existingStyle = props.editor.getAttributes('tableCell').style as string | undefined;
  const merged = mergeCellStyle(existingStyle, `background-color: ${color}`);
  props.editor.chain().focus().setCellAttribute('style', merged).run();
  hide();
}

function setTableAlign(align: 'left' | 'center' | 'right') {
  currentTableAlign.value = align;
  props.editor.chain().focus().setTableAlign(align).run();
  hide();
}

function setCellTextAlign(align: 'left' | 'center' | 'right') {
  const existingStyle = props.editor.getAttributes('tableCell').style as string | undefined;
  const merged = mergeCellStyle(existingStyle, `text-align: ${align}`);
  props.editor.chain().focus().setCellAttribute('style', merged).run();
  hide();
}

let editorDom: HTMLElement | null = null;

onMounted(() => {
  if (props.editor.isMounted) {
    editorDom = props.editor.view.dom;
    editorDom.addEventListener('contextmenu', handleContextMenu);
  }
  document.addEventListener('click', handleDocumentClick);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  if (editorDom) {
    editorDom.removeEventListener('contextmenu', handleContextMenu);
    editorDom = null;
  }
  document.removeEventListener('click', handleDocumentClick);
  document.removeEventListener('keydown', handleKeydown);
});

function btnClass(active = false) {
  return `flex h-7 w-7 items-center justify-center rounded transition-colors hover:bg-raised hover:text-primary disabled:opacity-40 ${active ? 'bg-accent/20 text-accent' : 'text-secondary'}`;
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="isVisible"
        ref="toolbarRef"
        class="fixed z-50 flex items-center gap-0.5 rounded-lg border border-border bg-elevated px-1.5 py-1 shadow-lg"
        :style="{ top: `${position.top}px`, left: `${position.left}px` }"
        @contextmenu.prevent
      >
        <!-- Row operations -->
        <button type="button" :class="btnClass()" :title="t('table.addRowAbove')" @click="addRowAbove">
          <ArrowUp class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.addRowBelow')" @click="addRowBelow">
          <ArrowDown class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.deleteRow')" @click="deleteRow">
          <Minus class="h-3.5 w-3.5" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- Column operations -->
        <button type="button" :class="btnClass()" :title="t('table.addColumnLeft')" @click="addColumnLeft">
          <ArrowLeft class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.addColumnRight')" @click="addColumnRight">
          <ArrowRight class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.deleteColumn')" @click="deleteColumn">
          <Minus class="h-3.5 w-3.5 rotate-90" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- Cell operations -->
        <button type="button" :class="btnClass()" :title="t('table.mergeCells')" @click="mergeCells">
          <TableCellsMerge class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.splitCell')" @click="splitCell">
          <TableCellsSplit class="h-3.5 w-3.5" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- Table header -->
        <button type="button" :class="btnClass()" :title="t('table.toggleHeader')" @click="toggleHeaderRow">
          <Table2 class="h-3.5 w-3.5" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- Delete table -->
        <button type="button" :class="btnClass()" :title="t('table.deleteTable')" @click="deleteTable">
          <Trash2 class="h-3.5 w-3.5 text-danger" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- Cell background color -->
        <div class="relative">
          <button
            type="button"
            :class="btnClass()"
            :title="t('table.cellBgColor')"
            @click="showCellColorPicker = !showCellColorPicker"
          >
            <PaintBucket class="h-3.5 w-3.5" />
          </button>
          <div
            v-if="showCellColorPicker"
            class="absolute top-full left-1/2 z-50 mt-1 -translate-x-1/2 rounded-lg border border-border bg-elevated p-2 shadow-lg"
          >
            <div class="grid grid-cols-5 gap-1">
              <button
                v-for="color in cellColors"
                :key="color"
                type="button"
                class="h-5 w-5 rounded border border-border transition-transform hover:scale-110"
                :style="{ backgroundColor: color }"
                @click="setCellBgColor(color)"
              />
            </div>
          </div>
        </div>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- 表格位置 -->
        <button type="button" :class="btnClass(currentTableAlign === 'left')" :title="t('table.tablePosition') + ': ' + t('table.alignLeft')" @click="setTableAlign('left')">
          <AlignLeft class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass(currentTableAlign === 'center')" :title="t('table.tablePosition') + ': ' + t('table.alignCenter')" @click="setTableAlign('center')">
          <AlignCenter class="h-3.5 w-3.5" />
        </button>
        <button type="button" :class="btnClass(currentTableAlign === 'right')" :title="t('table.tablePosition') + ': ' + t('table.alignRight')" @click="setTableAlign('right')">
          <AlignRight class="h-3.5 w-3.5" />
        </button>

        <div class="mx-0.5 h-4 w-px bg-border" />

        <!-- 单元格文字对齐 -->
        <button type="button" :class="btnClass()" :title="t('table.cellTextAlign') + ': ' + t('table.alignLeft')" @click="setCellTextAlign('left')">
          <AlignLeft class="h-3 w-3" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.cellTextAlign') + ': ' + t('table.alignCenter')" @click="setCellTextAlign('center')">
          <AlignCenter class="h-3 w-3" />
        </button>
        <button type="button" :class="btnClass()" :title="t('table.cellTextAlign') + ': ' + t('table.alignRight')" @click="setCellTextAlign('right')">
          <AlignRight class="h-3 w-3" />
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
