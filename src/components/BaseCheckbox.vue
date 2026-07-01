<script setup lang="ts">
import { Check } from '@lucide/vue';

const props = defineProps<{
  modelValue: boolean;
  id?: string;
  ariaLabel?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

function handleChange(event: Event) {
  emit('update:modelValue', (event.target as HTMLInputElement).checked);
}
</script>

<template>
  <label class="inline-flex cursor-pointer items-center gap-2 select-none">
    <span
      class="relative flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors hover:border-accent peer-focus-within:ring-2 peer-focus-within:ring-accent"
      :class="
        modelValue ? 'border-accent bg-accent text-white' : 'border-border bg-base text-transparent'
      "
    >
      <input
        :id="id"
        type="checkbox"
        :checked="props.modelValue"
        class="peer sr-only"
        :aria-label="ariaLabel"
        @change="handleChange"
      />
      <Check class="h-3 w-3" />
    </span>
    <span class="text-sm text-secondary">
      <slot />
    </span>
  </label>
</template>
