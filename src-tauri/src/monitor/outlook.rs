use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::time::{self, Duration};

use super::{IncomingMessage, MessageSender};

/// Spotlight 命中的邮件候选
#[derive(Debug)]
struct SpotlightHit {
    source: String,
    path: String,
}

/// 邮件客户端 Spotlight 扫描目标
#[derive(Debug, Clone)]
struct SpotlightTarget {
    label: String,
    root: PathBuf,
    strict_outlook_filter: bool,
}

/// 从 Spotlight 元数据解析出的邮件信息
#[derive(Debug)]
struct SpotlightEmail {
    subject: Option<String>,
    content: Option<String>,
    author: Option<String>,
}

const POLL_SECONDS: u64 = 3;
const OUTLOOK_LABEL: &str = "Outlook";
const OUTLOOK_ROOT: &str = "Library/Group Containers/UBF8T346G9.Office/Outlook";

fn add_target(
    targets: &mut Vec<SpotlightTarget>,
    dedup: &mut HashSet<PathBuf>,
    label: impl Into<String>,
    root: PathBuf,
    strict_outlook_filter: bool,
) {
    if !root.exists() || !root.is_dir() {
        return;
    }
    if dedup.insert(root.clone()) {
        targets.push(SpotlightTarget {
            label: label.into(),
            root,
            strict_outlook_filter,
        });
    }
}

fn is_mail_like_name(name: &str) -> bool {
    let n = name.to_lowercase();
    let keywords = [
        "mail",
        "outlook",
        "spark",
        "airmail",
        "canary",
        "thunderbird",
        "mailspring",
        "postbox",
        "inbox",
        "message",
    ];
    keywords.iter().any(|kw| n.contains(kw))
}

/// 扫描目录中的 mail-like 子目录，自动纳入 Spotlight 目标
fn discover_mail_like_subdirs(
    base: &Path,
    label_prefix: &str,
    add_data_subdir: bool,
    targets: &mut Vec<SpotlightTarget>,
    dedup: &mut HashSet<PathBuf>,
) {
    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let dir_name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !is_mail_like_name(&dir_name) {
            continue;
        }

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if add_data_subdir {
            let data_path = path.join("Data");
            if data_path.exists() && data_path.is_dir() {
                add_target(
                    targets,
                    dedup,
                    format!("{}: {}", label_prefix, dir_name),
                    data_path,
                    false,
                );
            } else {
                add_target(
                    targets,
                    dedup,
                    format!("{}: {}", label_prefix, dir_name),
                    path,
                    false,
                );
            }
        } else {
            add_target(
                targets,
                dedup,
                format!("{}: {}", label_prefix, dir_name),
                path,
                false,
            );
        }
    }
}

/// 自动发现可用的 Spotlight 邮件扫描目录（Outlook 保证兼容）
fn discover_targets() -> Result<Vec<SpotlightTarget>> {
    let home = PathBuf::from(env::var("HOME").context("无法获取 HOME")?);
    let mut targets = Vec::new();
    let mut dedup = HashSet::new();

    // 1) Outlook 官方路径（优先 + 保留旧过滤规则）
    add_target(
        &mut targets,
        &mut dedup,
        OUTLOOK_LABEL,
        home.join(OUTLOOK_ROOT),
        true,
    );

    // 2) 常见邮件客户端路径
    let common_roots = [
        ("Thunderbird", "Library/Thunderbird/Profiles", false),
        (
            "Mailspring",
            "Library/Application Support/Mailspring",
            false,
        ),
        ("MailMate", "Library/Application Support/MailMate", false),
        (
            "Airmail",
            "Library/Containers/it.bloop.airmail2/Data",
            false,
        ),
        (
            "Spark",
            "Library/Containers/com.readdle.smartemail-Mac/Data",
            false,
        ),
        (
            "Canary Mail",
            "Library/Containers/io.canarymail.mac/Data",
            false,
        ),
    ];
    for (label, rel, strict) in common_roots {
        add_target(&mut targets, &mut dedup, label, home.join(rel), strict);
    }

    // 3) 动态发现 mail-like 容器目录（提升通用性）
    discover_mail_like_subdirs(
        &home.join("Library/Containers"),
        "Container",
        true,
        &mut targets,
        &mut dedup,
    );
    discover_mail_like_subdirs(
        &home.join("Library/Group Containers"),
        "GroupContainer",
        false,
        &mut targets,
        &mut dedup,
    );
    discover_mail_like_subdirs(
        &home.join("Library/Application Support"),
        "AppSupport",
        false,
        &mut targets,
        &mut dedup,
    );

    Ok(targets)
}

fn source_name_for_target(target: &SpotlightTarget) -> String {
    if target.label == OUTLOOK_LABEL {
        OUTLOOK_LABEL.to_string()
    } else {
        format!("Spotlight ({})", target.label)
    }
}

fn is_likely_mail_candidate(path: &str, strict_outlook_filter: bool) -> bool {
    if strict_outlook_filter {
        // 兼容旧版 Outlook 过滤规则，保证现有行为稳定
        return path.contains("olk16") || path.contains("olk15") || path.ends_with("Message");
    }

    let p = path.to_lowercase();
    p.ends_with(".emlx")
        || p.ends_with(".eml")
        || p.ends_with(".msg")
        || p.contains("/mail/")
        || p.contains("/messages/")
        || p.contains("outlook")
        || p.contains("thunderbird")
        || p.contains("mailspring")
        || p.contains("canary")
        || p.contains("spark")
        || p.contains("airmail")
}

/// 在指定目标目录中查询最近变更文件
fn query_recent_in_target(
    target: &SpotlightTarget,
    since_seconds: u64,
) -> Result<Vec<SpotlightHit>> {
    let output = Command::new("mdfind")
        .args([
            "-onlyin",
            target.root.to_string_lossy().as_ref(),
            &format!(
                "kMDItemContentModificationDate >= $time.now(-{})",
                since_seconds
            ),
        ])
        .output()
        .context("执行 mdfind 失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("mdfind 返回错误 [{}]: {}", target.label, stderr);
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let hits = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| is_likely_mail_candidate(line, target.strict_outlook_filter))
        .map(|p| SpotlightHit {
            source: source_name_for_target(target),
            path: p.to_string(),
        })
        .collect();

    Ok(hits)
}

/// 汇总多个目标目录的 Spotlight 结果，并按路径去重
fn query_recent_spotlight_emails(
    targets: &[SpotlightTarget],
    since_seconds: u64,
) -> Vec<SpotlightHit> {
    let mut merged = Vec::new();
    let mut seen_paths = HashSet::new();

    for target in targets {
        match query_recent_in_target(target, since_seconds) {
            Ok(hits) => {
                for hit in hits {
                    if seen_paths.insert(hit.path.clone()) {
                        merged.push(hit);
                    }
                }
            }
            Err(e) => {
                log::warn!("查询 Spotlight 目标失败 [{}]: {}", target.label, e);
            }
        }
    }

    merged
}

/// 使用 mdls 获取文件的 Spotlight 元数据
fn get_spotlight_metadata(path: &str) -> Result<SpotlightEmail> {
    let output = Command::new("mdls")
        .args([
            "-name",
            "kMDItemSubject",
            "-name",
            "kMDItemTextContent",
            "-name",
            "kMDItemAuthors",
            "-name",
            "kMDItemDisplayName",
            path,
        ])
        .output()
        .context("执行 mdls 失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("mdls 命令失败: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut email = SpotlightEmail {
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
                let single_value = value[1..value.len() - 1].to_string();
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
                    &trimmed[..trimmed.len() - 1]
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
fn apply_metadata_value(email: &mut SpotlightEmail, key: &str, value: &str) {
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

/// 通用 Spotlight 邮件监控主循环（保留 Outlook 完整兼容）
pub async fn monitor(tx: MessageSender) -> Result<()> {
    log::info!("Spotlight 邮件监控启动");

    // 可扫描的邮件目录目标
    let mut targets = discover_targets().unwrap_or_default();
    if targets.is_empty() {
        log::warn!("Spotlight 未发现可用邮件目录，稍后会自动重试发现");
    } else {
        log::info!("Spotlight 已发现 {} 个邮件目录目标", targets.len());
    }

    // 已处理过的文件路径集合
    let mut processed: HashSet<String> = HashSet::new();

    // 首次启动：标记当前已有的文件为已处理
    for hit in query_recent_spotlight_emails(&targets, 60) {
        processed.insert(hit.path);
    }
    if !processed.is_empty() {
        log::info!("Spotlight 初始化：标记 {} 条已有记录", processed.len());
    }

    // 轮询间隔
    let mut interval = time::interval(Duration::from_secs(POLL_SECONDS));
    let mut tick_count: u64 = 0;

    loop {
        interval.tick().await;
        tick_count += 1;

        // 周期性重新发现目录，支持后续安装/迁移邮件客户端
        if tick_count % 40 == 0 {
            match discover_targets() {
                Ok(new_targets) => {
                    if !new_targets.is_empty() && new_targets.len() != targets.len() {
                        log::info!(
                            "Spotlight 目标目录更新: {} -> {}",
                            targets.len(),
                            new_targets.len()
                        );
                    }
                    targets = new_targets;
                }
                Err(e) => {
                    log::warn!("重新发现 Spotlight 目标失败: {}", e);
                }
            }
        }

        if targets.is_empty() {
            continue;
        }

        // 查询最近 10 秒内修改的文件
        for hit in query_recent_spotlight_emails(&targets, 10) {
            if processed.contains(&hit.path) {
                continue;
            }

            // 标记为已处理
            processed.insert(hit.path.clone());

            // 获取元数据
            match get_spotlight_metadata(&hit.path) {
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
                        "{} 新邮件: {} (发件人: {:?})",
                        hit.source,
                        &text[..text.len().min(80)],
                        email.author
                    );

                    if tx
                        .send(IncomingMessage {
                            source: hit.source,
                            text,
                            sender: email.author,
                        })
                        .await
                        .is_err()
                    {
                        log::error!("消息通道已关闭");
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::warn!("读取 Spotlight 邮件元数据失败: {}", e);
                }
            }
        }

        // 定期清理已处理集合，防止内存无限增长
        if processed.len() > 20000 {
            processed.clear();
            log::debug!("清理 Spotlight 已处理邮件缓存");
        }
    }
}
