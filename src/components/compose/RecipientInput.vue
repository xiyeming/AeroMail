<template>
  <div class="recipient-input flex flex-col gap-1">
    <label class="text-xs text-muted">{{ label }}</label>
    <div
      class="flex flex-wrap gap-1 rounded-lg border border-border bg-panel p-2 transition-colors focus-within:border-primary"
      @click="focusInput"
    >
      <span
        v-for="(email, idx) in modelValue"
        :key="idx"
        class="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-sm text-text"
      >
        {{ email }}
        <button
          type="button"
          class="text-text-secondary hover:text-danger"
          @click.stop="removeEmail(idx)"
        >
          ×
        </button>
      </span>
      <input
        ref="inputRef"
        v-model="inputValue"
        type="text"
        class="min-w-[120px] flex-1 bg-transparent px-1 py-0.5 text-sm text-text outline-none placeholder:text-muted"
        :placeholder="modelValue.length === 0 ? placeholder : ''"
        @keydown.enter.prevent="addEmail"
        @keydown.tab.prevent="addEmail"
        @keydown.backspace="onBackspace"
        @blur="addEmail"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  modelValue: string[];
  label: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string[]): void;
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function focusInput() {
  inputRef.value?.focus();
}

function addEmail() {
  const raw = inputValue.value.trim();
  if (!raw) return;
  const addresses = raw
    .split(/[,;\n]+/)
    .map((a) => a.trim())
    .filter(Boolean);
  const valid = addresses.filter((a) => EMAIL_REGEX.test(a));
  if (valid.length > 0) {
    emit('update:modelValue', [...props.modelValue, ...valid]);
  }
  inputValue.value = '';
}

function removeEmail(idx: number) {
  const updated = [...props.modelValue];
  updated.splice(idx, 1);
  emit('update:modelValue', updated);
}

function onBackspace() {
  if (inputValue.value === '' && props.modelValue.length > 0) {
    const updated = [...props.modelValue];
    updated.pop();
    emit('update:modelValue', updated);
  }
}
</script>
