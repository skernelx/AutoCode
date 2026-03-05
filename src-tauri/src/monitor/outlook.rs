use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::process::Command;
use tokio::time::{self, Duration};

use super::{IncomingMessage, MessageSender};

/// Outlook 邮件元数据
#[derive(Debug)]
struct OutlookEmail {
    subject: Option<String>,
    content: Option<String>,
    author: Option<String>,
}

/// 通过 Spotlight (mdfind) 查询 Outlook 最近的邮件
fn query_recent_outlook_emails(since_seconds: u64) -> Result<Vec<String>> {
    let home = env::var("HOME").context("无法获取 HOME")?;
    let outlook_dir = format!(
        "{}/Library/Group Containers/UBF8T346G9.Office/Outlook",
        home
    );

    // 使用 mdfind 搜索最近的 Outlook 邮件消息
    let output = Command::new("mdfind")
        .args([
            "-onlyin",
            &outlook_dir,
            &format!(
                "kMDItemContentModificationDate >= $time.now(-{})",
                since_seconds
            ),
        ])
        .output()
        .context("执行 mdfind 失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("mdfind 返回错误: {}", stderr);
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| {
            // 过滤 Outlook 邮件文件
            line.contains("olk16") || line.contains("olk15") || line.ends_with("Message")
        })
        .map(|s| s.to_string())
        .collect();

    Ok(paths)
}

/// 使用 mdls 获取文件的 Spotlight 元数据
fn get_spotlight_metadata(path: &str) -> Result<OutlookEmail> {
    let output = Command::new("mdls")
        .args([
            "-name", "kMDItemSubject",
            "-name", "kMDItemTextContent",
            "-name", "kMDItemAuthors",
            "-name", "kMDItemDisplayName",
            path,
        ])
        .output()
        .context("执行 mdls 失败")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut email = OutlookEmail {
        subject: None,
        content: None,
        author: None,
    };

    // mdls 输出可能包含多行值（尤其是 kMDItemTextContent），需要正确处理
    let mut current_key: Option<&str> = None;
    let mut current_value = String::new();
    let mut in_multiline = false;

    for line in stdout.lines() {
        let trimmed = line.trim();

        // 检查是否新的 key = value 行
        if let Some(eq_pos) = trimmed.find(" = ") {
            // 先保存上一个 key 的值
            if let Some(key) = current_key.take() {
                apply_metadata_value(&mut email, key, &current_value);
            }

            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 3..].trim();

            if value == "(null)" {
                current_key = None;
                current_value.clear();
                in_multiline = false;
            } else if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
                // 单行字符串值 — 直接保存
                let single_value = value[1..value.len()-1].to_string();
                apply_metadata_value(&mut email, key, &single_value);
                current_key = None;
                current_value.clear();
                in_multiline = false;
            } else if value.starts_with('"') {
                // 多行字符串值的开始
                let _ = current_key.replace(key);
                current_value = value[1..].to_string();
                in_multiline = true;
            } else if value.starts_with('(') {
                // 数组值
                let _ = current_key.replace(key);
                current_value = value.to_string();
                in_multiline = !value.ends_with(')');
            } else {
                current_value = value.to_string();
                apply_metadata_value(&mut email, key, &current_value);
                current_key = None;
                current_value.clear();
                in_multiline = false;
            }
        } else if in_multiline {
            // 多行值的后续行
            if trimmed.ends_with('"') || trimmed.ends_with(')') {
                let end = if trimmed.ends_with('"') {
                    &trimmed[..trimmed.len()-1]
                } else {
                    trimmed
                };
                current_value.push('\n');
                current_value.push_str(end);
                in_multiline = false;

                if let Some(key) = current_key.take() {
                    apply_metadata_value(&mut email, key, &current_value);
                }
                current_value.clear();
            } else {
                current_value.push('\n');
                current_value.push_str(trimmed);
            }
        }
    }

    // 处理最后一个 key
    if let Some(key) = current_key {
        apply_metadata_value(&mut email, key, &current_value);
    }

    Ok(email)
}

/// 将解析到的值应用到邮件结构体
fn apply_metadata_value(email: &mut OutlookEmail, key: &str, value: &str) {
    let clean_value = value.trim().to_string();
    if clean_value.is_empty() || clean_value == "(null)" {
        return;
    }

    // 处理数组格式 ("val1", "val2")
    let final_value = if clean_value.starts_with('(') {
        clean_value
            .trim_start_matches('(')
            .trim_end_matches(')')
            .split(',')
            .next()
            .map(|v| v.trim().trim_matches('"').trim().to_string())
            .unwrap_or_default()
    } else {
        clean_value
    };

    if final_value.is_empty() {
        return;
    }

    match key {
        "kMDItemSubject" => email.subject = Some(final_value),
        "kMDItemTextContent" => email.content = Some(final_value),
        "kMDItemAuthors" => email.author = Some(final_value),
        "kMDItemDisplayName" => {
            if email.subject.is_none() {
                email.subject = Some(final_value);
            }
        }
        _ => {}
    }
}

/// Outlook 监控主循环（基于 Spotlight 轮询）
pub async fn monitor(tx: MessageSender) -> Result<()> {
    log::info!("Outlook Spotlight 监控启动");

    // 已处理过的文件路径集合
    let mut processed: HashSet<String> = HashSet::new();

    // 首次启动：标记当前已有的文件为已处理
    if let Ok(existing) = query_recent_outlook_emails(60) {
        for path in existing {
            processed.insert(path);
        }
        log::info!("Outlook 初始化：标记 {} 封已有邮件", processed.len());
    }

    // 轮询间隔 3 秒
    let mut interval = time::interval(Duration::from_secs(3));

    loop {
        interval.tick().await;

        // 查询最近 10 秒内修改的文件
        match query_recent_outlook_emails(10) {
            Ok(paths) => {
                for path in paths {
                    if processed.contains(&path) {
                        continue;
                    }

                    // 标记为已处理
                    processed.insert(path.clone());

                    // 获取元数据
                    match get_spotlight_metadata(&path) {
                        Ok(email) => {
                            // 合并主题和内容
                            let text = match (&email.subject, &email.content) {
                                (Some(subj), Some(content)) => format!("{}\n{}", subj, content),
                                (Some(subj), None) => subj.clone(),
                                (None, Some(content)) => content.clone(),
                                (None, None) => continue,
                            };

                            if text.is_empty() {
                                continue;
                            }

                            log::debug!(
                                "Outlook 新邮件: {} (发件人: {:?})",
                                &text[..text.len().min(80)],
                                email.author
                            );

                            if tx.send(IncomingMessage {
                                source: "Outlook".into(),
                                text,
                                sender: email.author,
                            }).await.is_err() {
                                log::error!("消息通道已关闭");
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            log::warn!("获取 Outlook 邮件元数据失败: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("查询 Outlook Spotlight 失败: {}", e);
            }
        }

        // 定期清理已处理集合，防止内存无限增长
        if processed.len() > 10000 {
            processed.clear();
            log::debug!("清理 Outlook 已处理邮件缓存");
        }
    }
}
