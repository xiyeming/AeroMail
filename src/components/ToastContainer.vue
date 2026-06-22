<script setup lang="ts">
import { CheckCircle, AlertTriangle, XCircle, Info, X } from 'lucide-vue-next';
import { useToastStore, type ToastType } from '@/stores/toast';

const toastStore = useToastStore();

function borderClass(type: ToastType): string {
  return (
    {
      success: 'border-success',
      warning: 'border-warning',
      error: 'border-danger',
      info: 'border-primary',
    } as Record<ToastType, string>
  )[type];
}

function iconComponent(type: ToastType) {
  return (
    {
      success: CheckCircle,
      warning: AlertTriangle,
      error: XCircle,
      info: Info,
    } as Record<ToastType, typeof CheckCircle>
  )[type];
}
</script>

<template>
  <div aria-live="polite" aria-atomic="true" class="fixed right-4 top-4 z-40 flex flex-col gap-2">
    <TransitionGroup
      enter-active-class="transition duration-200 ease-out motion-reduce:transition-none"
      enter-from-class="translate-x-full opacity-0"
      enter-to-class="translate-x-0 opacity-100"
      leave-active-class="transition duration-150 ease-in motion-reduce:transition-none"
      leave-from-class="translate-x-0 opacity-100"
      leave-to-class="-translate-y-full opacity-0"
    >
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="flex min-h-11 min-w-72 max-w-sm items-center gap-3 rounded-lg border-l-4 bg-elevated px-4 py-3 shadow-md"
        :class="borderClass(toast.type)"
      >
        <component :is="iconComponent(toast.type)" class="h-4 w-4 shrink-0 text-secondary" />
        <span class="flex-1 text-sm text-primary">{{ toast.message }}</span>
        <button
          v-if="toast.action"
          type="button"
          class="text-sm font-medium text-accent hover:text-accent-hover"
          @click="
            toast.action.callback();
            toastStore.remove(toast.id);
          "
        >
          {{ toast.action.label }}
        </button>
        <button
          type="button"
          class="flex h-6 w-6 items-center justify-center text-secondary hover:text-primary"
          :aria-label="$t('common.close')"
          @click="toastStore.remove(toast.id)"
        >
          <X class="h-4 w-4" />
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
