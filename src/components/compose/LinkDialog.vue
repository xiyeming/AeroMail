<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { X, Link as LinkIcon } from '@lucide/vue';

const props = defineProps<{
  open: boolean;
  initialUrl?: string;
  initialText?: string;
}>();

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void;
  (e: 'confirm', url: string, text: string): void;
  (e: 'remove'): void;
}>();

const { t } = useI18n();
const url = ref('');
const text = ref('');

watch(
  () => props.open,
  (val) => {
    if (val) {
      url.value = props.initialUrl || '';
      text.value = props.initialText || '';
    }
  }
);

function confirm() {
  if (url.value.trim() && text.value.trim()) {
    // Auto-add https:// if no protocol
    let finalUrl = url.value.trim();
    if (!/^https?:\/\//i.test(finalUrl)) {
      finalUrl = `https://${finalUrl}`;
    }
    emit('confirm', finalUrl, text.value.trim());
  }
  emit('update:open', false);
}

function remove() {
  emit('remove');
  emit('update:open', false);
}

function cancel() {
  emit('update:open', false);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault();
    confirm();
  } else if (e.key === 'Escape') {
    cancel();
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-center justify-center bg-overlay"
        @click.self="cancel"
      >
        <div
          class="w-full max-w-md rounded-xl border border-border bg-elevated shadow-lg"
          @keydown="onKeydown"
        >
          <!-- Header -->
          <div class="flex items-center justify-between border-b border-border px-5 py-3">
            <div class="flex items-center gap-2 text-sm font-medium text-primary">
              <LinkIcon class="h-4 w-4 text-accent" />
              {{ t('compose.insertLink') }}
            </div>
            <button
              type="button"
              class="flex h-6 w-6 items-center justify-center rounded-md text-tertiary transition-colors hover:bg-raised hover:text-primary"
              @click="cancel"
            >
              <X class="h-4 w-4" />
            </button>
          </div>

          <!-- Body -->
          <div class="px-5 py-4">
            <label class="mb-1.5 block text-xs font-medium text-secondary">
              {{ t('compose.linkText') }}
            </label>
            <input
              v-model="text"
              type="text"
              :placeholder="t('compose.linkTextPlaceholder')"
              class="mb-3 h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none transition-colors placeholder:text-tertiary focus:border-accent"
            />
            <label class="mb-1.5 block text-xs font-medium text-secondary">
              {{ t('compose.linkUrl') }}
            </label>
            <input
              v-model="url"
              type="url"
              :placeholder="t('compose.linkPlaceholder')"
              class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none transition-colors placeholder:text-tertiary focus:border-accent"
              autofocus
            />
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-between border-t border-border px-5 py-3">
            <button
              v-if="initialUrl"
              type="button"
              class="rounded-md px-3 py-1.5 text-xs text-danger transition-colors hover:bg-danger-subtle"
              @click="remove"
            >
              {{ t('compose.removeLink') }}
            </button>
            <div v-else />
            <div class="flex items-center gap-2">
              <button
                type="button"
                class="rounded-md border border-border px-3 py-1.5 text-xs text-secondary transition-colors hover:bg-raised hover:text-primary"
                @click="cancel"
              >
                {{ t('compose.cancel') }}
              </button>
              <button
                type="button"
                class="rounded-md bg-accent px-4 py-1.5 text-xs font-medium text-white transition-colors hover:bg-accent-hover"
                @click="confirm"
              >
                {{ t('compose.confirm') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
