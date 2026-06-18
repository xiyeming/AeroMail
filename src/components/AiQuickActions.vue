<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Sparkles, Reply, ListTodo } from 'lucide-vue-next';

const emit = defineEmits<{
  summarize: [];
  reply: [];
  extractTodos: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.summarize')"
      @click="emit('summarize')"
    >
      <Sparkles class="h-3.5 w-3.5" />
      {{ t('aiActions.summarize') }}
    </button>
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.generateReply')"
      @click="emit('reply')"
    >
      <Reply class="h-3.5 w-3.5" />
      {{ t('aiActions.generateReply') }}
    </button>
    <button
      class="flex h-8 items-center gap-1.5 rounded-md border border-border px-2 text-xs text-text-secondary hover:bg-card"
      :title="t('aiActions.extractTodos')"
      @click="emit('extractTodos')"
    >
      <ListTodo class="h-3.5 w-3.5" />
      {{ t('aiActions.extractTodos') }}
    </button>
  </div>
</template>
