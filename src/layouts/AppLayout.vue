<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import { Minus, Square, X, Menu } from '@lucide/vue';
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
import SkeletonBlock from '@/components/SkeletonBlock.vue';

const aiStore = useAiStore();
const mailStore = useMailStore();
const todoStore = useTodoStore();
const { call } = useTauriInvoke();
const { decorations, initDecorations } = useWindowFrame();
const win = getCurrentWindow();

const { isWideScreen, isCollapsed, layoutMode } = useResponsive();

const sidebarWidth = computed(() => (isWideScreen.value ? 'w-64' : 'w-56'));
const mailListWidth = computed(() => (isWideScreen.value ? 'w-96' : 'w-80'));
const showCustomTitleBar = computed(() => decorations.value === 'none');

const showCloseConfirm = ref(false);
const initializing = ref(true);
const sidebarOpen = ref(false);

watch(isCollapsed, (collapsed) => {
  if (!collapsed) sidebarOpen.value = false;
});

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value;
}

function closeSidebar() {
  sidebarOpen.value = false;
}

let unlistenClose: UnlistenFn | undefined;
let unlistenFocus: UnlistenFn | undefined;

onMounted(async () => {
  await initDecorations();
  initializing.value = false;

  unlistenClose = await listen('app://close-requested', () => {
    showCloseConfirm.value = true;
  });

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
  await call('confirmed_exit');
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

    <!-- Skeleton loading state -->
    <div v-if="initializing" class="flex min-h-0 flex-1">
      <div class="w-64 shrink-0 border-r border-border bg-elevated p-3">
        <div class="mb-3 h-9 w-full rounded-md bg-raised" />
        <div class="space-y-1">
          <div v-for="i in 7" :key="i" class="flex items-center gap-3 rounded-md px-3 py-2">
            <SkeletonBlock shape="circle" class="h-4 w-4 shrink-0" />
            <SkeletonBlock class="h-4 w-24" />
          </div>
        </div>
      </div>
      <div class="flex min-w-0 flex-1 flex-col">
        <div class="flex min-h-0 flex-1">
          <div class="w-96 shrink-0 border-r border-border">
            <div class="flex h-12 items-center border-b border-border px-4">
              <SkeletonBlock class="h-4 w-24" />
            </div>
            <div class="flex flex-col">
              <div v-for="i in 10" :key="i" class="flex items-center gap-3 border-b border-border px-3 py-3">
                <SkeletonBlock shape="circle" class="h-4 w-4 shrink-0" />
                <div class="flex-1 space-y-2">
                  <div class="flex items-center justify-between gap-2">
                    <SkeletonBlock class="h-3.5 w-1/3" />
                    <SkeletonBlock class="h-3 w-8 shrink-0" />
                  </div>
                  <SkeletonBlock class="h-3 w-2/3" />
                  <SkeletonBlock class="h-3 w-1/3" />
                </div>
              </div>
            </div>
          </div>
          <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
            <slot />
          </main>
        </div>
        <div class="shrink-0 border-t border-border p-2">
          <SkeletonBlock class="h-4 w-32" />
        </div>
      </div>
    </div>

    <!-- Main content -->
    <template v-else>
      <div class="flex min-h-0 flex-1">
        <div
          class="sr-only focus-within:not-sr-only focus-within:absolute focus-within:z-50 focus-within:bg-accent focus-within:p-2 focus-within:text-white"
        >
          <a href="#mail-list" class="mr-4 underline">{{ $t('mail.skipToList') }}</a>
          <a href="#reader" class="underline">{{ $t('mail.skipToReader') }}</a>
        </div>

        <!-- Hamburger menu for collapsed sidebar -->
        <button
          v-if="isCollapsed && !mailStore.isReadingMode"
          type="button"
          class="flex h-12 w-10 shrink-0 items-center justify-center border-r border-border text-secondary transition-colors hover:bg-raised hover:text-primary"
          :aria-label="$t('nav.settings')"
          @click="toggleSidebar"
        >
          <Menu class="h-5 w-5" />
        </button>

        <!-- Sidebar overlay for collapsed mode -->
        <Teleport to="body">
          <div
            v-if="isCollapsed && sidebarOpen"
            class="fixed inset-0 z-40 bg-black/50"
            @click="closeSidebar"
          />
        </Teleport>

        <AppSidebar
          v-show="!mailStore.isReadingMode"
          :class="[
            'shrink-0 overflow-hidden border-r border-border transition-all duration-200',
            isCollapsed
              ? sidebarOpen
                ? 'fixed left-0 top-0 z-50 h-full w-56 opacity-100'
                : 'w-0 opacity-0'
              : sidebarWidth,
            !isCollapsed ? 'opacity-100' : '',
          ]"
          @click="isCollapsed && closeSidebar()"
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
    </template>

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
