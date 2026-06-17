<script setup lang="ts">
import AccountForm from '@/components/AccountForm.vue';
import AccountList from '@/components/AccountList.vue';
</script>

<template>
  <div class="flex h-full flex-col overflow-y-auto bg-panel">
    <AccountForm />
    <AccountList />
  </div>
</template>
