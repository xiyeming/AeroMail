import { createRouter, createWebHistory } from 'vue-router';
import MailViewer from './components/MailViewer.vue';
import SettingsErrorBoundary from './components/SettingsErrorBoundary.vue';
import ComposeView from './views/ComposeView.vue';
import DraftsView from './views/DraftsView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'inbox',
      component: MailViewer,
    },
    {
      path: '/folder/:folderId',
      name: 'folder',
      component: MailViewer,
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsErrorBoundary,
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
