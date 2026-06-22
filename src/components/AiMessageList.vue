<script setup lang="ts">
import type { AiChatMessage } from '@/types/ai';
import { Bot, User } from 'lucide-vue-next';

defineProps<{
  messages: AiChatMessage[];
}>();
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      v-for="msg in messages"
      :key="msg.id"
      :class="['flex gap-2', msg.role === 'user' ? 'justify-end' : 'justify-start']"
    >
      <div
        v-if="msg.role === 'assistant'"
        class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-primary/20"
      >
        <Bot class="h-3.5 w-3.5 text-primary" />
      </div>
      <div
        :class="[
          'max-w-[80%] rounded-lg px-3 py-2 text-sm',
          msg.role === 'user' ? 'bg-primary text-white' : 'bg-card text-text',
        ]"
      >
        <pre class="whitespace-pre-wrap font-sans">{{ msg.content }}</pre>
      </div>
      <div
        v-if="msg.role === 'user'"
        class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-muted/20"
      >
        <User class="h-3.5 w-3.5 text-muted" />
      </div>
    </div>
  </div>
</template>
