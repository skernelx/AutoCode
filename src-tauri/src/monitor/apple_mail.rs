use anyhow::{Context, Result};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc as tokio_mpsc;

use super::{IncomingMessage, MessageSender};

/// 获取 Apple Mail 数据目录
/// macOS 不同版本使用不同的 Mail 存储版本号
fn find_mail_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("无法获取 HOME 环境变量")?;
    let base = PathBuf::from(&home).join("Library/Mail");

    // 尝试 V10 (Sonoma+), V9, V8, ...
    for version in (5..=12).rev() {
        let dir = base.join(format!("V{}", version));
        if dir.exists() {
            log::info!("Apple Mail 数据目录: {:?}", dir);
            return Ok(dir);
        }
    }

    anyhow::bail!("未找到 Apple Mail 数据目录（~/Library/Mail/V*）")
}

/// 从 .emlx 文件读取邮件内容
fn read_emlx_content(path: &PathBuf) -> Result<(String, Option<String>)> {
    let raw = fs::read(path)
        .with_context(|| format!("读取 .emlx 文件失败: {:?}", path))?;

    let content = String::from_utf8_lossy(&raw).to_string();

    // .emlx 格式：第一行是字节计数，之后是 RFC 822 格式的邮件
    // 提取正文（简单处理：跳过头部，取 body）
    let body = extract_body_from_raw(&content);
    let sender = extract_sender_from_raw(&content);

    Ok((body, sender))
}

/// 从原始邮件内容提取正文
fn extract_body_from_raw(content: &str) -> String {
    // 跳过第一行（字节计数）
    let content = content.lines().skip(1).collect::<Vec<_>>().join("\n");

    // 找到空行（头部和正文的分隔）
    if let Some(pos) = content.find("\r\n\r\n") {
        content[pos + 4..].to_string()
    } else if let Some(pos) = content.find("\n\n") {
        content[pos + 2..].to_string()
    } else {
        content
    }
}

/// 从原始邮件头部提取发件人
fn extract_sender_from_raw(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.is_empty() || line == "\r" {
            break; // 到达头部和正文分隔
        }
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("from:") {
            let sender = line[5..].trim().to_string();
            // 提取 <email> 中的地址
            if let Some(start) = sender.find('<') {
                if let Some(end) = sender.find('>') {
                    return Some(sender[start + 1..end].to_string());
                }
            }
            return Some(sender);
        }
    }
    None
}

/// Apple Mail 监控主循环
pub async fn monitor(tx: MessageSender) -> Result<()> {
    let mail_dir = find_mail_dir()?;

    log::info!("Apple Mail 监控启动，目录: {:?}", mail_dir);

    let (notify_tx, mut notify_rx) = tokio_mpsc::channel::<PathBuf>(100);

    // 创建文件监控器
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            // 只处理新建文件事件
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        if path.extension().and_then(|e| e.to_str()) == Some("emlx") {
                            let _ = notify_tx.blocking_send(path);
                        }
                    }
                }
                _ => {}
            }
        }
    }).context("创建文件监控器失败")?;

    watcher
        .watch(&mail_dir, RecursiveMode::Recursive)
        .context("启动 Apple Mail 目录监控失败")?;

    log::info!("Apple Mail 文件监控已启动");

    // 简单的去抖：记录最近处理的文件路径和时间
    let mut last_processed: Option<(PathBuf, std::time::Instant)> = None;
    let debounce_duration = std::time::Duration::from_millis(500);

    while let Some(path) = notify_rx.recv().await {
        // 去抖：500ms 内的相同文件事件忽略
        if let Some((ref last_path, last_time)) = last_processed {
            if last_path == &path && last_time.elapsed() < debounce_duration {
                continue;
            }
        }

        // 等待文件写入完成
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        match read_emlx_content(&path) {
            Ok((body, sender)) => {
                if body.is_empty() {
                    continue;
                }

                log::debug!(
                    "Apple Mail 新邮件: {:?} (发件人: {:?})",
                    &body[..body.len().min(80)],
                    sender
                );

                if tx.send(IncomingMessage {
                    source: "Apple Mail".into(),
                    text: body,
                    sender,
                }).await.is_err() {
                    log::error!("消息通道已关闭");
                    return Ok(());
                }

                last_processed = Some((path, std::time::Instant::now()));
            }
            Err(e) => {
                log::warn!("读取 Apple Mail 邮件失败: {}", e);
            }
        }
    }

    Ok(())
}
