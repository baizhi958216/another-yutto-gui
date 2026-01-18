import {
  defineConfig,
  presetIcons,
  presetWind4,
  transformerDirectives,
} from 'unocss'

export default defineConfig({
  presets: [
    presetWind4(),
    presetIcons({
      cdn: 'https://esm.sh/',
    }),
  ],
  transformers: [
    transformerDirectives(),
  ],
  theme: {
    colors: {
      // 主色调 - Teal (青色)
      primary: '#14b8a6', // teal-500
      secondary: '#0d9488', // teal-600
      // 背景色
      bg: {
        primary: '#FFFFFF',
        secondary: '#F8F9FA',
        tertiary: '#F0F4F8',
      },
      // 文字色
      text: {
        primary: '#2C3E50',
        secondary: '#6C757D',
        tertiary: '#ADB5BD',
      },
      // 状态色（柔和版本）
      success: '#4CAF50',
      error: '#EF5350',
      warning: '#FFA726',
      info: '#29B6F6',
    },
    fontFamily: {
      sans: ['Quicksand', 'sans-serif'],
    },
  },
  shortcuts: {
    // 按钮样式
    'btn': 'px-4 py-2 rounded inline-block bg-teal-500 text-white cursor-pointer hover:bg-teal-600 disabled:cursor-default disabled:bg-gray-600 disabled:opacity-50 transition-all active:scale-95',
    'btn-primary': 'bg-teal-500 hover:bg-teal-600 text-white shadow-lg shadow-teal-100 px-4 py-2 rounded transition-all active:scale-95 cursor-pointer',
    'btn-secondary': 'bg-gray-200 hover:bg-gray-300 text-gray-700 px-4 py-2 rounded transition-all active:scale-95 cursor-pointer',
    // 卡片样式
    'card': 'bg-white rounded-2xl border border-gray-100 shadow-sm p-6',
    'card-hover': 'bg-white rounded-3xl border border-gray-100 shadow-sm p-6 hover:shadow-lg transition-all cursor-pointer',
    // 输入框样式
    'input-base': 'w-full px-4 py-2.5 bg-white border border-gray-100 rounded-xl focus:ring-2 ring-teal-100 outline-none shadow-sm transition-all',
    // 下拉框样式
    'select-base': 'w-full px-4 py-2.5 bg-white border border-gray-100 rounded-xl focus:ring-2 ring-teal-100 outline-none shadow-sm transition-all cursor-pointer',
    // 页面容器
    'page-container': 'mx-auto max-w-6xl p-6',
    // 渐变背景
    'bg-gradient-soft': 'bg-gradient-to-br from-teal-50 via-white to-blue-50',
  },
})
