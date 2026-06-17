<script setup lang="ts">
import { RouterView } from 'vue-router';
import CommandPalette from '@/components/CommandPalette.vue';
import ToastContainer from '@/components/ToastContainer.vue';
</script>

<template>
  <RouterView />
  <CommandPalette />
  <ToastContainer />
</template>
