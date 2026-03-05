mod autostart;
mod clipboard;
mod config;
mod extractor;
mod monitor;
mod paste;
mod permissions;

use config::{load_shared_config, AppConfig, SharedConfig};
use extractor::CodeExtractor;
use monitor::MonitorCommand;
use std::sync::Arc;
use tauri::{Emitter, Manager};

/// 全局监控控制器（用于动态启停监控源）
struct MonitorController {
    cmd_tx: monitor::CommandSender,
}

/// 托盘可勾选菜单项句柄
struct TrayMenuState {
    listen_imessage: tauri::menu::CheckMenuItem<tauri::Wry>,
    listen_apple_mail: tauri::menu::CheckMenuItem<tauri::Wry>,
    listen_outlook: tauri::menu::CheckMenuItem<tauri::Wry>,
    auto_enter: tauri::menu::CheckMenuItem<tauri::Wry>,
}

/// Tauri 命令：获取当前配置
#[tauri::command]
fn get_config(state: tauri::State<SharedConfig>) -> Result<AppConfig, String> {
    let config = state.read().map_err(|e| format!("读取配置失败: {}", e))?;
    Ok(config.clone())
}

/// Tauri 命令：更新配置
#[tauri::command]
fn update_config(
    state: tauri::State<SharedConfig>,
    monitor: tauri::State<MonitorController>,
    tray: tauri::State<TrayMenuState>,
    new_config: AppConfig,
) -> Result<(), String> {
    let old_config = {
        let cfg = state.read().map_err(|e| format!("读取配置失败: {}", e))?;
        cfg.clone()
    };

    // 同步开机自启状态
    if let Err(e) = autostart::set_enabled(new_config.launch_at_login) {
        log::warn!("设置开机自启失败: {}", e);
    }

    new_config
        .save()
        .map_err(|e| format!("保存配置失败: {}", e))?;
    {
        let mut config = state.write().map_err(|e| format!("写入配置失败: {}", e))?;
        *config = new_config.clone();
    }

    // 实时同步监控源启停（无需重启）
    sync_monitor_sources(&monitor.cmd_tx, &old_config, &new_config);
    // 同步托盘勾选状态（前端改配置后立即刷新）
    sync_tray_menu_state(&tray, &new_config);
    Ok(())
}

/// Tauri 命令：获取默认配置
#[tauri::command]
fn get_default_config() -> AppConfig {
    AppConfig::default()
}

/// Tauri 命令：获取版本信息
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Tauri 命令：检查权限状态
#[tauri::command]
fn check_permissions() -> permissions::PermissionStatus {
    permissions::check_all()
}

/// Tauri 命令：打开完全磁盘访问设置
#[tauri::command]
fn open_fda_settings() -> Result<(), String> {
    permissions::open_full_disk_access_settings().map_err(|e| format!("打开设置失败: {}", e))
}

/// Tauri 命令：打开辅助功能设置
#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    permissions::open_accessibility_settings().map_err(|e| format!("打开设置失败: {}", e))
}

/// 显示/创建主窗口
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 显示主窗口并切换到设置页
fn show_settings(app: &tauri::AppHandle) {
    show_main_window(app);
    let _ = app.emit("navigate", "settings");
}

/// 发送监控命令（非阻塞）
fn send_monitor_command(cmd_tx: &monitor::CommandSender, cmd: MonitorCommand) {
    let cmd_name = match &cmd {
        MonitorCommand::StartImessage => "StartImessage",
        MonitorCommand::StopImessage => "StopImessage",
        MonitorCommand::StartAppleMail => "StartAppleMail",
        MonitorCommand::StopAppleMail => "StopAppleMail",
        MonitorCommand::StartOutlook => "StartOutlook",
        MonitorCommand::StopOutlook => "StopOutlook",
        MonitorCommand::Shutdown => "Shutdown",
    };

    if let Err(e) = cmd_tx.try_send(cmd) {
        log::warn!("发送监控命令 {} 失败: {}", cmd_name, e);
    }
}

/// 根据配置差异同步监控源
fn sync_monitor_sources(cmd_tx: &monitor::CommandSender, old: &AppConfig, new: &AppConfig) {
    if old.listen_imessage != new.listen_imessage {
        if new.listen_imessage {
            send_monitor_command(cmd_tx, MonitorCommand::StartImessage);
        } else {
            send_monitor_command(cmd_tx, MonitorCommand::StopImessage);
        }
    }

    if old.listen_apple_mail != new.listen_apple_mail {
        if new.listen_apple_mail {
            send_monitor_command(cmd_tx, MonitorCommand::StartAppleMail);
        } else {
            send_monitor_command(cmd_tx, MonitorCommand::StopAppleMail);
        }
    }

    if old.listen_outlook != new.listen_outlook {
        if new.listen_outlook {
            send_monitor_command(cmd_tx, MonitorCommand::StartOutlook);
        } else {
            send_monitor_command(cmd_tx, MonitorCommand::StopOutlook);
        }
    }
}

/// 根据当前配置启动监控源（应用启动时）
fn start_enabled_sources(cmd_tx: &monitor::CommandSender, cfg: &AppConfig) {
    if cfg.listen_imessage {
        send_monitor_command(cmd_tx, MonitorCommand::StartImessage);
    }
    if cfg.listen_apple_mail {
        send_monitor_command(cmd_tx, MonitorCommand::StartAppleMail);
    }
    if cfg.listen_outlook {
        send_monitor_command(cmd_tx, MonitorCommand::StartOutlook);
    }
}

fn set_checked(item: &tauri::menu::CheckMenuItem<tauri::Wry>, checked: bool, name: &str) {
    if let Err(e) = item.set_checked(checked) {
        log::warn!("更新托盘菜单 '{}' 失败: {}", name, e);
    }
}

fn sync_tray_menu_state(tray: &TrayMenuState, cfg: &AppConfig) {
    set_checked(
        &tray.listen_imessage,
        cfg.listen_imessage,
        "listen_imessage",
    );
    set_checked(
        &tray.listen_apple_mail,
        cfg.listen_apple_mail,
        "listen_apple_mail",
    );
    set_checked(&tray.listen_outlook, cfg.listen_outlook, "listen_outlook");
    set_checked(&tray.auto_enter, cfg.auto_enter, "auto_enter");
}

enum TraySource {
    Imessage,
    AppleMail,
    Outlook,
}

/// 托盘中切换监控源并持久化配置
fn toggle_source_from_tray(app: &tauri::AppHandle, source: TraySource) {
    let config_state = app.state::<SharedConfig>();
    let monitor = app.state::<MonitorController>();
    let tray = app.state::<TrayMenuState>();

    let (old_cfg, new_cfg, enabled) = {
        let mut cfg = match config_state.write() {
            Ok(c) => c,
            Err(e) => {
                log::error!("获取配置锁失败: {}", e);
                return;
            }
        };
        let old = cfg.clone();
        let enabled = match source {
            TraySource::Imessage => {
                cfg.listen_imessage = !cfg.listen_imessage;
                cfg.listen_imessage
            }
            TraySource::AppleMail => {
                cfg.listen_apple_mail = !cfg.listen_apple_mail;
                cfg.listen_apple_mail
            }
            TraySource::Outlook => {
                cfg.listen_outlook = !cfg.listen_outlook;
                cfg.listen_outlook
            }
        };
        let new = cfg.clone();
        (old, new, enabled)
    };

    sync_monitor_sources(&monitor.cmd_tx, &old_cfg, &new_cfg);
    sync_tray_menu_state(&tray, &new_cfg);
    if let Err(e) = new_cfg.save() {
        log::error!("保存配置失败: {}", e);
    }
    let source_name = match source {
        TraySource::Imessage => "iMessage",
        TraySource::AppleMail => "Apple Mail",
        TraySource::Outlook => "Spotlight 邮件",
    };
    log::info!("{} 监听状态: {}", source_name, enabled);
}

/// 托盘中切换自动回车并持久化配置
fn toggle_auto_enter_from_tray(app: &tauri::AppHandle) {
    let config_state = app.state::<SharedConfig>();
    let tray = app.state::<TrayMenuState>();
    let cfg = {
        let mut cfg = match config_state.write() {
            Ok(c) => c,
            Err(e) => {
                log::error!("获取配置锁失败: {}", e);
                return;
            }
        };
        cfg.auto_enter = !cfg.auto_enter;
        if let Err(e) = cfg.save() {
            log::error!("保存配置失败: {}", e);
        }
        cfg.clone()
    };
    sync_tray_menu_state(&tray, &cfg);
    log::info!("自动回车: {}", cfg.auto_enter);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("=== AutoCode v{} ===", env!("CARGO_PKG_VERSION"));

    // 加载配置
    let shared_config = match load_shared_config() {
        Ok(c) => c,
        Err(e) => {
            log::error!("加载配置失败: {}", e);
            Arc::new(std::sync::RwLock::new(AppConfig::default()))
        }
    };

    // 同步开机自启状态：以 LaunchAgent 是否存在为准
    {
        let mut cfg = match shared_config.write() {
            Ok(c) => c,
            Err(e) => {
                log::error!("获取配置锁失败: {}", e);
                Arc::new(std::sync::RwLock::new(AppConfig::default()))
                    .write()
                    .expect("创建默认配置失败")
            }
        };
        cfg.launch_at_login = autostart::is_enabled();
    }

    let config_for_setup = shared_config.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shared_config.clone())
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            get_default_config,
            get_version,
            check_permissions,
            open_fda_settings,
            open_accessibility_settings
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let config = config_for_setup.clone();
            let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);

            // 在后台启动 tokio 运行时和监控系统
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    start_monitoring(app_handle, config, ready_tx).await;
                });
            });

            // 等待监控系统就绪并注入全局状态
            let cmd_tx = ready_rx
                .recv()
                .map_err(|e| format!("监控系统初始化失败: {}", e))?;
            app.manage(MonitorController { cmd_tx });

            // 设置系统托盘（Tauri v2 方式）
            setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("AutoCode 启动失败");
}

/// 启动监控系统
async fn start_monitoring(
    app_handle: tauri::AppHandle,
    config: SharedConfig,
    ready_tx: std::sync::mpsc::SyncSender<monitor::CommandSender>,
) {
    let (cmd_tx, mut msg_rx) = monitor::start_monitor();
    let _ = ready_tx.send(cmd_tx.clone());

    // 读取配置并启动对应的监控源
    {
        let cfg = match config.read() {
            Ok(c) => c,
            Err(e) => {
                log::error!("读取配置失败: {}", e);
                return;
            }
        };
        start_enabled_sources(&cmd_tx, &cfg);
    }

    // 创建提取引擎
    let extractor = CodeExtractor::new();

    // 消息处理循环
    while let Some(msg) = msg_rx.recv().await {
        paste::handle_message(msg, &config, &extractor, &app_handle).await;
    }
}

/// 设置系统托盘
fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{
        menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
        tray::TrayIconBuilder,
    };

    let config = app.state::<SharedConfig>();
    let cfg = match config.read() {
        Ok(c) => c,
        Err(e) => {
            log::error!("读取配置失败: {}", e);
            return Err("读取配置失败".into());
        }
    };

    // 创建菜单项
    let listen_imessage = CheckMenuItem::with_id(
        app,
        "listen_imessage",
        "监听 iMessage",
        true,
        cfg.listen_imessage,
        None::<&str>,
    )?;
    let listen_apple_mail = CheckMenuItem::with_id(
        app,
        "listen_apple_mail",
        "监听 Apple Mail",
        true,
        cfg.listen_apple_mail,
        None::<&str>,
    )?;
    let listen_outlook = CheckMenuItem::with_id(
        app,
        "listen_outlook",
        "监听 Spotlight 邮件",
        true,
        cfg.listen_outlook,
        None::<&str>,
    )?;
    let auto_enter = CheckMenuItem::with_id(
        app,
        "auto_enter",
        "自动回车",
        true,
        cfg.auto_enter,
        None::<&str>,
    )?;

    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "设置...", true, None::<&str>)?;
    let about = MenuItem::with_id(
        app,
        "about",
        &format!("关于 AutoCode v{}", env!("CARGO_PKG_VERSION")),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &listen_imessage,
            &listen_apple_mail,
            &listen_outlook,
            &auto_enter,
            &separator,
            &settings,
            &about,
            &quit,
        ],
    )?;

    let icon_bytes = include_bytes!("../icons/tray-icon.png");
    let icon = tauri::image::Image::from_bytes(icon_bytes)?;

    app.manage(TrayMenuState {
        listen_imessage: listen_imessage.clone(),
        listen_apple_mail: listen_apple_mail.clone(),
        listen_outlook: listen_outlook.clone(),
        auto_enter: auto_enter.clone(),
    });

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("AutoCode - 验证码自动提取")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => {
                log::info!("用户退出 AutoCode");
                app.exit(0);
            }
            "settings" => {
                log::info!("打开设置窗口");
                show_settings(app);
            }
            "about" => {
                log::info!("显示主窗口");
                show_main_window(app);
            }
            "listen_imessage" => {
                toggle_source_from_tray(app, TraySource::Imessage);
            }
            "listen_apple_mail" => {
                toggle_source_from_tray(app, TraySource::AppleMail);
            }
            "listen_outlook" => {
                toggle_source_from_tray(app, TraySource::Outlook);
            }
            "auto_enter" => {
                toggle_auto_enter_from_tray(app);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
