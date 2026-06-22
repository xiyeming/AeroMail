<template>
  <div v-if="attachments.length > 0" class="flex flex-wrap gap-2 border-t border-border p-4">
    <div
      v-for="att in attachments"
      :key="att.id"
      class="flex items-center gap-2 rounded-lg border border-border bg-panel px-3 py-1.5 text-sm"
    >
      <span class="text-text">{{ att.filename }}</span>
      <span class="text-xs text-muted">{{ formatSize(att.size) }}</span>
      <button
        type="button"
        class="text-text-secondary hover:text-danger"
        @click="$emit('remove', att.id)"
      >
        ×
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AttachmentDraft } from '@/types/compose';

defineProps<{
  attachments: AttachmentDraft[];
}>();

defineEmits<{
  (e: 'remove', attachmentId: string): void;
}>();

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
</script>
