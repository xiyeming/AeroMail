<script setup lang="ts">
import { computed } from 'vue';
import { useResponsive } from '@/composables/useResponsive';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import AppSidebar from '@/components/AppSidebar.vue';
import MailList from '@/components/MailList.vue';
import StatusBar from '@/components/StatusBar.vue';
import AiAssistantPanel from '@/components/AiAssistantPanel.vue';
import ToastContainer from '@/components/ToastContainer.vue';
import CommandPalette from '@/components/CommandPalette.vue';

const aiStore = useAiStore();
const mailStore = useMailStore();

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-64' : 'w-56'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-96' : 'w-80'));
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-base text-primary">
    <div
      class="sr-only focus-within:not-sr-only focus-within:absolute focus-within:z-50 focus-within:bg-accent focus-within:p-2 focus-within:text-white"
    >
      <a href="#mail-list" class="mr-4 underline">{{ $t('mail.skipToList') }}</a>
      <a href="#reader" class="underline">{{ $t('mail.skipToReader') }}</a>
    </div>

    <AppSidebar
      v-show="!mailStore.isReadingMode"
      :class="[
        'shrink-0 overflow-hidden border-r border-border transition-all duration-200',
        sidebarWidth,
        isCollapsed ? 'w-0 opacity-0' : 'opacity-100',
      ]"
    />

    <div class="flex min-w-0 flex-1 flex-col">
      <div class="flex min-h-0 flex-1">
        <div
          id="mail-list"
          v-show="!mailStore.isReadingMode"
          :class="[
            'flex shrink-0 flex-col border-r border-border',
            mailListWidth,
            layoutMode === 'mobile' ? 'hidden' : 'flex',
          ]"
        >
          <MailList class="flex-1 min-h-0" />
        </div>

        <main id="reader" class="flex min-w-0 flex-1 flex-col overflow-hidden">
          <slot />
        </main>
      </div>

      <StatusBar class="shrink-0 border-t border-border" />
    </div>

    <AiAssistantPanel v-if="aiStore.isPanelOpen" />
    <ToastContainer />
    <CommandPalette />
  </div>
</template>
