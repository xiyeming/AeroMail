import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { SearchQuery, SearchResult, SearchStats } from '@/types/search';

const SEARCH_HISTORY_KEY = 'aeromail_search_history';
const MAX_HISTORY_ITEMS = 10;

export function useSearch() {
  const { call } = useTauriInvoke();
  const results = ref<SearchResult[]>([]);
  const isSearching = ref(false);
  const error = ref<string | null>(null);
  const stats = ref<SearchStats | null>(null);
  const searchHistory = ref<string[]>(loadHistory());

  // 加载搜索历史
  function loadHistory(): string[] {
    try {
      const stored = localStorage.getItem(SEARCH_HISTORY_KEY);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  // 保存搜索历史
  function saveHistory() {
    try {
      localStorage.setItem(SEARCH_HISTORY_KEY, JSON.stringify(searchHistory.value));
    } catch {
      // 忽略存储错误
    }
  }

  // 添加到搜索历史
  function addToHistory(query: string) {
    if (!query.trim()) return;

    // 移除重复项
    const index = searchHistory.value.indexOf(query);
    if (index !== -1) {
      searchHistory.value.splice(index, 1);
    }

    // 添加到开头
    searchHistory.value.unshift(query);

    // 限制历史记录数量
    if (searchHistory.value.length > MAX_HISTORY_ITEMS) {
      searchHistory.value = searchHistory.value.slice(0, MAX_HISTORY_ITEMS);
    }

    saveHistory();
  }

  // 清除搜索历史
  function clearHistory() {
    searchHistory.value = [];
    saveHistory();
  }

  // 从历史中删除
  function removeFromHistory(query: string) {
    const index = searchHistory.value.indexOf(query);
    if (index !== -1) {
      searchHistory.value.splice(index, 1);
      saveHistory();
    }
  }

  async function search(query: SearchQuery) {
    isSearching.value = true;
    error.value = null;
    try {
      results.value = await call<SearchResult[]>('search_mails', { query });
      // 添加到搜索历史
      addToHistory(query.query);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isSearching.value = false;
    }
  }

  async function indexPendingMails() {
    try {
      const count = await call<number>('index_pending_mails');
      return count;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      return 0;
    }
  }

  async function loadStats() {
    try {
      stats.value = await call<SearchStats>('get_search_stats');
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  function clearResults() {
    results.value = [];
  }

  return {
    results,
    isSearching,
    error,
    stats,
    searchHistory,
    search,
    indexPendingMails,
    loadStats,
    clearResults,
    clearHistory,
    removeFromHistory,
  };
}
