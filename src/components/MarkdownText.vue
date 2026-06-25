<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  content: string;
}>();

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function sanitizeHref(url: string): string {
  if (/^(https?|mailto):/i.test(url)) {
    return url;
  }
  return '#';
}

const rendered = computed(() => {
  let html = escapeHtml(props.content);

  // Fenced code blocks
  html = html.replace(
    /```([\s\S]*?)```/g,
    (_, code: string) =>
      `<pre class="rounded-md bg-base px-2.5 py-2 text-xs overflow-x-auto"><code>${code.trim()}</code></pre>`
  );

  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code class="rounded bg-base px-1 py-0.5 text-xs">$1</code>');

  // Bold
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  // Italic (avoid already processed bold)
  html = html.replace(/(?<!\*)\*(?!\*)([^*]+)(?<!\*)\*(?!\*)/g, '<em>$1</em>');

  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text: string, url: string) => {
    const safe = sanitizeHref(url);
    return `<a href="${safe}" class="underline hover:opacity-80" target="_blank" rel="noopener noreferrer">${text}</a>`;
  });

  // Blockquotes
  html = html.replace(
    /^&gt;\s*(.+)$/gm,
    '<blockquote class="border-l-2 border-border pl-2 text-secondary">$1</blockquote>'
  );

  // Process lists and paragraphs line by line
  const lines = html.split('\n');
  const out: string[] = [];
  let inList: 'ul' | 'ol' | null = null;

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    const ulMatch = line.match(/^(\s*)[-*]\s+(.+)$/);
    const olMatch = line.match(/^(\s*)\d+\.\s+(.+)$/);

    if (ulMatch) {
      if (inList !== 'ul') {
        if (inList) out.push(inList === 'ul' ? '</ul>' : '</ol>');
        out.push('<ul class="list-disc pl-4 space-y-0.5">');
        inList = 'ul';
      }
      out.push(`<li>${ulMatch[2]}</li>`);
    } else if (olMatch) {
      if (inList !== 'ol') {
        if (inList) out.push(inList === 'ul' ? '</ul>' : '</ol>');
        out.push('<ol class="list-decimal pl-4 space-y-0.5">');
        inList = 'ol';
      }
      out.push(`<li>${olMatch[2]}</li>`);
    } else {
      if (inList) {
        out.push(inList === 'ul' ? '</ul>' : '</ol>');
        inList = null;
      }
      if (line.trim() === '') {
        out.push('');
      } else {
        out.push(`<p class="min-h-[1em]">${line}</p>`);
      }
    }
  }
  if (inList) {
    out.push(inList === 'ul' ? '</ul>' : '</ol>');
  }

  // Collapse multiple blank paragraph lines into a single break
  const joined = out
    .join('\n')
    .replace(
      /<p class="min-h-\[1em\]"><\/p>(\n<p class="min-h-\[1em\]"><\/p>)+/g,
      '<p class="min-h-[1em]"></p>'
    );

  return joined;
});
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -->
  <div class="markdown-text leading-relaxed" v-html="rendered" />
</template>

<style scoped>
.markdown-text :deep(p) {
  margin: 0.35em 0;
}
.markdown-text :deep(p:first-child) {
  margin-top: 0;
}
.markdown-text :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-text :deep(ul),
.markdown-text :deep(ol) {
  margin: 0.35em 0;
}
.markdown-text :deep(blockquote) {
  margin: 0.35em 0;
}
</style>
