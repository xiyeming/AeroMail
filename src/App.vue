<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView } from 'vue-router';
import CommandPalette from '@/components/CommandPalette.vue';
import ToastContainer from '@/components/ToastContainer.vue';
import { useLocale } from '@/composables/useLocale';

const { initLocale } = useLocale();

onMounted(() => {
  initLocale();
});
</script>

<template>
  <RouterView />
  <CommandPalette />
  <ToastContainer />
</template>
