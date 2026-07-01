<script setup lang="ts">
import { computed } from 'vue';
import { ChevronDown } from '@lucide/vue';
import {
  SelectContent,
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  SelectRoot,
  SelectTrigger,
  SelectValue,
  SelectViewport,
} from 'radix-vue';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: SelectOption[];
    placeholder?: string;
    id?: string;
    disabled?: boolean;
    size?: 'sm' | 'md';
    variant?: 'base' | 'elevated';
  }>(),
  {
    size: 'md',
    variant: 'base',
    placeholder: '',
    id: undefined,
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const selectedValue = computed({
  get: () => props.modelValue,
  set: (value: string) => emit('update:modelValue', value),
});

const sizeClasses = computed(() => (props.size === 'sm' ? 'h-8 px-2 text-xs' : 'h-9 px-3 text-sm'));

const variantClasses = computed(() => (props.variant === 'elevated' ? 'bg-elevated' : 'bg-base'));
</script>

<template>
  <SelectRoot v-model="selectedValue">
    <SelectTrigger
      :id="id"
      :disabled="disabled"
      class="group flex w-full items-center justify-between gap-2 rounded-md border border-border outline-none transition-colors focus:border-accent disabled:cursor-not-allowed disabled:opacity-50"
      :class="[sizeClasses, variantClasses]"
    >
      <SelectValue :placeholder="placeholder" class="truncate text-primary" />
      <ChevronDown
        class="h-4 w-4 shrink-0 text-tertiary transition-transform group-data-[state=open]:rotate-180"
        aria-hidden="true"
      />
    </SelectTrigger>

    <SelectContent
      position="popper"
      :side-offset="4"
      class="z-50 min-w-[var(--radix-select-trigger-width)] overflow-hidden rounded-md border border-border bg-elevated shadow-md"
    >
      <SelectViewport class="max-h-60 overflow-y-auto py-1">
        <SelectItem
          v-for="option in options"
          :key="option.value"
          :value="option.value"
          :disabled="option.disabled"
          class="relative flex h-8 cursor-pointer select-none items-center px-3 text-sm text-secondary outline-none transition-colors data-[disabled]:cursor-not-allowed data-[highlighted]:bg-raised data-[highlighted]:text-primary data-[state=checked]:bg-accent-subtle data-[state=checked]:text-accent"
        >
          <SelectItemText class="flex-1 truncate">
            {{ option.label }}
          </SelectItemText>
          <SelectItemIndicator class="ml-2 text-accent">
            <svg
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              aria-hidden="true"
            >
              <path
                d="M2 6L5 9L10 3"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </SelectItemIndicator>
        </SelectItem>
      </SelectViewport>
    </SelectContent>
  </SelectRoot>
</template>
