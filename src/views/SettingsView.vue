<script setup lang="ts">
import LocaleSwitch from '@/components/LocaleSwitch.vue';
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-panel p-6">
    <h1 class="text-h1 mb-6 font-semibold text-text">{{ $t('nav.settings') }}</h1>
    <section class="rounded-lg border border-border bg-card p-4">
      <h2 class="mb-4 text-lg font-medium text-text">{{ $t('settings.language') }}</h2>
      <LocaleSwitch />
    </section>
  </div>
</template>
