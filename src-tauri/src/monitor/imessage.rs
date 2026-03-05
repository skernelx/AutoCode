use anyhow::{Context, Result};
use rusqlite::Connection;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::time::{self, Duration};

use super::{IncomingMessage, MessageSender};

/// 上次处理的最大 ROWID（原子操作，线程安全）
static LAST_ROWID: AtomicI64 = AtomicI64::new(0);

/// 获取 iMessage 数据库路径
fn db_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("无法获取 HOME 环境变量")?;
    Ok(PathBuf::from(home).join("Library/Messages/chat.db"))
}

/// 查询最新消息
fn query_new_messages(
    conn: &Connection,
    since_rowid: i64,
) -> Result<Vec<(i64, String, Option<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT m.ROWID, m.text, h.id as sender
         FROM message m
         LEFT JOIN handle h ON m.handle_id = h.ROWID
         WHERE m.ROWID > ?1
           AND m.text IS NOT NULL
           AND m.text != ''
         ORDER BY m.ROWID ASC
         LIMIT 20",
    )?;

    let rows = stmt.query_map([since_rowid], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let mut messages = Vec::new();
    for row in rows {
        match row {
            Ok(msg) => messages.push(msg),
            Err(e) => log::warn!("解析消息行出错: {}", e),
        }
    }

    Ok(messages)
}

/// 获取当前最大 ROWID（首次启动时用）
fn get_max_rowid(conn: &Connection) -> Result<i64> {
    let rowid: i64 = conn.query_row("SELECT COALESCE(MAX(ROWID), 0) FROM message", [], |row| {
        row.get(0)
    })?;
    Ok(rowid)
}

/// iMessage 监控主循环
pub async fn monitor(tx: MessageSender) -> Result<()> {
    let path = db_path()?;

    if !path.exists() {
        anyhow::bail!(
            "iMessage 数据库不存在: {:?}（需要「完全磁盘访问」权限）",
            path
        );
    }

    log::info!("iMessage 监控启动，数据库路径: {:?}", path);

    // 以只读模式打开数据库
    let conn = Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("无法打开 iMessage 数据库（需要「完全磁盘访问」权限）")?;

    // 初始化：获取当前最大 ROWID，只处理之后的新消息
    let current_max = get_max_rowid(&conn)?;
    LAST_ROWID.store(current_max, Ordering::SeqCst);
    log::info!("iMessage 初始 ROWID: {}", current_max);

    // 轮询间隔 1 秒
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let last = LAST_ROWID.load(Ordering::SeqCst);

        match query_new_messages(&conn, last) {
            Ok(messages) => {
                for (rowid, text, sender) in messages {
                    log::debug!(
                        "iMessage 新消息 [ROWID={}]: {}",
                        rowid,
                        &text[..text.len().min(50)]
                    );

                    if tx
                        .send(IncomingMessage {
                            source: "iMessage".into(),
                            text,
                            sender,
                        })
                        .await
                        .is_err()
                    {
                        log::error!("消息通道已关闭");
                        return Ok(());
                    }

                    // 更新 ROWID（原子操作）
                    LAST_ROWID.fetch_max(rowid, Ordering::SeqCst);
                }
            }
            Err(e) => {
                log::warn!("查询 iMessage 出错: {}（数据库可能被锁定）", e);
            }
        }
    }
}
