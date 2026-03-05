use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const PLIST_LABEL: &str = "com.autocode.app";

/// 获取 LaunchAgent plist 文件路径
fn plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法获取用户主目录")?;
    let dir = home.join("Library/LaunchAgents");
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{}.plist", PLIST_LABEL)))
}

/// 获取当前应用的可执行文件路径
fn app_executable() -> Result<PathBuf> {
    // 优先使用 .app bundle 中的路径
    let exe = std::env::current_exe().context("无法获取当前可执行文件路径")?;

    // 如果在 .app bundle 内运行，exe 路径形如:
    // /Applications/AutoCode.app/Contents/MacOS/autocode
    // 直接使用即可
    Ok(exe)
}

/// 生成 LaunchAgent plist 内容
fn generate_plist(exe_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>"#,
        label = PLIST_LABEL,
        exe = exe_path,
    )
}

/// 注册开机自启（创建 LaunchAgent plist）
pub fn enable() -> Result<()> {
    let path = plist_path()?;
    let exe = app_executable()?;
    let exe_str = exe.to_string_lossy();

    let content = generate_plist(&exe_str);
    fs::write(&path, &content)
        .with_context(|| format!("写入 LaunchAgent 失败: {:?}", path))?;

    log::info!("已注册开机自启: {:?}", path);
    Ok(())
}

/// 取消开机自启（删除 LaunchAgent plist）
pub fn disable() -> Result<()> {
    let path = plist_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("删除 LaunchAgent 失败: {:?}", path))?;
        log::info!("已取消开机自启: {:?}", path);
    }
    Ok(())
}

/// 检查是否已注册开机自启
pub fn is_enabled() -> bool {
    match plist_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

/// 根据 bool 状态切换自启
pub fn set_enabled(enabled: bool) -> Result<()> {
    if enabled {
        enable()
    } else {
        disable()
    }
}
