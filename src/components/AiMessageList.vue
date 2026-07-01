<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AiChatMessage } from '@/types/ai';
import { Bot, ChevronDown, ChevronUp, Mail, User } from '@lucide/vue';
import MarkdownText from './MarkdownText.vue';

const { t } = useI18n();

defineProps<{
  messages: AiChatMessage[];
  isLoading?: boolean;
}>();

const expandedThinking = ref<Set<string>>(new Set());
const expandedQuotes = ref<Set<string>>(new Set());

interface QuoteBlock {
  label: string;
  subject: string;
  from: string;
  body: string;
}

function toggleThinking(id: string) {
  const next = new Set(expandedThinking.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  expandedThinking.value = next;
}

function toggleQuote(key: string) {
  const next = new Set(expandedQuotes.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  expandedQuotes.value = next;
}

function parseMessage(content: string): { text: string; quotes: QuoteBlock[] } {
  const parts = content.split('\n\n---\n\n');
  const text = parts[0] ?? '';
  const quotes: QuoteBlock[] = [];
  for (let i = 1; i < parts.length; i += 1) {
    const match = parts[i]
      .trim()
      .match(/^\[(.+?)\]\n(.+?):\s*(.+?)\n(.+?):\s*(.+?)\n\n([\s\S]*)$/m);
    if (match) {
      quotes.push({
        label: match[1],
        subject: match[3],
        from: match[5],
        body: match[6],
      });
    }
  }
  return { text, quotes };
}

function quoteKey(msgId: string, index: number): string {
  return `${msgId}-${index}`;
}
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
          'max-w-xl rounded-lg px-3 py-2 text-sm',
          msg.role === 'user' ? 'bg-accent text-white' : 'bg-raised text-primary',
        ]"
      >
        <template v-if="msg.role === 'user'">
          <div v-if="parseMessage(msg.content).quotes.length > 0" class="mb-2 flex flex-col gap-2">
            <div
              v-for="(quote, idx) in parseMessage(msg.content).quotes"
              :key="idx"
              class="rounded-md border border-white/20 bg-elevated p-2 text-left text-primary shadow-sm"
            >
              <div class="flex items-center gap-1 text-[11px] text-accent">
                <Mail class="h-3 w-3" />
                <span>{{ quote.label }}</span>
              </div>
              <div class="mt-1 text-xs font-medium">{{ quote.subject || t('mail.noSubject') }}</div>
              <div class="text-[11px] text-secondary">
                {{ quote.from || t('mail.unknownSender') }}
              </div>
              <div
                v-if="expandedQuotes.has(quoteKey(msg.id, idx))"
                class="mt-1 max-h-32 overflow-y-auto text-[11px] text-secondary whitespace-pre-wrap"
              >
                {{ quote.body }}
              </div>
              <button
                type="button"
                class="mt-1 text-[11px] text-secondary transition-colors hover:text-primary"
                @click="toggleQuote(quoteKey(msg.id, idx))"
              >
                <span v-if="expandedQuotes.has(quoteKey(msg.id, idx))">{{
                  t('mail.collapseQuote')
                }}</span>
                <span v-else>{{ t('mail.expandQuote') }}</span>
              </button>
            </div>
          </div>
          <MarkdownText
            v-if="parseMessage(msg.content).text.trim()"
            :content="parseMessage(msg.content).text"
            class="text-current"
          />
        </template>
        <template v-else>
          <MarkdownText :content="msg.content" class="text-current" />

          <template v-if="msg.thinking">
            <button
              type="button"
              class="mt-2 flex items-center gap-1 text-xs text-secondary transition-colors hover:text-primary"
              @click="toggleThinking(msg.id)"
            >
              <ChevronUp v-if="expandedThinking.has(msg.id)" class="h-3.5 w-3.5" />
              <ChevronDown v-else class="h-3.5 w-3.5" />
              <span>
                {{
                  expandedThinking.has(msg.id)
                    ? $t('aiAssistant.collapseThinking')
                    : $t('aiAssistant.expandThinking')
                }}
              </span>
            </button>
            <div
              v-if="expandedThinking.has(msg.id)"
              class="mt-2 rounded-md bg-base px-2.5 py-2 text-xs text-secondary"
            >
              <div class="mb-1 font-medium text-tertiary">
                {{ $t('aiAssistant.thinkingProcess') }}
              </div>
              <pre class="whitespace-pre-wrap font-sans">{{ msg.thinking }}</pre>
            </div>
          </template>
        </template>
      </div>
      <div
        v-if="msg.role === 'user'"
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-raised"
      >
        <User class="h-3.5 w-3.5 text-secondary" />
      </div>
    </div>

    <!-- Replying indicator -->
    <div v-if="isLoading" class="flex justify-start gap-2">
      <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-accent-subtle">
        <Bot class="h-3.5 w-3.5 text-accent" />
      </div>
      <div
        class="flex max-w-xl items-center gap-1.5 rounded-lg bg-raised px-3 py-2 text-sm text-secondary"
        :aria-label="$t('aiAssistant.replying')"
      >
        <span
          class="h-1.5 w-1.5 rounded-full bg-secondary animate-bounce"
          style="animation-delay: -0.3s"
        />
        <span
          class="h-1.5 w-1.5 rounded-full bg-secondary animate-bounce"
          style="animation-delay: -0.15s"
        />
        <span class="h-1.5 w-1.5 rounded-full bg-secondary animate-bounce" />
      </div>
    </div>
  </div>
</template>
