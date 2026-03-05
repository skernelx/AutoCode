<div align="center">
  <h1>AutoCode</h1>
  <p><strong>macOS 2FA Code Assistant · 验证码自动提取助手</strong></p>
  <p>
    <img alt="platform" src="https://img.shields.io/badge/platform-macOS%2012%2B-0f172a?style=flat-square">
    <img alt="stack" src="https://img.shields.io/badge/stack-Tauri%20v2%20%2B%20Rust%20%2B%20Vanilla%20JS-2563eb?style=flat-square">
    <img alt="license" src="https://img.shields.io/badge/license-MIT-16a34a?style=flat-square">
  </p>
</div>

> [!IMPORTANT]
> 本项目是参考 [MessAuto](https://github.com/LeeeSe/MessAuto) 思路后进行的重写实现（不是 fork）。  
> This project is a rewritten implementation inspired by [MessAuto](https://github.com/LeeeSe/MessAuto), not a fork.

## 目录 | Table of Contents

- [项目简介 | Overview](#项目简介--overview)
- [核心功能 | Features](#核心功能--features)
- [快速开始 | Quick Start](#快速开始--quick-start)
- [权限说明 | Permissions](#权限说明--permissions)
- [配置说明 | Configuration](#配置说明--configuration)
- [项目结构 | Project Structure](#项目结构--project-structure)
- [常见问题 | FAQ](#常见问题--faq)
- [致谢 | Acknowledgement](#致谢--acknowledgement)
- [许可证 | License](#许可证--license)

## 项目简介 | Overview

AutoCode 是一个面向 macOS 的验证码助手，自动监听 iMessage、Apple Mail、Spotlight 邮件源（含 Outlook）中的新消息并提取验证码，然后按你的策略自动输入或复制到剪贴板。  
AutoCode is a macOS desktop app that monitors incoming messages/emails from iMessage, Apple Mail, and Spotlight-based mail sources (including Outlook), then auto-types or copies verification codes based on your settings.

## 核心功能 | Features

| 中文 | English |
| --- | --- |
| 多来源监听：iMessage / Apple Mail / Spotlight 邮件（含 Outlook） | Multi-source monitoring: iMessage / Apple Mail / Spotlight mail sources (including Outlook) |
| 多策略提取：模板正则、发件人白名单、HTML 结构、关键词近邻 | Multi-strategy extraction: regex templates, sender whitelist, HTML structure, keyword proximity |
| 粘贴模式：`smart` / `always` / `floating_only` / `clipboard_only` | Paste modes: `smart` / `always` / `floating_only` / `clipboard_only` |
| 前端设置修改后，托盘勾选状态即时同步 | Tray check states sync immediately after settings are changed in UI |
| 支持自动回车、开机自启、规则自定义 | Supports auto-enter, launch-at-login, and custom extraction rules |

## 快速开始 | Quick Start

### 运行环境 | Requirements

- macOS 12+
- Node.js 18+
- Rust stable (`rustup`)
- Xcode Command Line Tools (`xcode-select --install`)

### 本地开发 | Development

```bash
npm install
npm run tauri dev
```

### 生产构建 | Build

```bash
npm install
npm run tauri build
```

构建产物通常位于 / Build artifacts are usually under:

- `src-tauri/target/release/bundle/`

## 权限说明 | Permissions

AutoCode 在 macOS 上需要以下权限：

1. `完全磁盘访问 (Full Disk Access)`
   - 用于读取 iMessage 数据库和 Apple Mail 文件
2. `辅助功能 (Accessibility)`
   - 用于模拟键盘输入验证码、自动回车

如果权限未授予，应用仍可运行，但会降级为部分功能不可用。  
Without these permissions, the app still runs but some features are degraded.

## 配置说明 | Configuration

配置文件路径 / Config file path:

- `dirs::config_dir()/autocode/config.toml`
- macOS 常见路径 / Typical macOS path: `~/Library/Application Support/autocode/config.toml`

### 关键配置项 | Key Fields

| Field | Default | 说明 (CN) | Description (EN) |
| --- | --- | --- | --- |
| `listen_imessage` | `true` | 是否监听 iMessage | Enable iMessage monitor |
| `listen_apple_mail` | `true` | 是否监听 Apple Mail | Enable Apple Mail monitor |
| `listen_outlook` | `true` | 是否监听 Spotlight 邮件源（含 Outlook） | Enable Spotlight mail monitor (including Outlook) |
| `paste_mode` | `smart` | 粘贴策略模式 | Paste behavior mode |
| `auto_enter` | `false` | 自动输入后回车 | Press Enter after auto-typing |
| `launch_at_login` | `false` | 开机自启 | Launch at login |
| `autofill_detect_delay_ms` | `1500` | Smart 模式等待时长 | Smart mode delay before fallback |
| `verification_keywords` | built-in | 关键词提取列表 | Keyword extraction list |
| `verification_patterns` | built-in | 模板正则列表 | Template regex list |
| `known_2fa_senders` | built-in | 发件人白名单 | Trusted 2FA sender list |
| `native_autofill_apps` | built-in | 原生 AutoFill 应用白名单 | Native AutoFill app whitelist |

### `paste_mode` 行为 | Mode Behavior

- `smart`:
  - iMessage 来源下遇到原生 AutoFill 应用时，先延迟检测，仍冲突则改为复制
  - For iMessage in native AutoFill apps, delay and fallback to copy to avoid conflicts
- `always`:
  - 总是尝试自动输入，失败则保底复制
  - Always auto-type; fallback to copy on failure
- `floating_only` / `clipboard_only`:
  - 不自动输入，仅通知并复制
  - No auto-typing; notify/copy only

## 项目结构 | Project Structure

```text
AutoCode/
├─ src/                    # Frontend (Vanilla HTML/CSS/JS)
├─ src-tauri/
│  ├─ src/
│  │  ├─ monitor/          # iMessage / Apple Mail / Spotlight mail monitors
│  │  ├─ extractor.rs      # Multi-strategy code extraction
│  │  ├─ paste.rs          # Auto-typing and conflict handling
│  │  ├─ permissions.rs    # Permission checks and settings shortcuts
│  │  ├─ autostart.rs      # LaunchAgent startup integration
│  │  └─ lib.rs            # Tauri commands, tray, runtime wiring
│  ├─ tauri.conf.json
│  └─ Cargo.toml
├─ README.md
└─ LICENSE
```

## 常见问题 | FAQ

### iMessage / Apple Mail 没有识别到验证码

- 检查 `完全磁盘访问` 是否已授予
- 检查监控源开关是否开启（设置页或托盘）

### 识别成功但没自动输入

- 检查 `辅助功能` 权限
- 如果在 `smart` 模式，可能因为 AutoFill 冲突规避而回退为“仅复制”

### Spotlight 邮件识别不稳定

- 依赖 Spotlight 索引
- 先确认 `mdfind` 能在目标客户端数据目录检索到邮件

## 致谢 | Acknowledgement

- 感谢 [LeeeSe/MessAuto](https://github.com/LeeeSe/MessAuto) 提供灵感和思路。  
- 本项目为参考其方向后进行的重写实现，代码组织与实现细节已按本项目需求重新设计。  
- Thanks to [LeeeSe/MessAuto](https://github.com/LeeeSe/MessAuto) for inspiration. This project is a rewritten implementation tailored for this codebase.

## 许可证 | License

本项目采用 [MIT License](./LICENSE)。  
This project is licensed under the [MIT License](./LICENSE).
