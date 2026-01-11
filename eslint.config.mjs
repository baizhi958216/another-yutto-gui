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

  // Parse .gitignore to get ignores
  gitignore: true,

  ignores: [
    'src-tauri',
  ],

  rules: {
    '@typescript-eslint/no-empty-object-type': 0,
  },

})
