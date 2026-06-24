import { ref, computed } from 'vue';
import { defineStore } from 'pinia';

export interface TodoItem {
  id: string;
  text: string;
  done: boolean;
  mailId?: string;
  createdAt: number;
}

let idCounter = 0;

function generateId(): string {
  idCounter += 1;
  return `${Date.now()}-${idCounter}`;
}

export const useTodoStore = defineStore('todo', () => {
  const items = ref<TodoItem[]>([]);
  const isPanelOpen = ref(false);

  const pendingItems = computed(() => items.value.filter((i) => !i.done));
  const doneItems = computed(() => items.value.filter((i) => i.done));

  function addTodo(text: string, mailId?: string) {
    const trimmed = text.trim();
    if (!trimmed) return;
    items.value.push({
      id: generateId(),
      text: trimmed,
      done: false,
      mailId,
      createdAt: Date.now(),
    });
  }

  function removeTodo(id: string) {
    items.value = items.value.filter((i) => i.id !== id);
  }

  function toggleDone(id: string) {
    const item = items.value.find((i) => i.id === id);
    if (item) {
      item.done = !item.done;
    }
  }

  function updateText(id: string, text: string) {
    const item = items.value.find((i) => i.id === id);
    if (item) {
      item.text = text.trim();
    }
  }

  function setFromAiTodos(todos: string[], mailId?: string) {
    const existing = new Set(items.value.map((i) => i.text));
    for (const text of todos) {
      if (!existing.has(text.trim())) {
        addTodo(text, mailId);
      }
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

  function clearCompleted() {
    items.value = items.value.filter((i) => !i.done);
  }

  return {
    items,
    isPanelOpen,
    pendingItems,
    doneItems,
    addTodo,
    removeTodo,
    toggleDone,
    updateText,
    setFromAiTodos,
    togglePanel,
    openPanel,
    closePanel,
    clearCompleted,
  };
});
