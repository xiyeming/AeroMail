import { createRouter, createWebHistory } from 'vue-router';
import InboxView from './views/InboxView.vue';
import AccountsView from './views/AccountsView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'inbox',
      component: InboxView,
    },
    {
      path: '/accounts',
      name: 'accounts',
      component: AccountsView,
    },
  ],
});

export default router;
