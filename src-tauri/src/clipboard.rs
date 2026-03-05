use anyhow::{Context, Result};
use enigo::{Enigo, Keyboard, Key, Settings, Direction};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// 将文本复制到剪贴板
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("启动 pbcopy 失败")?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(text.as_bytes())?;
    }

    child.wait()?;
    Ok(())
}

/// 模拟 Cmd+V 粘贴
#[allow(dead_code)]
pub fn paste_from_clipboard() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("创建 Enigo 实例失败: {}", e))?;

    // 短暂延时确保剪贴板就绪
    thread::sleep(Duration::from_millis(50));

    enigo.key(Key::Meta, Direction::Press)
        .map_err(|e| anyhow::anyhow!("按下 Meta 键失败: {}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| anyhow::anyhow!("点击 V 键失败: {}", e))?;
    enigo.key(Key::Meta, Direction::Release)
        .map_err(|e| anyhow::anyhow!("释放 Meta 键失败: {}", e))?;

    Ok(())
}

/// 直接键盘输入文本（不占用剪贴板）
pub fn type_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("创建 Enigo 实例失败: {}", e))?;

    enigo.text(text)
        .map_err(|e| anyhow::anyhow!("输入文本失败: {}", e))?;

    Ok(())
}

/// 模拟按下回车键
pub fn press_enter() -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| anyhow::anyhow!("创建 Enigo 实例失败: {}", e))?;

    thread::sleep(Duration::from_millis(100));

    enigo.key(Key::Return, Direction::Click)
        .map_err(|e| anyhow::anyhow!("按下回车键失败: {}", e))?;

    Ok(())
}

/// 自动粘贴验证码
///
/// `direct_input`: true = 直接键盘输入（不占用剪贴板），false = 通过剪贴板粘贴
#[allow(dead_code)]
pub fn auto_paste(direct_input: bool, text: &str) -> Result<()> {
    if direct_input {
        type_text(text)
    } else {
        copy_to_clipboard(text)?;
        paste_from_clipboard()
    }
}

/// 获取当前前台应用的 Bundle ID
pub fn get_frontmost_app_bundle_id() -> Option<String> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get bundle identifier of first process whose frontmost is true",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !bundle_id.is_empty() {
            return Some(bundle_id);
        }
    }
    None
}
