<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';
import AppLayout from '@/layouts/AppLayout.vue';
import CommandPalette from '@/components/CommandPalette.vue';
import ToastContainer from '@/components/ToastContainer.vue';
import { useLocale } from '@/composables/useLocale';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useStatusStore } from '@/stores/status';

const { initLocale } = useLocale();
const statusStore = useStatusStore();

// Initialize keyboard shortcuts
useKeyboardShortcuts();

onMounted(() => {
  initLocale();
  statusStore.initEventListeners();
});
</script>

<template>
  <AppLayout>
    <RouterView />
  </AppLayout>
  <CommandPalette />
  <ToastContainer />
</template>
