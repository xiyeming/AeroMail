<script setup lang="ts">
const mockMails = [
  {
    id: '1',
    fromName: 'GitHub',
    subject: 'Security Alert',
    snippet: 'New login detected from...',
    time: '14:22',
    isRead: false,
    isStarred: false,
    hasAttachments: false,
  },
  {
    id: '2',
    fromName: 'Billing',
    subject: 'Invoice May 2026',
    snippet: 'Your invoice for May 2026 is ready...',
    time: '12:10',
    isRead: true,
    isStarred: true,
    hasAttachments: true,
  },
];
</script>

<template>
  <div class="flex h-full flex-col bg-panel">
    <div class="flex h-12 items-center border-b border-border px-4 font-medium text-text">
      {{ $t('nav.inbox') }}
    </div>
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="mail in mockMails"
        :key="mail.id"
        class="group relative cursor-pointer border-b border-border px-4 py-3 transition-colors hover:bg-card"
        :class="!mail.isRead ? 'bg-card/30' : ''"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span
              v-if="!mail.isRead"
              class="h-1.5 w-1.5 flex-shrink-0 rounded-full bg-primary"
            />
            <span class="text-sm font-medium" :class="!mail.isRead ? 'text-text' : 'text-text-secondary'">
              {{ mail.fromName }}
            </span>
          </div>
          <span class="flex-shrink-0 text-xs text-muted">{{ mail.time }}</span>
        </div>
        <div class="mt-1 truncate text-sm" :class="!mail.isRead ? 'text-text' : 'text-text-secondary'">
          {{ mail.subject }}
        </div>
        <div class="mt-0.5 truncate text-xs text-muted">{{ mail.snippet }}</div>
      </div>
    </div>
  </div>
</template>
