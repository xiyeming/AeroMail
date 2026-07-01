<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Sparkles, ListTodo, Loader2 } from '@lucide/vue';

withDefaults(
  defineProps<{
    isSummarizing?: boolean;
    isExtractingTodos?: boolean;
  }>(),
  {
    isSummarizing: false,
    isExtractingTodos: false,
  }
);

const emit = defineEmits<{
  summarize: [];
  extractTodos: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      type="button"
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-secondary transition-colors hover:bg-raised disabled:opacity-50"
      :title="t('aiActions.summarize')"
      :disabled="isSummarizing"
      @click="emit('summarize')"
    >
      <Loader2 v-if="isSummarizing" class="h-3.5 w-3.5 animate-spin" />
      <Sparkles v-else class="h-3.5 w-3.5" />
      {{ isSummarizing ? t('aiActions.summarizing') : t('aiActions.summarize') }}
    </button>
    <button
      type="button"
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-secondary transition-colors hover:bg-raised disabled:opacity-50"
      :title="t('aiActions.extractTodos')"
      :disabled="isExtractingTodos"
      @click="emit('extractTodos')"
    >
      <Loader2 v-if="isExtractingTodos" class="h-3.5 w-3.5 animate-spin" />
      <ListTodo v-else class="h-3.5 w-3.5" />
      {{ isExtractingTodos ? t('aiActions.extractingTodos') : t('aiActions.extractTodos') }}
    </button>
  </div>
</template>
