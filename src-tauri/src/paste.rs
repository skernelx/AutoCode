use anyhow::Result;
use log;
use std::time::Duration;
use tokio::time;

use crate::clipboard;
use crate::config::{AppConfig, PasteMode, SharedConfig};
use crate::extractor::{CodeExtractor, VerificationCode};
use crate::monitor::IncomingMessage;
use tauri::Emitter;

/// 智能粘贴决策
#[derive(Debug)]
enum PasteDecision {
    /// 直接自动粘贴
    AutoPaste,
    /// 只通知（悬浮窗/通知），不自动粘贴
    NotifyOnly,
    /// 等待检测后再决定
    DelayedCheck {
        /// 触发延时检测时的前台应用（通常是原生 AutoFill 应用）
        bundle_id: Option<String>,
    },
}

/// 处理传入的消息：提取验证码 → 决策 → 执行粘贴/通知
pub async fn handle_message(
    msg: IncomingMessage,
    config: &SharedConfig,
    extractor: &CodeExtractor,
    app_handle: &tauri::AppHandle,
) {
    let cfg = config.read().unwrap().clone();

    // 尝试提取验证码
    let code = extractor.extract(&msg.text, msg.sender.as_deref(), &cfg);

    let code = match code {
        Some(c) => c,
        None => {
            log::debug!("消息中未发现验证码 (来源: {})", msg.source);
            return;
        }
    };

    log::info!(
        "发现验证码: {} (来源: {}, 策略: {}, 置信度: {:.0}%)",
        code.code,
        msg.source,
        code.source,
        code.confidence * 100.0
    );

    // 通知前端（悬浮窗/通知栏）
    let _ = app_handle.emit("verification-code", serde_json::json!({
        "code": code.code,
        "source": msg.source,
        "strategy": code.source,
        "confidence": code.confidence,
    }));

    // 决策并执行粘贴
    execute_paste(&cfg, &code, &msg.source).await;
}

/// 根据配置和当前环境决定并执行粘贴操作
async fn execute_paste(config: &AppConfig, code: &VerificationCode, source: &str) {
    let decision = make_paste_decision(config, source);

    match decision {
        PasteDecision::AutoPaste => {
            do_paste(config, &code.code).await;
        }
        PasteDecision::DelayedCheck { bundle_id } => {
            // Smart 模式：等待一段时间让系统 AutoFill 先工作
            let delay = Duration::from_millis(config.autofill_detect_delay_ms);
            log::info!("Smart 模式：等待 {}ms 检测系统 AutoFill...", config.autofill_detect_delay_ms);
            time::sleep(delay).await;

            // 延时后仍在原生 AutoFill App 内，优先避免冲突，改为仅复制
            if let Some(expected_bundle) = bundle_id {
                let current_bundle = clipboard::get_frontmost_app_bundle_id();
                if current_bundle.as_deref() == Some(expected_bundle.as_str()) {
                    log::info!(
                        "Smart 模式：{} 仍在前台，跳过自动输入避免冲突，改为仅复制",
                        expected_bundle
                    );
                    copy_code_only(&code.code);
                    return;
                }
            }

            do_paste(config, &code.code).await;
        }
        PasteDecision::NotifyOnly => {
            log::info!("仅通知模式：验证码 {} 已发送到前端", code.code);
            copy_code_only(&code.code);
        }
    }
}

/// 粘贴决策逻辑
fn make_paste_decision(config: &AppConfig, source: &str) -> PasteDecision {
    match config.paste_mode {
        PasteMode::Always => PasteDecision::AutoPaste,
        PasteMode::FloatingOnly | PasteMode::ClipboardOnly => PasteDecision::NotifyOnly,
        PasteMode::Smart => {
            // iMessage 来源需要考虑系统 AutoFill 冲突
            if source == "iMessage" {
                // 检查当前前台 App 是否有原生 AutoFill 支持
                if let Some(bundle_id) = clipboard::get_frontmost_app_bundle_id() {
                    if config.native_autofill_apps.iter().any(|id| id == &bundle_id) {
                        log::info!("当前 App {} 支持原生 AutoFill，使用延时检测", bundle_id);
                        return PasteDecision::DelayedCheck {
                            bundle_id: Some(bundle_id),
                        };
                    }
                }
                // 非原生 App，直接粘贴
                PasteDecision::AutoPaste
            } else {
                // 邮件来源（Apple Mail、Outlook）：系统不支持 AutoFill，直接粘贴
                PasteDecision::AutoPaste
            }
        }
    }
}

/// 仅复制验证码到剪贴板
fn copy_code_only(code: &str) {
    if let Err(e) = clipboard::copy_to_clipboard(code) {
        log::error!("复制到剪贴板失败: {}", e);
    } else {
        log::info!("验证码已复制到剪贴板");
    }
}

/// 执行实际的粘贴操作
async fn do_paste(config: &AppConfig, code: &str) {
    // 使用 blocking task 执行键盘操作（避免阻塞 tokio）
    let code = code.to_string();
    let auto_enter = config.auto_enter;

    let result = tokio::task::spawn_blocking(move || -> Result<()> {
        // 先复制到剪贴板（作为保底，即使键盘输入失败用户也能手动粘贴）
        if let Err(e) = clipboard::copy_to_clipboard(&code) {
            log::warn!("复制到剪贴板失败: {}", e);
        }

        // 尝试直接键盘输入（需要 Accessibility 权限）
        match clipboard::type_text(&code) {
            Ok(()) => {
                log::info!("已自动输入验证码: {}", code);

                if auto_enter {
                    if let Err(e) = clipboard::press_enter() {
                        log::warn!("自动回车失败: {}", e);
                    } else {
                        log::info!("已自动按下回车");
                    }
                }
            }
            Err(e) => {
                // Accessibility 权限不足，回退到仅复制模式
                log::warn!("键盘输入失败（可能缺少辅助功能权限）: {}", e);
                log::info!("验证码已复制到剪贴板，请手动 Cmd+V 粘贴");
            }
        }

        Ok(())
    }).await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => log::error!("粘贴操作失败: {}", e),
        Err(e) => log::error!("粘贴任务 panic: {}", e),
    }
}
