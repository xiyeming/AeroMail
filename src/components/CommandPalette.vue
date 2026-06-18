<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Search } from 'lucide-vue-next';
import { useLocale } from '@/composables/useLocale';

const { t } = useI18n();
const { setLocale } = useLocale();

const isOpen = ref(false);
const query = ref('');
const highlightedIndex = ref(0);

const mockResults = [
  { id: '1', title: 'GitHub Security Alert' },
  { id: '2', title: 'Invoice May 2026' },
  { id: '3', title: 'Meeting Notes' },
];

const languageCommands = computed(() => [
  {
    id: 'switch-lang-en',
    title: t('commandPalette.switchToEnglish'),
    action: () => setLocale('en'),
  },
  {
    id: 'switch-lang-zh',
    title: t('commandPalette.switchToChinese'),
    action: () => setLocale('zh-CN'),
  },
]);

const allItems = computed(() => [...mockResults, ...languageCommands.value]);

const results = ref(allItems.value);

watch(allItems, (val) => {
  if (!query.value) {
    results.value = val;
  } else {
    results.value = val.filter((r) =>
      r.title.toLowerCase().includes(query.value.toLowerCase())
    );
  }
});

watch(query, (val) => {
  if (!val) {
    results.value = allItems.value;
    return;
  }
  results.value = allItems.value.filter((r) =>
    r.title.toLowerCase().includes(val.toLowerCase())
  );
});

function open() {
  isOpen.value = true;
}

function close() {
  isOpen.value = false;
  query.value = '';
  highlightedIndex.value = 0;
}

function highlightPrev() {
  highlightedIndex.value = Math.max(0, highlightedIndex.value - 1);
}

function highlightNext() {
  highlightedIndex.value = Math.min(
    results.value.length - 1,
    highlightedIndex.value + 1
  );
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    if (isOpen.value) {
      close();
    } else {
      open();
    }
  }
  if (!isOpen.value) return;
  if (e.key === 'Escape') close();
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    highlightPrev();
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    highlightNext();
  }
  if (e.key === 'Enter' && results.value[highlightedIndex.value]) {
    const item = results.value[highlightedIndex.value];
    if ('action' in item && typeof item.action === 'function') {
      item.action();
    }
    close();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

defineExpose({ open, close });
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-250 ease-out"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
      >
        <div class="absolute inset-0 bg-overlay" @click="close" />
        <div
          class="relative w-[560px] max-h-[400px] overflow-hidden rounded-xl bg-panel shadow-modal"
        >
          <div class="flex h-14 items-center gap-3 px-4">
            <Search class="h-5 w-5 text-muted" />
            <input
              v-model="query"
              type="text"
              class="flex-1 bg-transparent text-base text-text placeholder-muted outline-none"
              :placeholder="t('commandPalette.placeholder')"
            />
          </div>
          <div class="max-h-[340px] overflow-y-auto">
            <div
              v-for="(item, index) in results"
              :key="item.id"
              class="flex h-12 cursor-pointer items-center px-4 text-sm"
              :class="[
                index === highlightedIndex
                  ? 'border-l-[3px] border-primary bg-card'
                  : 'border-l-[3px] border-transparent',
              ]"
              @mouseenter="highlightedIndex = index"
              @click="() => { if ('action' in item && typeof item.action === 'function') item.action(); close(); }"
            >
              {{ item.title }}
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
