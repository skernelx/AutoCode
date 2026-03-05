<div align="center">

<img src="./icon.png" width="128" height="128" alt="AutoCode Icon">

# AutoCode

**Mac 上自动接收 iPhone 验证码 · 告别频繁掏手机**

[![macOS](https://img.shields.io/badge/macOS-12.0+-000000?style=for-the-badge&logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](./LICENSE)

<p align="center">
  <img src="https://img.shields.io/github/stars/skernelx/AutoCode?style=social" alt="GitHub stars">
  <img src="https://img.shields.io/github/forks/skernelx/AutoCode?style=social" alt="GitHub forks">
</p>

[English](#english) | [中文](#中文)

</div>

---

## 中文

### 💡 解决什么问题？

你是否经常遇到这样的场景：

- 📱 在 Mac 上登录网站，需要输入验证码
- 🔍 掏出 iPhone 查看短信或邮件
- ⌨️ 记住验证码，切回 Mac 手动输入
- 😫 验证码太长记不住，来回切换多次

**AutoCode 让这一切自动化！**

当你的 iPhone 收到验证码短信或邮件时，AutoCode 会：
1. ✅ 自动在 Mac 上检测到（通过 iCloud 同步的 iMessage 和邮件）
2. ✅ 智能提取验证码
3. ✅ 自动输入到当前输入框，或复制到剪贴板
4. ✅ 无需掏出手机，无需手动输入

### ✨ 核心特性

AutoCode 是一款专为 macOS 设计的智能验证码助手，利用苹果生态系统的无缝连接，让你在 Mac 上自动接收 iPhone 的验证码。

#### 🎯 核心功能

- **🔍 多源监听**
  - iMessage 短信验证码
  - Apple Mail 邮件验证码
  - Spotlight 邮件源（支持 Outlook、Gmail 等第三方邮件客户端）

- **🧠 智能提取**
  - 模板正则匹配（高精度）
  - 发件人白名单识别
  - HTML 结构解析
  - 关键词近邻搜索
  - 多策略组合，置信度评分

- **⚡️ 灵活粘贴**
  - `Smart` 模式：智能避免与系统 AutoFill 冲突
  - `Always` 模式：总是自动输入
  - `Floating Only` 模式：仅显示悬浮窗
  - `Clipboard Only` 模式：仅复制到剪贴板

- **🎨 用户体验**
  - 系统托盘常驻，快速访问
  - 实时同步配置状态
  - 验证码历史记录
  - 支持自动回车
  - 开机自启动

### 📦 安装

#### 下载预编译版本

前往 [Releases](https://github.com/skernelx/AutoCode/releases) 页面下载最新版本的 `.dmg` 文件。

#### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/skernelx/AutoCode.git
cd AutoCode

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

**环境要求：**
- macOS 12.0+
- Node.js 18+
- Rust 1.70+ (通过 `rustup` 安装)
- Xcode Command Line Tools (`xcode-select --install`)

### 🔑 权限设置

AutoCode 需要以下系统权限才能正常工作：

#### 1. 完全磁盘访问 (Full Disk Access)
用于读取 iMessage 数据库和 Apple Mail 邮件文件。

**设置路径：** 系统设置 → 隐私与安全性 → 完全磁盘访问

#### 2. 辅助功能 (Accessibility)
用于模拟键盘输入验证码和自动回车。

**设置路径：** 系统设置 → 隐私与安全性 → 辅助功能

> 💡 **提示：** 应用内置权限检测和快捷跳转功能，首次启动时会自动引导你完成权限设置。

### ⚙️ 配置

配置文件位于：`~/Library/Application Support/autocode/config.toml`

#### 主要配置项

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `listen_imessage` | `true` | 是否监听 iMessage |
| `listen_apple_mail` | `true` | 是否监听 Apple Mail |
| `listen_outlook` | `true` | 是否监听 Spotlight 邮件源 |
| `paste_mode` | `smart` | 粘贴模式 |
| `auto_enter` | `false` | 自动输入后按回车 |
| `launch_at_login` | `false` | 开机自启动 |
| `autofill_detect_delay_ms` | `1500` | Smart 模式延迟检测时间 |

#### 粘贴模式说明

- **Smart（推荐）**：在 Safari 等原生支持 AutoFill 的应用中，延迟检测避免冲突；其他应用直接自动输入
- **Always**：总是尝试自动输入，失败则复制到剪贴板
- **Floating Only**：仅显示悬浮通知，不自动输入
- **Clipboard Only**：仅复制到剪贴板，不显示通知

### 🎨 自定义规则

你可以在设置页面自定义：

- **验证码关键词**：用于关键词近邻搜索
- **正则表达式模板**：用于模板匹配
- **可信发件人列表**：白名单发件人地址
- **原生 AutoFill 应用**：需要避免冲突的应用 Bundle ID

### 🛠️ 技术架构

```
AutoCode/
├── src/                      # 前端 (Vanilla JS)
│   ├── index.html           # 主页面
│   ├── main.js              # 业务逻辑
│   └── styles.css           # 样式
└── src-tauri/               # 后端 (Rust)
    ├── src/
    │   ├── monitor/         # 监控模块
    │   │   ├── imessage.rs  # iMessage 监控
    │   │   ├── apple_mail.rs # Apple Mail 监控
    │   │   └── outlook.rs   # Spotlight 邮件监控
    │   ├── extractor.rs     # 验证码提取引擎
    │   ├── paste.rs         # 粘贴策略处理
    │   ├── config.rs        # 配置管理
    │   ├── permissions.rs   # 权限检测
    │   ├── clipboard.rs     # 剪贴板操作
    │   ├── autostart.rs     # 开机自启
    │   └── lib.rs           # 主入口
    └── Cargo.toml
```

**核心技术：**
- **Tauri 2.0**：轻量级桌面应用框架
- **Rust**：高性能后端逻辑
- **Tokio**：异步运行时
- **Regex**：正则表达式引擎
- **SQLite**：iMessage 数据库读取

### 🐛 常见问题

<details>
<summary><b>Q: 为什么没有识别到验证码？</b></summary>

1. 检查是否授予了"完全磁盘访问"权限
2. 确认对应的监控源已开启（设置页面或托盘菜单）
3. 查看日志文件：`~/Library/Application Support/autocode/logs/`
</details>

<details>
<summary><b>Q: 识别成功但没有自动输入？</b></summary>

1. 检查是否授予了"辅助功能"权限
2. 如果使用 Smart 模式，可能因为避免冲突而降级为仅复制
3. 尝试切换到 Always 模式测试
</details>

<details>
<summary><b>Q: Outlook 邮件识别不稳定？</b></summary>

Spotlight 邮件监控依赖 macOS 的 Spotlight 索引。如果识别不稳定：
1. 确保 Spotlight 索引正常工作
2. 在终端运行 `mdfind -onlyin ~/Library/Group\ Containers/ kind:email` 测试
3. 考虑使用 Apple Mail 作为主要邮件客户端
</details>

<details>
<summary><b>Q: 如何添加自定义验证码规则？</b></summary>

在设置页面的"高级设置"中：
1. 添加关键词：如"验证码"、"OTP"等
2. 添加正则表达式：如 `\b(\d{6})\b` 匹配 6 位数字
3. 添加可信发件人：如 `noreply@example.com`
</details>

### 🔒 隐私与安全

- ✅ 所有数据本地处理，不上传任何信息
- ✅ 仅读取必要的数据库和文件
- ✅ 开源代码，可审计
- ✅ 不收集任何用户数据

### 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 📝 更新日志

查看 [CHANGELOG.md](./CHANGELOG.md) 了解版本更新历史。

### 🙏 致谢

- 灵感来源：[MessAuto](https://github.com/LeeeSe/MessAuto)
- 本项目是完全重写的实现，代码架构和实现细节均为原创

### 📄 许可证

本项目采用 [MIT License](./LICENSE) 开源协议。

---

## English

### 💡 What Problem Does It Solve?

Have you ever experienced this scenario:

- 📱 Logging into a website on your Mac, need to enter a verification code
- 🔍 Pull out your iPhone to check SMS or email
- ⌨️ Memorize the code, switch back to Mac and type it manually
- 😫 Code too long to remember, switch back and forth multiple times

**AutoCode automates all of this!**

When your iPhone receives a verification code via SMS or email, AutoCode will:
1. ✅ Automatically detect it on your Mac (via iCloud-synced iMessage and Mail)
2. ✅ Intelligently extract the verification code
3. ✅ Auto-type it into the current input field, or copy to clipboard
4. ✅ No need to pull out your phone, no manual typing required

### ✨ Core Features

AutoCode is an intelligent verification code assistant designed for macOS that leverages the seamless integration of Apple's ecosystem to automatically receive iPhone verification codes on your Mac.

#### 🎯 Core Features

- **🔍 Multi-Source Monitoring**
  - iMessage SMS verification codes
  - Apple Mail email verification codes
  - Spotlight mail sources (supports Outlook, Gmail, and other third-party email clients)

- **🧠 Smart Extraction**
  - Template regex matching (high precision)
  - Sender whitelist recognition
  - HTML structure parsing
  - Keyword proximity search
  - Multi-strategy combination with confidence scoring

- **⚡️ Flexible Pasting**
  - `Smart` mode: Intelligently avoids conflicts with system AutoFill
  - `Always` mode: Always auto-types
  - `Floating Only` mode: Shows floating window only
  - `Clipboard Only` mode: Copies to clipboard only

- **🎨 User Experience**
  - System tray resident for quick access
  - Real-time configuration sync
  - Verification code history
  - Auto-enter support
  - Launch at login

### 📦 Installation

#### Download Pre-built Version

Visit the [Releases](https://github.com/skernelx/AutoCode/releases) page to download the latest `.dmg` file.

#### Build from Source

```bash
# Clone repository
git clone https://github.com/skernelx/AutoCode.git
cd AutoCode

# Install dependencies
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

**Requirements:**
- macOS 12.0+
- Node.js 18+
- Rust 1.70+ (install via `rustup`)
- Xcode Command Line Tools (`xcode-select --install`)

### 🔑 Permission Setup

AutoCode requires the following system permissions:

#### 1. Full Disk Access
Required to read iMessage database and Apple Mail files.

**Path:** System Settings → Privacy & Security → Full Disk Access

#### 2. Accessibility
Required to simulate keyboard input for verification codes and auto-enter.

**Path:** System Settings → Privacy & Security → Accessibility

> 💡 **Tip:** The app includes built-in permission detection and quick navigation. It will guide you through the setup on first launch.

### ⚙️ Configuration

Configuration file location: `~/Library/Application Support/autocode/config.toml`

#### Main Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `listen_imessage` | `true` | Monitor iMessage |
| `listen_apple_mail` | `true` | Monitor Apple Mail |
| `listen_outlook` | `true` | Monitor Spotlight mail sources |
| `paste_mode` | `smart` | Paste mode |
| `auto_enter` | `false` | Press Enter after auto-typing |
| `launch_at_login` | `false` | Launch at login |
| `autofill_detect_delay_ms` | `1500` | Smart mode detection delay |

#### Paste Mode Explanation

- **Smart (Recommended)**: In apps with native AutoFill support (like Safari), delays detection to avoid conflicts; auto-types directly in other apps
- **Always**: Always attempts to auto-type, falls back to clipboard on failure
- **Floating Only**: Shows floating notification only, no auto-typing
- **Clipboard Only**: Copies to clipboard only, no notification

### 🛠️ Technical Architecture

Built with modern technologies:
- **Tauri 2.0**: Lightweight desktop application framework
- **Rust**: High-performance backend logic
- **Tokio**: Async runtime
- **Regex**: Regular expression engine
- **SQLite**: iMessage database access

### 🐛 FAQ

<details>
<summary><b>Q: Why aren't verification codes being detected?</b></summary>

1. Check if "Full Disk Access" permission is granted
2. Ensure the corresponding monitoring source is enabled (Settings page or tray menu)
3. Check log files: `~/Library/Application Support/autocode/logs/`
</details>

<details>
<summary><b>Q: Codes detected but not auto-typed?</b></summary>

1. Check if "Accessibility" permission is granted
2. If using Smart mode, it may have downgraded to copy-only to avoid conflicts
3. Try switching to Always mode for testing
</details>

<details>
<summary><b>Q: Outlook email detection unstable?</b></summary>

Spotlight mail monitoring relies on macOS Spotlight indexing. If detection is unstable:
1. Ensure Spotlight indexing is working properly
2. Test in Terminal: `mdfind -onlyin ~/Library/Group\ Containers/ kind:email`
3. Consider using Apple Mail as your primary email client
</details>

### 🔒 Privacy & Security

- ✅ All data processed locally, no uploads
- ✅ Only reads necessary databases and files
- ✅ Open source code, auditable
- ✅ No user data collection

### 🤝 Contributing

Issues and Pull Requests are welcome!

1. Fork this repository
2. Create feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to branch (`git push origin feature/AmazingFeature`)
5. Open Pull Request

### 🙏 Acknowledgements

- Inspired by: [MessAuto](https://github.com/LeeeSe/MessAuto)
- This project is a complete rewrite with original architecture and implementation

### 📄 License

This project is licensed under the [MIT License](./LICENSE).

---

<div align="center">
  <p>Made with ❤️ for macOS users</p>
  <p>If you find this project helpful, please consider giving it a ⭐️</p>
</div>
