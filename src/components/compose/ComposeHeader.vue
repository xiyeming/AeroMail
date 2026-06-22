<template>
  <div class="flex flex-col gap-3 border-b border-border bg-elevated p-4">
    <div class="flex items-center gap-3">
      <div class="flex flex-1 items-center gap-2">
        <label for="compose-account" class="text-sm text-secondary">{{ $t('compose.selectAccount') }}</label>
        <div class="flex-1">
          <BaseSelect
            id="compose-account"
            :model-value="draft.accountId"
            :placeholder="$t('compose.selectAccount')"
            :options="accounts.map((acc) => ({ value: acc.id, label: acc.name }))"
            @update:model-value="updateAccount"
          />
        </div>
      </div>

      <div class="relative">
        <button
          type="button"
          class="rounded-md bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
          :disabled="!canSend"
          @click="$emit('send')"
        >
          {{ $t('compose.send') }}
        </button>
        <div
          v-if="!canSend"
          class="pointer-events-none absolute right-0 top-full mt-1 w-max max-w-xs rounded-md border border-border bg-raised px-2 py-1 text-xs text-secondary shadow-md"
        >
          {{ $t('compose.sendDisabledHint') }}
        </div>
      </div>
    </div>

    <RecipientInput
      v-model="localDraft.to"
      :label="$t('compose.to')"
      :placeholder="$t('compose.toPlaceholder')"
    />

    <div class="flex items-center gap-3">
      <button
        type="button"
        class="text-xs text-accent hover:underline"
        @click="showCc = !showCc"
      >
        {{ $t('compose.cc') }}
      </button>
      <button
        type="button"
        class="text-xs text-accent hover:underline"
        @click="showBcc = !showBcc"
      >
        {{ $t('compose.bcc') }}
      </button>
    </div>

    <RecipientInput v-if="showCc" v-model="localDraft.cc" :label="$t('compose.cc')" />
    <RecipientInput v-if="showBcc" v-model="localDraft.bcc" :label="$t('compose.bcc')" />

    <div class="flex items-center gap-2">
      <label for="compose-subject" class="text-sm text-secondary">{{ $t('compose.subject') }}</label>
      <input
        id="compose-subject"
        v-model="localDraft.subject"
        type="text"
        class="h-9 flex-1 rounded-md border border-border bg-base px-3 text-sm text-primary outline-none placeholder:text-tertiary focus:border-accent"
        :placeholder="$t('compose.subjectPlaceholder')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ComposeDraft } from '@/types/compose';
import type { AccountSummary } from '@/types/account';
import BaseSelect from '@/components/BaseSelect.vue';
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
  return props.draft.accountId !== '' && props.draft.to.length > 0;
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
