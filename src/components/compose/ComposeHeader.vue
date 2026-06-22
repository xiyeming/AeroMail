<template>
  <div class="flex flex-col gap-3 border-b border-border p-4">
    <div class="flex items-center gap-3">
      <select
        :value="draft.accountId"
        class="rounded-lg border border-border bg-panel px-3 py-1.5 text-sm text-text outline-none focus:border-primary"
        @change="updateAccount(($event.target as HTMLSelectElement).value)"
      >
        <option value="" disabled>{{ $t('compose.selectAccount') }}</option>
        <option v-for="acc in accounts" :key="acc.id" :value="acc.id">
          {{ acc.name }} ({{ acc.smtpHost }})
        </option>
      </select>
      <button
        type="button"
        class="ml-auto rounded-lg bg-primary px-4 py-1.5 text-sm font-medium text-white transition-colors hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="!canSend"
        @click="$emit('send')"
      >
        {{ $t('compose.send') }}
      </button>
    </div>

    <RecipientInput
      v-model="localDraft.to"
      :label="$t('compose.to')"
      :placeholder="$t('compose.toPlaceholder')"
    />

    <div class="flex items-center gap-3">
      <button type="button" class="text-xs text-primary hover:underline" @click="showCc = !showCc">
        {{ $t('compose.cc') }}
      </button>
      <button
        type="button"
        class="text-xs text-primary hover:underline"
        @click="showBcc = !showBcc"
      >
        {{ $t('compose.bcc') }}
      </button>
    </div>

    <RecipientInput v-if="showCc" v-model="localDraft.cc" :label="$t('compose.cc')" />
    <RecipientInput v-if="showBcc" v-model="localDraft.bcc" :label="$t('compose.bcc')" />

    <input
      v-model="localDraft.subject"
      type="text"
      class="rounded-lg border border-border bg-panel px-3 py-1.5 text-sm text-text outline-none placeholder:text-muted focus:border-primary"
      :placeholder="$t('compose.subject')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ComposeDraft } from '@/types/compose';
import type { AccountSummary } from '@/types/account';
import RecipientInput from './RecipientInput.vue';

const props = defineProps<{
  draft: ComposeDraft;
  accounts: AccountSummary[];
}>();

const emit = defineEmits<{
  (e: 'update:draft', value: ComposeDraft): void;
  (e: 'send'): void;
}>();

const localDraft = computed({
  get: () => props.draft,
  set: (v) => emit('update:draft', v),
});

const showCc = ref(false);
const showBcc = ref(false);

const canSend = computed(() => {
  return props.draft.accountId !== '' && props.draft.to.length > 0 && props.draft.subject !== '';
});

function updateAccount(accountId: string) {
  localDraft.value = { ...props.draft, accountId };
}

watch(
  () => props.draft.cc,
  (v) => {
    if (v.length > 0) showCc.value = true;
  },
  { immediate: true }
);
watch(
  () => props.draft.bcc,
  (v) => {
    if (v.length > 0) showBcc.value = true;
  },
  { immediate: true }
);
</script>
