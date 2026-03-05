use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub mod apple_mail;
pub mod imessage;
pub mod outlook;

/// 收到的新消息/邮件
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    /// 消息来源（例如 "iMessage", "Apple Mail", "Outlook", "Spotlight (...)"）
    pub source: String,
    /// 消息文本内容
    pub text: String,
    /// 发件人地址（如果有）
    pub sender: Option<String>,
}

/// 监控命令
#[derive(Debug)]
#[allow(dead_code)]
pub enum MonitorCommand {
    StartImessage,
    StopImessage,
    StartAppleMail,
    StopAppleMail,
    StartOutlook,
    StopOutlook,
    Shutdown,
}

/// 消息通道发送端类型
pub type MessageSender = mpsc::Sender<IncomingMessage>;
/// 消息通道接收端类型
pub type MessageReceiver = mpsc::Receiver<IncomingMessage>;
/// 命令通道发送端类型
pub type CommandSender = mpsc::Sender<MonitorCommand>;

/// 监控 Actor — 管理所有邮件/消息源
pub struct MonitorActor {
    cmd_rx: mpsc::Receiver<MonitorCommand>,
    msg_tx: MessageSender,
    imessage_handle: Option<tokio::task::JoinHandle<()>>,
    imessage_cancel: Option<CancellationToken>,
    apple_mail_handle: Option<tokio::task::JoinHandle<()>>,
    apple_mail_cancel: Option<CancellationToken>,
    outlook_handle: Option<tokio::task::JoinHandle<()>>,
    outlook_cancel: Option<CancellationToken>,
}

impl MonitorActor {
    pub fn new(cmd_rx: mpsc::Receiver<MonitorCommand>, msg_tx: MessageSender) -> Self {
        Self {
            cmd_rx,
            msg_tx,
            imessage_handle: None,
            imessage_cancel: None,
            apple_mail_handle: None,
            apple_mail_cancel: None,
            outlook_handle: None,
            outlook_cancel: None,
        }
    }

    /// 运行 Actor 事件循环
    pub async fn run(&mut self) {
        log::info!("MonitorActor 启动");

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                MonitorCommand::StartImessage => {
                    if self.imessage_handle.is_none() {
                        log::info!("启动 iMessage 监控");
                        let tx = self.msg_tx.clone();
                        let cancel_token = CancellationToken::new();
                        self.imessage_cancel = Some(cancel_token.clone());
                        self.imessage_handle = Some(tokio::spawn(async move {
                            tokio::select! {
                                result = imessage::monitor(tx) => {
                                    if let Err(e) = result {
                                        log::error!("iMessage 监控出错: {}", e);
                                    }
                                }
                                _ = cancel_token.cancelled() => {
                                    log::info!("iMessage 监控已取消");
                                }
                            }
                        }));
                    }
                }
                MonitorCommand::StopImessage => {
                    if let Some(cancel) = self.imessage_cancel.take() {
                        log::info!("停止 iMessage 监控");
                        cancel.cancel();
                    }
                    if let Some(handle) = self.imessage_handle.take() {
                        let _ = handle.await;
                    }
                }
                MonitorCommand::StartAppleMail => {
                    if self.apple_mail_handle.is_none() {
                        log::info!("启动 Apple Mail 监控");
                        let tx = self.msg_tx.clone();
                        let cancel_token = CancellationToken::new();
                        self.apple_mail_cancel = Some(cancel_token.clone());
                        self.apple_mail_handle = Some(tokio::spawn(async move {
                            tokio::select! {
                                result = apple_mail::monitor(tx) => {
                                    if let Err(e) = result {
                                        log::error!("Apple Mail 监控出错: {}", e);
                                    }
                                }
                                _ = cancel_token.cancelled() => {
                                    log::info!("Apple Mail 监控已取消");
                                }
                            }
                        }));
                    }
                }
                MonitorCommand::StopAppleMail => {
                    if let Some(cancel) = self.apple_mail_cancel.take() {
                        log::info!("停止 Apple Mail 监控");
                        cancel.cancel();
                    }
                    if let Some(handle) = self.apple_mail_handle.take() {
                        let _ = handle.await;
                    }
                }
                MonitorCommand::StartOutlook => {
                    if self.outlook_handle.is_none() {
                        log::info!("启动 Spotlight 邮件监控（兼容 Outlook）");
                        let tx = self.msg_tx.clone();
                        let cancel_token = CancellationToken::new();
                        self.outlook_cancel = Some(cancel_token.clone());
                        self.outlook_handle = Some(tokio::spawn(async move {
                            tokio::select! {
                                result = outlook::monitor(tx) => {
                                    if let Err(e) = result {
                                        log::error!("Spotlight 邮件监控出错: {}", e);
                                    }
                                }
                                _ = cancel_token.cancelled() => {
                                    log::info!("Spotlight 邮件监控已取消");
                                }
                            }
                        }));
                    }
                }
                MonitorCommand::StopOutlook => {
                    if let Some(cancel) = self.outlook_cancel.take() {
                        log::info!("停止 Spotlight 邮件监控");
                        cancel.cancel();
                    }
                    if let Some(handle) = self.outlook_handle.take() {
                        let _ = handle.await;
                    }
                }
                MonitorCommand::Shutdown => {
                    log::info!("MonitorActor 收到关闭命令");
                    // 优雅地停止所有监控
                    if let Some(cancel) = self.imessage_cancel.take() {
                        cancel.cancel();
                    }
                    if let Some(cancel) = self.apple_mail_cancel.take() {
                        cancel.cancel();
                    }
                    if let Some(cancel) = self.outlook_cancel.take() {
                        cancel.cancel();
                    }
                    // 等待所有任务完成
                    if let Some(h) = self.imessage_handle.take() {
                        let _ = h.await;
                    }
                    if let Some(h) = self.apple_mail_handle.take() {
                        let _ = h.await;
                    }
                    if let Some(h) = self.outlook_handle.take() {
                        let _ = h.await;
                    }
                    break;
                }
            }
        }

        log::info!("MonitorActor 已退出");
    }
}

/// 启动监控系统，返回命令发送端和消息接收端
pub fn start_monitor() -> (CommandSender, MessageReceiver) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<MonitorCommand>(32);
    let (msg_tx, msg_rx) = mpsc::channel::<IncomingMessage>(64);

    let mut actor = MonitorActor::new(cmd_rx, msg_tx);

    tokio::spawn(async move {
        actor.run().await;
    });

    (cmd_tx, msg_rx)
}
