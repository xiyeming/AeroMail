<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { Search } from '@lucide/vue';
import { useLocale } from '@/composables/useLocale';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import { useSearch } from '@/composables/useSearch';

const { t } = useI18n();
const { setLocale } = useLocale();
const aiStore = useAiStore();
const mailStore = useMailStore();
const { search, results: searchResults, isSearching, searchHistory } = useSearch();

const isOpen = ref(false);
const query = ref('');
const highlightedIndex = ref(0);

interface TextSegment {
  text: string;
  highlight: boolean;
}

function highlightSegments(text: string, searchTerm: string): TextSegment[] {
  if (!searchTerm || !text) return [{ text, highlight: false }];

  const regex = new RegExp(`(${searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
  const segments: TextSegment[] = [];
  let lastIndex = 0;

  for (const match of text.matchAll(regex)) {
    const index = match.index ?? 0;
    if (index > lastIndex) {
      segments.push({ text: text.slice(lastIndex, index), highlight: false });
    }
    segments.push({ text: match[0], highlight: true });
    lastIndex = index + match[0].length;
  }

  if (lastIndex < text.length) {
    segments.push({ text: text.slice(lastIndex), highlight: false });
  }

  return segments;
}

const languageCommands = computed(() => [
  {
    id: 'switch-lang-en',
    title: t('commandPalette.switchToEnglish'),
    type: 'command' as const,
    action: () => setLocale('en'),
  },
  {
    id: 'switch-lang-zh',
    title: t('commandPalette.switchToChinese'),
    type: 'command' as const,
    action: () => setLocale('zh-CN'),
  },
]);

const aiCommands = computed(() => [
  {
    id: 'open-ai-assistant',
    title: t('commandPalette.openAiAssistant'),
    type: 'command' as const,
    action: () => aiStore.togglePanel(),
  },
]);

const mailResults = computed(() =>
  searchResults.value.map((r) => ({
    id: r.mailId,
    title: r.snippet || r.mailId,
    type: 'mail' as const,
    score: r.score,
    action: () => mailStore.selectMail(r.mailId),
  }))
);

const historyItems = computed(() =>
  searchHistory.value.map((h) => ({
    id: `history-${h}`,
    title: h,
    type: 'history' as const,
    action: () => {
      query.value = h;
      search({ query: h });
    },
  }))
);

const allItems = computed(() => [
  ...historyItems.value,
  ...mailResults.value,
  ...languageCommands.value,
  ...aiCommands.value,
]);

const results = ref(allItems.value);

let searchTimeout: ReturnType<typeof setTimeout> | null = null;

watch(query, (val) => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }

  if (!val) {
    results.value = allItems.value;
    return;
  }

  searchTimeout = setTimeout(() => {
    search({ query: val });
  }, 300);
});

watch(allItems, (val) => {
  if (!query.value) {
    results.value = val;
  }
});

watch(searchResults, (val) => {
  if (query.value) {
    results.value = val;
  }
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
  highlightedIndex.value = Math.min(results.value.length - 1, highlightedIndex.value + 1);
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
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
});

defineExpose({ open, close });
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-250 ease-out motion-reduce:transition-none"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition duration-150 ease-in motion-reduce:transition-none"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-start justify-center pt-24"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
      >
        <div class="absolute inset-0 bg-overlay" @click="close" />
        <div
          class="relative w-full max-w-2xl max-h-96 overflow-hidden rounded-xl border border-border bg-elevated shadow-lg"
        >
          <div class="flex h-14 items-center gap-3 px-4">
            <Search class="h-5 w-5 text-secondary" />
            <input
              v-model="query"
              type="text"
              class="flex-1 bg-transparent text-base text-primary placeholder:text-tertiary outline-none"
              :placeholder="t('commandPalette.placeholder')"
            />
            <div
              v-if="isSearching"
              class="h-4 w-4 animate-spin rounded-full border-2 border-accent border-t-transparent"
            />
          </div>
          <div class="max-h-80 overflow-y-auto">
            <div
              v-for="(item, index) in results"
              :key="item.id"
              class="flex h-12 cursor-pointer items-center px-4 text-sm"
              :class="[
                index === highlightedIndex
                  ? 'border-l-4 border-accent bg-base'
                  : 'border-l-4 border-transparent',
              ]"
              @mouseenter="highlightedIndex = index"
              @click="
                () => {
                  if ('action' in item && typeof item.action === 'function') item.action();
                  close();
                }
              "
            >
              <span class="truncate">
                <span
                  v-for="(segment, sIdx) in highlightSegments(item.title, query)"
                  :key="sIdx"
                  :class="{
                    'rounded bg-accent-subtle px-0.5': segment.highlight,
                  }"
                >
                  {{ segment.text }}
                </span>
              </span>
              <span v-if="item.type === 'mail'" class="ml-auto shrink-0 text-xs text-secondary">
                {{ t('commandPalette.mail') }}
              </span>
              <span v-if="item.type === 'history'" class="ml-auto shrink-0 text-xs text-secondary">
                {{ t('commandPalette.history') }}
              </span>
            </div>
            <div
              v-if="results.length === 0 && query"
              class="flex h-12 items-center px-4 text-sm text-secondary"
            >
              {{ t('commandPalette.noResults') }}
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
