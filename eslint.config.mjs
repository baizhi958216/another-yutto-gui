// eslint.config.mjs
import antfu from '@antfu/eslint-config'

export default antfu({
  // Enable Vue support
  vue: true,

  // Enable TypeScript support
  typescript: true,

  // Enable stylistic formatting rules
  stylistic: {
    indent: 2,
    quotes: 'single',
  },

  // Enable CSS/SCSS formatting
  formatters: {
    css: true,
    html: true,
  },

  // Parse .gitignore to get ignores
  gitignore: true,

  ignores: [
    'src-tauri',
  ],

  unocss: true,

  rules: {
    '@typescript-eslint/no-empty-object-type': 0,
    'unused-imports/no-unused-vars': [
      'off',
      {
        vars: 'all',
        varsIgnorePattern: '^_',
        args: 'after-used',
        argsIgnorePattern: '^_',
      },
    ],
    'no-console': 'off',
    'no-alert': 'off',
  },

})
