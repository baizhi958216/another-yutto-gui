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
      // 主色调 - 柔和的蓝色
      primary: {
        50: '#E3F2FD',
        100: '#BBDEFB',
        200: '#90CAF9',
        300: '#64B5F6',
        400: '#42A5F5',
        500: '#2196F3',
        600: '#1E88E5',
        700: '#1976D2',
      },
      // 点缀色 - 淡紫色
      accent: {
        50: '#F3E5F5',
        100: '#E1BEE7',
        200: '#CE93D8',
        300: '#BA68C8',
      },
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
  },
  shortcuts: {
    // 按钮样式
    'btn-primary': 'bg-primary-400 text-white px-4 py-2 rounded-lg hover:bg-primary-500 active:bg-primary-600 transition-all duration-200 cursor-pointer shadow-sm hover:shadow-md',
    'btn-secondary': 'bg-white text-primary-500 border border-primary-300 px-4 py-2 rounded-lg hover:bg-primary-50 transition-all duration-200 cursor-pointer',
    // 卡片样式
    'card': 'bg-white rounded-xl shadow-sm p-4 transition-shadow duration-200',
    'card-hover': 'bg-white rounded-xl shadow-sm p-4 hover:shadow-md transition-all duration-200 cursor-pointer',
    // 输入框样式
    'input-base': 'border border-gray-200 rounded-lg px-3 py-2 focus:border-primary-400 focus:ring-2 focus:ring-primary-100 focus:outline-none w-full transition-all duration-200',
    // 页面容器
    'page-container': 'mx-auto max-w-6xl p-6',
    // 渐变背景
    'bg-gradient-soft': 'bg-gradient-to-br from-primary-50 via-white to-accent-50',
  },
})
