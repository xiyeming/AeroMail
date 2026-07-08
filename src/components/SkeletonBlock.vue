<template>
  <div
    class="animate-pulse rounded-md bg-raised"
    :class="[shape === 'circle' ? 'rounded-full' : 'rounded-md', $attrs.class]"
    role="presentation"
    aria-hidden="true"
  />
</template>

<script setup lang="ts">
defineProps<{
  shape?: 'rect' | 'circle';
}>();
</script>
