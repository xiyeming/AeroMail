import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  // 🛠️ 新增：针对 Tauri 客户端生产环境的打包优化
  build: {
    // 1. 调整大文件警告阈值到 1000 kB（Tauri 本地加载非常快，800k 其实不会卡顿，但分包更规范）
    chunkSizeWarningLimit: 1000,

    // 2. 针对 Rolldown / Rollup 的分包策略
    rollupOptions: {
      output: {
        manualChunks(id) {
          // 将 node_modules 中的公共依赖（如 vue, lucide-vue-next, @vueuse）提取到独立的 vendor 块中
          if (id.includes('node_modules')) {
            return 'vendor';
          }
        },
      },
    },
  },
});