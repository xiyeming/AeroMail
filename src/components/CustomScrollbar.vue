<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';

const props = defineProps<{
  scrollTop?: number;
}>();

const emit = defineEmits<{
  'update:scrollTop': [value: number];
}>();

const contentRef = ref<HTMLDivElement | null>(null);

const contentScrollTop = ref(0);
const scrollHeight = ref(0);
const clientHeight = ref(0);

const showScrollbar = computed(() => scrollHeight.value > clientHeight.value);

const thumbHeight = computed(() => {
  if (!showScrollbar.value) return 0;
  const ratio = clientHeight.value / scrollHeight.value;
  return Math.max(32, ratio * clientHeight.value);
});

const thumbTop = computed(() => {
  if (!showScrollbar.value) return 0;
  const maxScroll = scrollHeight.value - clientHeight.value;
  const maxThumbTop = clientHeight.value - thumbHeight.value;
  if (maxThumbTop <= 0) return 0;
  return (contentScrollTop.value / maxScroll) * maxThumbTop;
});

function updateMetrics() {
  const el = contentRef.value;
  if (!el) return;
  contentScrollTop.value = el.scrollTop;
  scrollHeight.value = el.scrollHeight;
  clientHeight.value = el.clientHeight;
}

function handleScroll() {
  updateMetrics();
  emit('update:scrollTop', contentScrollTop.value);
}

let isDragging = false;
let dragStartY = 0;
let dragStartScrollTop = 0;

function handleThumbMouseDown(e: MouseEvent) {
  e.preventDefault();
  e.stopPropagation();
  isDragging = true;
  dragStartY = e.clientY;
  dragStartScrollTop = contentRef.value?.scrollTop ?? 0;
  document.addEventListener('mousemove', handleThumbMouseMove);
  document.addEventListener('mouseup', handleThumbMouseUp);
}

function handleThumbMouseMove(e: MouseEvent) {
  if (!isDragging || !contentRef.value) return;
  const deltaY = e.clientY - dragStartY;
  const maxScroll = Math.max(0, scrollHeight.value - clientHeight.value);
  const maxThumbTop = Math.max(0, clientHeight.value - thumbHeight.value);
  if (maxThumbTop <= 0) return;
  const scrollDelta = (deltaY / maxThumbTop) * maxScroll;
  contentRef.value.scrollTop = Math.max(0, Math.min(maxScroll, dragStartScrollTop + scrollDelta));
}

function handleThumbMouseUp() {
  isDragging = false;
  document.removeEventListener('mousemove', handleThumbMouseMove);
  document.removeEventListener('mouseup', handleThumbMouseUp);
}

function handleTrackClick(e: MouseEvent) {
  const track = e.currentTarget as HTMLElement;
  const rect = track.getBoundingClientRect();
  const clickY = e.clientY - rect.top;
  const el = contentRef.value;
  if (!el) return;

  if (clickY < thumbTop.value) {
    el.scrollTop -= clientHeight.value;
  } else if (clickY > thumbTop.value + thumbHeight.value) {
    el.scrollTop += clientHeight.value;
  }
}

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateMetrics();
  void nextTick(() => updateMetrics());
  resizeObserver = new ResizeObserver(() => updateMetrics());
  if (contentRef.value) {
    resizeObserver.observe(contentRef.value);
  }
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  document.removeEventListener('mousemove', handleThumbMouseMove);
  document.removeEventListener('mouseup', handleThumbMouseUp);
});

watch(
  () => props.scrollTop,
  (value) => {
    if (value === undefined || !contentRef.value || isDragging) return;
    contentRef.value.scrollTop = value;
  }
);

defineExpose({
  scrollTo: (options) => contentRef.value?.scrollTo(options),
  scrollToTop: () => contentRef.value?.scrollTo({ top: 0 }),
  getScrollElement: () => contentRef.value,
});
</script>

<template>
  <div class="custom-scrollbar-viewport relative h-full w-full overflow-hidden">
    <div
      ref="contentRef"
      class="custom-scrollbar-content absolute inset-0 overflow-y-auto"
      @scroll="handleScroll"
    >
      <slot />
    </div>

    <div
      v-if="showScrollbar"
      class="custom-scrollbar-track pointer-events-auto absolute right-1 top-1 bottom-1 z-10 w-1.5 rounded-full bg-transparent"
      @mousedown="handleTrackClick"
    >
      <div
        class="custom-scrollbar-thumb absolute right-0 w-1.5 rounded-full bg-border-strong transition-colors hover:bg-text-tertiary"
        :style="{ top: `${thumbTop}px`, height: `${thumbHeight}px` }"
        @mousedown="handleThumbMouseDown"
      />
    </div>
  </div>
</template>

<style>
.custom-scrollbar-content::-webkit-scrollbar {
  display: none;
}
.custom-scrollbar-content {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
</style>
