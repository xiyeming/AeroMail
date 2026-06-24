import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';
import { i18n, loadLocaleMessages, type Locale } from './i18n';
import './styles/theme.css';
import './styles/fonts.css';

async function bootstrap() {
  const locale: Locale = 'en';
  await loadLocaleMessages(locale);
  i18n.global.locale.value = locale;

  const app = createApp(App);
  app.config.errorHandler = (err, instance, info) => {
    console.error('[Vue global error]', err, info, instance);
  };
  app.use(createPinia());
  app.use(router);
  app.use(i18n);
  app.mount('#app');
}

void bootstrap();
