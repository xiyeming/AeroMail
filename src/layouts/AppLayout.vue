<script setup lang="ts">
import { computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X } from 'lucide-vue-next';
import { useResponsive } from '@/composables/useResponsive';
import { useWindowFrame } from '@/composables/useWindowFrame';
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
const { decorations } = useWindowFrame();
const win = getCurrentWindow();

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-64' : 'w-56'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-96' : 'w-80'));
const showCustomTitleBar = computed(() => decorations.value === 'none');

async function minimizeWindow() {
  await win.minimize();
}

async function toggleMaximizeWindow() {
  if (await win.isMaximized()) {
    await win.unmaximize();
  } else {
    await win.maximize();
  }
}

async function closeWindow() {
  await win.close();
}
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-base text-primary">
    <div
      v-if="showCustomTitleBar"
      class="flex h-9 shrink-0 items-center justify-between border-b border-border bg-elevated px-3"
      data-tauri-drag-region
    >
      <span class="select-none text-xs font-medium text-primary">{{ $t('app.name') }}</span>
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="flex h-6 w-6 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised hover:text-primary"
          :aria-label="$t('common.minimize')"
          @click="minimizeWindow"
        >
          <Minus class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="flex h-6 w-6 items-center justify-center rounded-md text-secondary transition-colors hover:bg-raised hover:text-primary"
          :aria-label="$t('common.maximize')"
          @click="toggleMaximizeWindow"
        >
          <Square class="h-3.5 w-3.5" />
        </button>
        <button
          type="button"
          class="flex h-6 w-6 items-center justify-center rounded-md text-secondary transition-colors hover:bg-danger-subtle hover:text-danger"
          :aria-label="$t('common.close')"
          @click="closeWindow"
        >
          <X class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <div class="flex min-h-0 flex-1">
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
            v-show="!mailStore.isReadingMode"
            id="mail-list"
            :class="[
              'flex shrink-0 flex-col border-r border-border',
              mailListWidth,
              layoutMode === 'mobile' ? 'hidden' : 'flex',
            ]"
          >
            <MailList class="min-h-0 flex-1" />
          </div>

          <main id="reader" class="flex min-w-0 flex-1 flex-col overflow-hidden">
            <slot />
          </main>
        </div>

        <StatusBar class="shrink-0 border-t border-border" />
      </div>
    </div>

    <AiAssistantPanel v-if="aiStore.isPanelOpen" />
    <ToastContainer />
    <CommandPalette />
  </div>
</template>
