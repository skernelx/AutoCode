use anyhow::Result;
use serde::Serialize;

/// 权限检查结果
#[derive(Debug, Clone, Serialize)]
pub struct PermissionStatus {
    /// 完全磁盘访问权限（读取 iMessage 数据库、Apple Mail 邮件）
    pub full_disk_access: bool,
    /// 辅助功能权限（模拟键盘粘贴）
    pub accessibility: bool,
}

/// 检查全部权限状态
pub fn check_all() -> PermissionStatus {
    PermissionStatus {
        full_disk_access: check_full_disk_access(),
        accessibility: check_accessibility(),
    }
}

/// 检查「完全磁盘访问」权限
/// 原理：尝试读取受 TCC 保护的 iMessage 数据库文件
fn check_full_disk_access() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    // chat.db 是受 TCC 保护的典型文件
    let chat_db = home.join("Library/Messages/chat.db");
    if chat_db.exists() {
        // 尝试 open() — 如果 FDA 未授权，会返回 PermissionDenied
        match std::fs::File::open(&chat_db) {
            Ok(_) => return true,
            Err(e) => {
                log::debug!("FDA check: chat.db open failed: {}", e);
            }
        }
    }

    // 备选：检查 Safari 历史数据库（同样受 TCC 保护）
    let safari_db = home.join("Library/Safari/History.db");
    if safari_db.exists() {
        match std::fs::File::open(&safari_db) {
            Ok(_) => return true,
            Err(_) => {}
        }
    }

    false
}

/// 检查「辅助功能」权限
/// 使用 macOS 的 AXIsProcessTrusted() API
fn check_accessibility() -> bool {
    // 通过 FFI 调用 ApplicationServices 框架
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }

    unsafe { AXIsProcessTrusted() }
}

/// 打开系统偏好设置 — 完全磁盘访问
pub fn open_full_disk_access_settings() -> Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        .spawn()?;
    Ok(())
}

/// 打开系统偏好设置 — 辅助功能
pub fn open_accessibility_settings() -> Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()?;
    Ok(())
}

/// 请求辅助功能权限（弹出系统授权对话框）
/// 调用 AXIsProcessTrustedWithOptions，设置 kAXTrustedCheckOptionPrompt = true
#[allow(dead_code)]
pub fn request_accessibility() -> bool {
    use std::process::Command;
    // 使用 osascript 触发系统权限弹框
    // 这比 FFI 调 AXIsProcessTrustedWithOptions 更简单可靠
    let result = Command::new("osascript")
        .arg("-e")
        .arg(r##"tell application "System Events" to keystroke """##)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                return true;
            }
            // 如果失败，系统会自动弹出授权对话框
            false
        }
        Err(_) => false,
    }
}
