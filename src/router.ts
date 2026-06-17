import { createRouter, createWebHistory } from 'vue-router';
import InboxView from './views/InboxView.vue';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'inbox',
      component: InboxView,
    },
  ],
});

export default router;
