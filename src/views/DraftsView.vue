<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useComposeStore } from '@/stores/compose';
import { FileText, Trash2 } from '@lucide/vue';

const router = useRouter();
const composeStore = useComposeStore();

onMounted(() => {
  void composeStore.loadDrafts();
});

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

function openDraft(draftId: string) {
  void router.push({ name: 'compose-draft', params: { draftId } });
}

async function deleteDraft(draftId: string, event: MouseEvent) {
  event.stopPropagation();
  await composeStore.deleteDraft(draftId);
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="border-b border-border px-4 py-3">
      <h1 class="text-h1 font-semibold text-text">{{ $t('drafts.title') }}</h1>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div v-if="composeStore.drafts.length === 0" class="p-8 text-center text-muted">
        {{ $t('drafts.noDrafts') }}
      </div>

      <ul v-else class="divide-y divide-border">
        <li
          v-for="draft in composeStore.drafts"
          :key="draft.id"
          class="group flex cursor-pointer items-center justify-between px-4 py-3 hover:bg-card"
          @click="openDraft(draft.id)"
        >
          <div class="flex min-w-0 items-center gap-3">
            <FileText class="h-4 w-4 shrink-0 text-muted" />
            <div class="min-w-0">
              <p class="truncate text-sm font-medium text-text">
                {{ draft.subject || $t('drafts.noSubject') }}
              </p>
              <p class="truncate text-xs text-muted">
                {{ $t('drafts.to') }}: {{ draft.to.join(', ') || $t('drafts.noRecipient') }}
              </p>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-3">
            <span class="text-xs text-muted">{{ formatDate(draft.savedAt) }}</span>
            <button
              class="rounded p-1 text-muted opacity-0 transition-opacity hover:text-danger group-hover:opacity-100"
              @click="deleteDraft(draft.id, $event)"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
