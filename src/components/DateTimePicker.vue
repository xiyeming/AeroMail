<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Bell, X } from '@lucide/vue';
import { PopoverContent, PopoverRoot, PopoverTrigger } from 'radix-vue';
import BaseSelect from '@/components/BaseSelect.vue';

const modelValue = defineModel<string | undefined>('modelValue');

const open = ref(false);

const dateValue = ref('');
const hourValue = ref('00');
const minuteValue = ref('00');

const hourOptions = Array.from({ length: 24 }, (_, i) => ({
  value: String(i).padStart(2, '0'),
  label: String(i).padStart(2, '0'),
}));

const minuteOptions = ['00', '15', '30', '45'].map((m) => ({
  value: m,
  label: m,
}));

function parseModelValue(value?: string) {
  if (!value) {
    dateValue.value = '';
    hourValue.value = '00';
    minuteValue.value = '00';
    return;
  }
  const [datePart, timePart] = value.split('T');
  dateValue.value = datePart ?? '';
  const [hour, minute] = (timePart ?? '').split(':');
  hourValue.value = hour ?? '00';
  minuteValue.value = minute ?? '00';
}

function emitUpdate() {
  if (!dateValue.value) {
    modelValue.value = undefined;
    return;
  }
  modelValue.value = `${dateValue.value}T${hourValue.value}:${minuteValue.value}`;
}

watch(modelValue, parseModelValue, { immediate: true });

const hasValue = computed(() => Boolean(modelValue.value));

function clearValue() {
  modelValue.value = undefined;
}
</script>

<template>
  <PopoverRoot v-model:open="open">
    <PopoverTrigger as-child>
      <button
        type="button"
        class="relative flex h-9 w-9 shrink-0 items-center justify-center rounded-md border transition-colors"
        :class="[
          hasValue
            ? 'border-accent/50 text-accent'
            : 'border-border text-secondary hover:bg-raised hover:text-primary',
        ]"
        :title="$t('todo.reminder')"
      >
        <Bell class="h-4 w-4" />
      </button>
    </PopoverTrigger>
    <PopoverContent
      side="bottom"
      :side-offset="4"
      align="end"
      class="z-50 w-64 rounded-lg border border-border bg-elevated p-3 shadow-lg"
    >
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium text-primary">{{ $t('todo.reminder') }}</span>
          <button
            v-if="hasValue"
            type="button"
            class="rounded p-1 text-tertiary transition-colors hover:text-primary"
            :title="$t('common.clear')"
            @click="clearValue"
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>

        <input
          v-model="dateValue"
          type="date"
          class="h-9 w-full rounded-md border border-border bg-base px-3 text-sm text-primary outline-none focus:border-accent"
          @change="emitUpdate"
        />

        <div class="flex items-center gap-2">
          <div class="flex-1">
            <BaseSelect
              v-model="hourValue"
              :options="hourOptions"
              size="sm"
              @update:model-value="emitUpdate"
            />
          </div>
          <span class="text-secondary">:</span>
          <div class="flex-1">
            <BaseSelect
              v-model="minuteValue"
              :options="minuteOptions"
              size="sm"
              @update:model-value="emitUpdate"
            />
          </div>
        </div>
      </div>
    </PopoverContent>
  </PopoverRoot>
</template>
