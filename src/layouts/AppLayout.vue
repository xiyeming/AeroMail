<script setup lang="ts">
import { computed } from 'vue';
import { useResponsive } from '@/composables/useResponsive';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import AppSidebar from '@/components/AppSidebar.vue';
import MailList from '@/components/MailList.vue';
import BulkActions from '@/components/BulkActions.vue';
import StatusBar from '@/components/StatusBar.vue';
import AiAssistantPanel from '@/components/AiAssistantPanel.vue';

const aiStore = useAiStore();
const mailStore = useMailStore();

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-[260px]' : 'w-[240px]'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-[480px]' : 'w-[420px]'));
</script>

<template>
  <div class="flex h-screen w-screen flex-col bg-background text-text">
    <div class="flex flex-1 overflow-hidden">
      <AppSidebar
        v-show="!mailStore.isReadingMode"
        :class="[
          'flex-shrink-0 overflow-hidden transition-all duration-200',
          sidebarWidth,
          isCollapsed ? 'w-0 opacity-0' : 'opacity-100',
        ]"
      />

      <div
        v-show="!mailStore.isReadingMode"
        :class="[
          'flex flex-col flex-shrink-0 border-r border-border',
          mailListWidth,
          layoutMode === 'mobile' ? 'hidden' : 'block',
        ]"
      >
        <BulkActions />
        <MailList class="flex-1 min-h-0" />
      </div>

      <main
        :class="[
          'flex min-w-0 flex-col overflow-hidden',
          mailStore.isReadingMode ? 'flex-1' : 'flex-1',
        ]"
      >
        <slot />
      </main>

      <AiAssistantPanel v-if="aiStore.isPanelOpen" />
    </div>
    <StatusBar v-show="!mailStore.isReadingMode" />
  </div>
</template>
