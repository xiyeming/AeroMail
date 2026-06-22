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
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-accent-subtle"
      >
        <Bot class="h-3.5 w-3.5 text-accent" />
      </div>
      <div
        :class="[
          'max-w-[80%] rounded-lg px-3 py-2 text-sm',
          msg.role === 'user' ? 'bg-accent text-white' : 'bg-raised text-primary',
        ]"
      >
        <pre class="whitespace-pre-wrap font-sans">{{ msg.content }}</pre>
      </div>
      <div
        v-if="msg.role === 'user'"
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-raised"
      >
        <User class="h-3.5 w-3.5 text-secondary" />
      </div>
    </div>
  </div>
</template>
