# Another Yutto GUI

基于 Tauri 2 + Vue 3 的 Bilibili 视频下载器图形界面，为 [Yutto](https://github.com/yutto-dev/yutto) 提供GUI。

![yuttogui](./public/preview.png)

## Wayland

如果你在Wayland环境下，需要配置

```bash
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=1
```

## TODO
-  [ ] 识别不同视频链接
-  [x] 下载队列暂停、取消
-  [ ] 视频预览字幕、弹幕显示
-  [ ] 下载进度优化
-  [ ] 评论下载优化
-  [ ] 内置yutto ffmpeg (吗？)
-  [ ] Web端 (吗？)

## 致谢

- [Yutto](https://github.com/yutto-dev/yutto) - 一个可爱且任性的 B 站视频下载器
- [Tauri](https://tauri.app/) - Tauri 2.0 创建小型、快速、安全、跨平台的应用程序
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架

## 免责声明
本软件提供的所有内容，仅可用作学习交流使用，禁止用于其他用途。请在下载24小时内删除。为尊重作者版权，请前往资源的原始发布网站观看，支持原创，谢谢。
