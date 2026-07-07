<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';
import AppLayout from '@/layouts/AppLayout.vue';
import { useLocale } from '@/composables/useLocale';
import { useTheme } from '@/composables/useTheme';
import { useWindowFrame } from '@/composables/useWindowFrame';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useStatusStore } from '@/stores/status';

const { initLocale } = useLocale();
const { initTheme } = useTheme();
const { initDecorations } = useWindowFrame();
const statusStore = useStatusStore();

useKeyboardShortcuts();

onMounted(async () => {
  void initLocale();
  void initTheme();
  void initDecorations();
  await statusStore.initEventListeners();
  void statusStore.loadLastSyncTime();
});
</script>

<template>
  <AppLayout>
    <RouterView />
  </AppLayout>
</template>
