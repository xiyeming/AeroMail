import { createRouter, createWebHistory } from 'vue-router';
import InboxView from './views/InboxView.vue';
import SettingsView from './views/SettingsView.vue';
import ComposeView from './views/ComposeView.vue';
import DraftsView from './views/DraftsView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'inbox',
      component: InboxView,
    },
    {
      path: '/folder/:folderId',
      name: 'folder',
      component: InboxView,
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView,
    },
    {
      path: '/compose',
      name: 'compose',
      component: ComposeView,
    },
    {
      path: '/compose/:draftId',
      name: 'compose-draft',
      component: ComposeView,
    },
    {
      path: '/drafts',
      name: 'drafts',
      component: DraftsView,
    },
    {
      path: '/reply/:mailId',
      name: 'reply',
      component: ComposeView,
    },
    {
      path: '/reply-all/:mailId',
      name: 'reply-all',
      component: ComposeView,
    },
    {
      path: '/forward/:mailId',
      name: 'forward',
      component: ComposeView,
    },
  ],
});

export default router;
