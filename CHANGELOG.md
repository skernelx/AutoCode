# Changelog

All notable changes to AutoCode will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-03-05

### ✨ Added
- **自动更新功能**：启动时自动检查 GitHub 更新，支持一键下载安装
- **后台运行支持**：关闭窗口只隐藏程序，不退出，保持后台监控
- **安装说明优化**：添加 xattr 命令移除隔离属性的方法

### 🔧 Fixed
- **窗口关闭行为**：点击关闭按钮不再退出程序，只有从托盘菜单选择退出才真正退出
- **程序持久运行**：修复程序自动退出的问题，现在会一直在后台运行

## [0.2.1] - 2026-03-05

### 🎨 Changed
- **图标优化**：使用全新设计的程序图标和托盘图标
- **README 优化**：
  - 突出核心价值：Mac 自动接收 iPhone 验证码
  - 说明相比 macOS 自带功能的优势（支持所有应用）
  - 添加程序图标展示
  - 补充详细的适用场景说明（浏览器、桌面应用、游戏客户端等）

## [0.2.0] - 2026-03-05

### 🎨 Changed
- **全新图标设计**：更新了应用图标和系统托盘图标
- **重写 README**：提供更专业、更详细的项目文档

### 🔧 Fixed
- **错误处理改进**：将所有 `unwrap()` 调用改为安全的错误处理
- **配置验证**：添加配置文件验证逻辑，防止无效配置导致崩溃
- **正则表达式优化**：预编译常用正则表达式，提升性能
- **任务取消机制**：使用 `CancellationToken` 替代 `abort()`，实现优雅的任务取消
- **前端错误处理**：添加 clipboard API 降级方案，提高兼容性

### 🚀 Improved
- **性能优化**：正则表达式缓存机制，减少重复编译
- **代码质量**：移除所有潜在的 panic 点
- **用户体验**：更好的错误提示和降级处理

### 📦 Dependencies
- 添加 `tokio-util` 0.7 用于任务取消

## [0.1.1] - 2026-03-04

### 🎉 Added
- 初始版本发布
- iMessage 验证码监控
- Apple Mail 验证码监控
- Spotlight 邮件源监控（支持 Outlook）
- 多策略验证码提取
- 智能粘贴模式
- 系统托盘集成
- 配置管理界面

### ✨ Features
- 模板正则匹配
- 发件人白名单
- HTML 结构解析
- 关键词近邻搜索
- 自动回车支持
- 开机自启动
- 验证码历史记录

---

[0.2.0]: https://github.com/skernelx/AutoCode/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/skernelx/AutoCode/releases/tag/v0.1.1
