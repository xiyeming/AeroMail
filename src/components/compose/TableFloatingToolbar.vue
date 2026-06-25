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

onMounted(() => {
  props.editor.on('selectionUpdate', checkTableSelection);
  props.editor.on('transaction', checkTableSelection);
});

onUnmounted(() => {
  props.editor.off('selectionUpdate', checkTableSelection);
  props.editor.off('transaction', checkTableSelection);
});

function btnClass() {
  return 'flex h-7 w-7 items-center justify-center rounded text-secondary transition-colors hover:bg-raised hover:text-primary disabled:opacity-40';
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
