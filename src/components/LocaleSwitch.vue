<script setup lang="ts">
import { computed } from 'vue';
import { useLocale, type Locale } from '@/composables/useLocale';

const { locale, setLocale, supportedLocales } = useLocale();

const labels: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '简体中文',
};

const selected = computed({
  get: () => locale.value as Locale,
  set: (value: Locale) => setLocale(value),
});
</script>

<template>
  <div class="flex items-center gap-3">
    <label class="text-sm text-text-secondary">{{ $t('settings.language') }}</label>
    <select
      v-model="selected"
      class="h-8 rounded-md border border-border bg-card px-2 text-sm text-text outline-none focus:border-primary"
    >
      <option v-for="loc in supportedLocales" :key="loc" :value="loc">
        {{ labels[loc] }}
      </option>
    </select>
  </div>
</template>
