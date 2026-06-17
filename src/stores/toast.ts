import { defineStore } from 'pinia';
import { ref } from 'vue';

export type ToastType = 'success' | 'warning' | 'error' | 'info';

export interface ToastItem {
  id: string;
  type: ToastType;
  message: string;
  action?: { label: string; callback: () => void };
  duration: number;
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<ToastItem[]>([]);

  function add(toast: Omit<ToastItem, 'id'>) {
    const id = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
    const item: ToastItem = { ...toast, id };
    toasts.value.push(item);
    if (toasts.value.length > 3) {
      toasts.value.shift();
    }
    setTimeout(() => remove(id), toast.duration);
  }

  function remove(id: string) {
    const idx = toasts.value.findIndex((t) => t.id === id);
    if (idx >= 0) {
      toasts.value.splice(idx, 1);
    }
  }

  return {
    toasts,
    add,
    remove,
  };
});
