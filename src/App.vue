<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';
import AppLayout from '@/layouts/AppLayout.vue';
import { useLocale } from '@/composables/useLocale';
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts';
import { useStatusStore } from '@/stores/status';

const { initLocale } = useLocale();
const statusStore = useStatusStore();

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
</template>
