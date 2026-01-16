# Another Yutto GUI

基于 Tauri 2 + Vue 3 的 Bilibili 视频下载器图形界面，为 [Yutto](https://github.com/yutto-dev/yutto) 提供GUI。

## 特性

- **视频下载** - 支持 Bilibili 视频下载，包含多种清晰度选项（8K/4K/1080P/720P 等）
- **音频质量** - 支持 Hi-Res、杜比全景声、320kbps 等多种音频质量
- **下载队列** - 智能队列管理，支持并发下载（可配置 1-10 个任务）
- **任务控制** - 支持暂停、恢复、取消下载任务
- **历史记录** - 自动记录下载历史，方便查看和管理
- **预设配置** - 保存常用下载配置，一键应用
- **弹幕字幕** - 可选下载弹幕、字幕和封面图
- **用户认证** - 支持 SESSDATA 登录，下载会员专享内容
- **跨平台** - 支持 Windows、macOS 和 Linux

## 技术栈

### 前端
- **Vue 3** - 使用 Composition API 和 TypeScript
- **Pinia** - 状态管理，支持持久化
- **Vue Router** - 路由管理
- **UnoCSS** - 原子化 CSS 框架
- **Vite** - 构建工具

### 后端
- **Rust** - 高性能后端
- **Tauri 2** - 跨平台桌面应用框架
- **Tokio** - 异步运行时
- **SQLite** - 本地数据存储

## 前置要求

- **Node.js** >= 18
- **Rust** >= 1.70
- **Yutto CLI** - 需要安装 [Yutto](https://github.com/yutto-dev/yutto) 命令行工具

## 安装

```bash
# 克隆项目
git clone <repository-url>
cd yutto-gui

# 安装依赖
npm install
```

## 开发

```bash
# 启动开发服务器
npm run dev

# 代码检查
npm run lint

# 自动修复代码问题
npm run lint:fix
```

开发服务器将在 `http://localhost:1420` 启动，支持热更新。

## 构建

```bash
# 构建生产版本
npm run build

# 预览构建结果
npm run preview
```

构建完成后，可执行文件将在 `src-tauri/target/release` 目录中生成。

## 项目结构

```
yutto-gui/
├── src/                    # 前端源码 (Vue 3 + TypeScript)
│   ├── components/         # 组件
│   ├── views/             # 页面
│   ├── stores/            # Pinia 状态管理
│   ├── services/          # API 服务
│   ├── composables/       # 组合式函数
│   ├── types/             # TypeScript 类型定义
│   └── router/            # 路由配置
├── src-tauri/             # 后端源码 (Rust)
│   ├── src/
│   │   ├── commands/      # Tauri 命令处理
│   │   ├── services/      # 业务逻辑
│   │   └── models/        # 数据模型
│   └── Cargo.toml         # Rust 依赖
├── package.json           # Node.js 依赖
└── vite.config.ts         # Vite 配置
```

## 使用说明

1. **配置 Yutto 路径** - 在设置页面配置 Yutto CLI 工具的路径
2. **设置下载目录** - 选择视频保存位置
3. **输入视频链接** - 支持 BV 号或完整 URL
4. **选择质量** - 根据需要选择视频和音频质量
5. **开始下载** - 任务将自动加入队列并开始下载

## 配置

### SESSDATA 登录

要下载会员专享内容，需要配置 SESSDATA Cookie：

1. 登录 Bilibili 网站
2. 打开浏览器开发者工具 (F12)
3. 在 Application/Storage > Cookies 中找到 `SESSDATA`
4. 复制其值并在应用设置中配置

## 开发计划

- [ ] 主题切换（深色模式）
- [ ] 多语言支持（英文）
- [ ] 批量下载优化
- [ ] 下载速度限制
- [ ] 自动更新功能

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 致谢

- [Yutto](https://github.com/yutto-dev/yutto) - 强大的 Bilibili 下载工具
- [Tauri](https://tauri.app/) - 现代化的桌面应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
