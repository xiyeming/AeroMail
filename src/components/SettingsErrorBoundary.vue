<script setup lang="ts">
import { onErrorCaptured, ref } from 'vue';
import SettingsView from '@/views/SettingsView.vue';

const error = ref<string | null>(null);

onErrorCaptured((err) => {
  error.value = err instanceof Error ? err.message : String(err);
  console.error('[SettingsErrorBoundary] captured error:', err);
  return false;
});
</script>

<template>
  <div v-if="error" class="flex h-full flex-col overflow-y-auto bg-base p-6">
    <h1 class="mb-6 text-2xl font-semibold text-danger">Settings Error</h1>
    <pre class="whitespace-pre-wrap text-sm text-danger">{{ error }}</pre>
  </div>
  <SettingsView v-else />
</template>
