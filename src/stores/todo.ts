import { ref, computed, onMounted, onUnmounted } from 'vue';
import { defineStore } from 'pinia';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';
import { i18n } from '@/i18n';

export interface TodoItem {
  id: string;
  text: string;
  done: boolean;
  mailId?: string;
  createdAt: number;
  reminderAt?: number;
  completedAt?: number;
  completionLog: number[];
  notifiedAt?: number;
}

let idCounter = 0;

function generateId(): string {
  idCounter += 1;
  return `${Date.now()}-${idCounter}`;
}

export const useTodoStore = defineStore('todo', () => {
  const { call } = useTauriInvoke();
  const items = ref<TodoItem[]>([]);
  const isPanelOpen = ref(false);
  const isLoading = ref(false);

  const pendingItems = computed(() => items.value.filter((i) => !i.done));
  const doneItems = computed(() => items.value.filter((i) => i.done));

  async function loadTodos() {
    isLoading.value = true;
    try {
      items.value = await call<TodoItem[]>('list_todos');
    } finally {
      isLoading.value = false;
    }
  }

  async function addTodo(text: string, mailId?: string, reminderAt?: number) {
    const trimmed = text.trim();
    if (!trimmed) return;
    const todo: TodoItem = {
      id: generateId(),
      text: trimmed,
      done: false,
      mailId,
      createdAt: Date.now(),
      reminderAt,
      completionLog: [],
    };
    await call('upsert_todo', { todo });
    items.value.push(todo);
  }

  async function removeTodo(id: string) {
    await call('delete_todo', { todoId: id });
    items.value = items.value.filter((i) => i.id !== id);
  }

  async function setDone(id: string, done: boolean) {
    const item = items.value.find((i) => i.id === id);
    if (!item) return;

    item.done = done;
    if (done) {
      const now = Date.now();
      item.completedAt = now;
      item.completionLog.push(now);
    } else {
      item.completedAt = undefined;
    }

    await call('upsert_todo', { todo: item });
  }

  function toggleDone(id: string) {
    const item = items.value.find((i) => i.id === id);
    if (item) {
      void setDone(id, !item.done);
    }
  }

  async function updateText(id: string, text: string) {
    const item = items.value.find((i) => i.id === id);
    if (!item) return;
    item.text = text.trim();
    await call('upsert_todo', { todo: item });
  }

  async function setReminder(id: string, reminderAt?: number) {
    const item = items.value.find((i) => i.id === id);
    if (!item) return;
    item.reminderAt = reminderAt;
    item.notifiedAt = undefined;
    await call('upsert_todo', { todo: item });
  }

  async function setFromAiTodos(todos: string[], mailId?: string) {
    const existing = new Set(items.value.map((i) => i.text));
    for (const text of todos) {
      const trimmed = text.trim();
      if (!trimmed || existing.has(trimmed)) continue;
      await addTodo(trimmed, mailId);
      existing.add(trimmed);
    }
  }

  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value;
  }

  function openPanel() {
    isPanelOpen.value = true;
  }

  function closePanel() {
    isPanelOpen.value = false;
  }

  async function clearCompleted() {
    await call('clear_completed_todos');
    items.value = items.value.filter((i) => !i.done);
  }

  async function ensureNotificationPermission() {
    try {
      const granted = await isPermissionGranted();
      if (granted) return true;
      const state = await requestPermission();
      return state === 'granted';
    } catch {
      return false;
    }
  }

  function checkReminders() {
    const now = Date.now();
    const title = i18n.global.t('notification.reminderTitle') as string;
    for (const item of items.value) {
      if (!item.done && item.reminderAt && item.reminderAt <= now && !item.notifiedAt) {
        try {
          sendNotification({
            title,
            body: item.text,
          });
        } catch {
          // Ignore notification errors to avoid breaking the store.
        }
        item.notifiedAt = now;
        void call('upsert_todo', { todo: item });
      }
    }
  }

  let reminderInterval: ReturnType<typeof setInterval> | null = null;

  onMounted(() => {
    void ensureNotificationPermission();
    void loadTodos();
    reminderInterval = setInterval(checkReminders, 30_000);
  });

  onUnmounted(() => {
    if (reminderInterval) {
      clearInterval(reminderInterval);
      reminderInterval = null;
    }
  });

  return {
    items,
    isPanelOpen,
    isLoading,
    pendingItems,
    doneItems,
    loadTodos,
    addTodo,
    removeTodo,
    setDone,
    toggleDone,
    updateText,
    setReminder,
    setFromAiTodos,
    togglePanel,
    openPanel,
    closePanel,
    clearCompleted,
  };
});
