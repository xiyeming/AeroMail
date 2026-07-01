<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Minus, Square, X } from '@lucide/vue';
import { useResponsive } from '@/composables/useResponsive';
import { useWindowFrame } from '@/composables/useWindowFrame';
import { useAiStore } from '@/stores/ai';
import { useMailStore } from '@/stores/mail';
import { useTodoStore } from '@/stores/todo';
import AppSidebar from '@/components/AppSidebar.vue';
import MailList from '@/components/MailList.vue';
import StatusBar from '@/components/StatusBar.vue';
import AiAssistantPanel from '@/components/AiAssistantPanel.vue';
import TodoPanel from '@/components/TodoPanel.vue';
import ToastContainer from '@/components/ToastContainer.vue';
import CommandPalette from '@/components/CommandPalette.vue';

const aiStore = useAiStore();
const mailStore = useMailStore();
const todoStore = useTodoStore();
const { decorations } = useWindowFrame();
const win = getCurrentWindow();

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-64' : 'w-56'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-96' : 'w-80'));
const showCustomTitleBar = computed(() => decorations.value === 'none');

const showCloseConfirm = ref(false);
let unlistenClose: UnlistenFn | undefined;
let unlistenFocus: UnlistenFn | undefined;

onMounted(async () => {
  unlistenClose = await listen('app://close-requested', () => {
    showCloseConfirm.value = true;
  });

  // When the window is minimized through the system title bar, it loses focus
  // while `isMinimized()` is true. Hide it so it lands in the tray instead of
  // staying in the taskbar.
  unlistenFocus = await win.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      void win.isMinimized().then((minimized) => {
        if (minimized) {
          void win.hide();
        }
      });
    }
  });
});

onUnmounted(() => {
  if (unlistenClose) {
    unlistenClose();
  }
  if (unlistenFocus) {
    unlistenFocus();
  }
});

async function minimizeWindow() {
  await win.hide();
}

async function toggleMaximizeWindow() {
  if (await win.isMaximized()) {
    await win.unmaximize();
  } else {
    await win.maximize();
  }
}

async function closeWindow() {
  showCloseConfirm.value = true;
}

async function confirmClose() {
  showCloseConfirm.value = false;
  await invoke('confirmed_exit');
}

function cancelClose() {
  showCloseConfirm.value = false;
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
    <TodoPanel v-if="todoStore.isPanelOpen" />
    <ToastContainer />
    <CommandPalette />

    <!-- Close confirmation dialog -->
    <div
      v-if="showCloseConfirm"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      :aria-label="$t('common.closeConfirmTitle')"
    >
      <div class="w-full max-w-sm rounded-lg border border-border bg-elevated p-5 shadow-lg">
        <h3 class="mb-2 text-lg font-medium text-primary">{{ $t('common.closeConfirmTitle') }}</h3>
        <p class="mb-5 text-sm text-secondary">{{ $t('common.closeConfirmMessage') }}</p>
        <div class="flex justify-end gap-2">
          <button
            type="button"
            class="rounded-md border border-border bg-base px-4 py-2 text-sm text-primary transition-colors hover:bg-raised"
            @click="cancelClose"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            type="button"
            class="rounded-md bg-danger px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-danger-hover"
            @click="confirmClose"
          >
            {{ $t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
