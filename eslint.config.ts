import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import vueeslint from 'eslint-plugin-vue';
import globals from 'globals';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'target/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      'node_modules/**',
      '.pnpm-store/**',
    ],
  },
  eslint.configs.recommended,
  tseslint.configs.recommended,
  ...vueeslint.configs['flat/recommended'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
  },
  {
    files: ['*.vue', '**/*.vue'],
    languageOptions: {
      parser: vueeslint.parser,
      parserOptions: {
        parser: tseslint.parser,
        sourceType: 'module',
      },
    },
  },
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      'vue/multi-word-component-names': 'off',
    },
  },
  prettier
);
