<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
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
} from 'lucide-vue-next';

const props = defineProps<{
  editor: Editor;
}>();

const { t } = useI18n();

const isVisible = ref(false);
const toolbarRef = ref<HTMLDivElement | null>(null);
const position = ref({ top: 0, left: 0 });

function checkTableSelection() {
  if (!props.editor) return;
  const { state } = props.editor;
  const { selection } = state;

  // Check if the selection is inside a table node
  const isTableNode = (node: { type: { name: string } }) => node.type.name === 'table';

  let isInTable = false;
  state.doc.nodesBetween(selection.from, selection.to, (node) => {
    if (isTableNode(node)) {
      isInTable = true;
    }
  });

  if (isInTable) {
    // Get cursor position — coordsAtPos returns viewport-relative coords
    const { from } = selection;
    const view = props.editor.view;
    const coords = view.coordsAtPos(from);

    // Position toolbar above the cursor, clamped to viewport
    const toolbarHeight = 40;
    const toolbarWidth = 280;
    const top = Math.max(8, coords.top - toolbarHeight - 8);
    const left = Math.max(8, Math.min(coords.left - toolbarWidth / 2, window.innerWidth - toolbarWidth - 8));

    position.value = { top, left };
    isVisible.value = true;
  } else {
    isVisible.value = false;
  }
}

function addRowAbove() {
  props.editor.chain().focus().addRowBefore().run();
}

function addRowBelow() {
  props.editor.chain().focus().addRowAfter().run();
}

function deleteRow() {
  props.editor.chain().focus().deleteRow().run();
}

function addColumnLeft() {
  props.editor.chain().focus().addColumnBefore().run();
}

function addColumnRight() {
  props.editor.chain().focus().addColumnAfter().run();
}

function deleteColumn() {
  props.editor.chain().focus().deleteColumn().run();
}

function mergeCells() {
  props.editor.chain().focus().mergeCells().run();
}

function splitCell() {
  props.editor.chain().focus().splitCell().run();
}

function deleteTable() {
  props.editor.chain().focus().deleteTable().run();
}

function toggleHeaderRow() {
  props.editor.chain().focus().toggleHeaderRow().run();
}

const cellBgColor = ref('#ffffff');
const showCellColorPicker = ref(false);

const cellColors = [
  '#ffffff', '#fef3c7', '#fde68a', '#d1fae5', '#a7f3d0',
  '#bfdbfe', '#ddd6fe', '#fecaca', '#fbcfe8', '#e5e7eb',
  '#fef9c3', '#fed7aa', '#fde047', '#bbf7d0', '#99f6e4',
  '#bae6fd', '#c7d2fe', '#fca5a5', '#f9a8d4', '#d4d4d4',
];

function setCellBgColor(color: string) {
  cellBgColor.value = color;
  showCellColorPicker.value = false;
  // Apply background to the currently selected cells
  const { state } = props.editor;
  const { selection } = state;
  const cellSelection = selection;
  // Use setCellAttribute to set background-color
  props.editor.chain().focus().setCellAttribute('style', `background-color: ${color}`).run();
}

const currentTableAlign = ref<'left' | 'center' | 'right'>('left');

function detectTableAlign() {
  const { state } = props.editor;
  const { selection } = state;
  let foundTable = false;
  state.doc.nodesBetween(selection.from, selection.to, (node) => {
    if (node.type.name === 'table') foundTable = true;
  });
  if (!foundTable) return;
  // 检查 tableWrapper 的 margin 来判断当前对齐
  const tableEl = props.editor.view.domAtPos(selection.from);
  let el = tableEl.node as HTMLElement;
  if (el.nodeType === 3) el = el.parentElement!;
  const wrapper = el.closest?.('.tableWrapper') || el.closest?.('table')?.parentElement;
  if (wrapper) {
    const style = wrapper.getAttribute('style') || '';
    if (style.includes('margin-left:auto') && style.includes('margin-right:auto')) {
      currentTableAlign.value = 'center';
    } else if (style.includes('margin-left:auto')) {
      currentTableAlign.value = 'right';
    } else {
      currentTableAlign.value = 'left';
    }
  }
}

function setTableAlign(align: 'left' | 'center' | 'right') {
  currentTableAlign.value = align;
  const { state } = props.editor;
  const { from } = state.selection;
  // 找到 tableWrapper DOM 元素并修改其 margin
  try {
    const resolved = props.editor.view.domAtPos(from);
    let el = resolved.node as HTMLElement;
    if (el.nodeType === 3) el = el.parentElement!;
    const wrapper = el.closest?.('.tableWrapper') || el.closest?.('table')?.parentElement;
    if (wrapper) {
      let marginStyle = '';
      switch (align) {
        case 'left':
          marginStyle = 'margin:0.5em 0;margin-right:auto;';
          break;
        case 'center':
          marginStyle = 'margin:0.5em auto;';
          break;
        case 'right':
          marginStyle = 'margin:0.5em 0;margin-left:auto;';
          break;
      }
      wrapper.setAttribute('style', marginStyle);
    }
  } catch {
    // fallback: 忽略
  }
}

function setCellTextAlign(align: 'left' | 'center' | 'right') {
  props.editor.chain().focus().setCellAttribute('style', `text-align: ${align}`).run();
}

onMounted(() => {
  props.editor.on('selectionUpdate', () => { checkTableSelection(); detectTableAlign(); });
  props.editor.on('transaction', checkTableSelection);
  detectTableAlign();
});

onUnmounted(() => {
  props.editor.off('selectionUpdate', checkTableSelection);
  props.editor.off('transaction', checkTableSelection);
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

        <!-- Cell text alignment -->
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
