use tokio::sync::mpsc;

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
    apple_mail_handle: Option<tokio::task::JoinHandle<()>>,
    outlook_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MonitorActor {
    pub fn new(cmd_rx: mpsc::Receiver<MonitorCommand>, msg_tx: MessageSender) -> Self {
        Self {
            cmd_rx,
            msg_tx,
            imessage_handle: None,
            apple_mail_handle: None,
            outlook_handle: None,
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
                        self.imessage_handle = Some(tokio::spawn(async move {
                            if let Err(e) = imessage::monitor(tx).await {
                                log::error!("iMessage 监控出错: {}", e);
                            }
                        }));
                    }
                }
                MonitorCommand::StopImessage => {
                    if let Some(handle) = self.imessage_handle.take() {
                        log::info!("停止 iMessage 监控");
                        handle.abort();
                    }
                }
                MonitorCommand::StartAppleMail => {
                    if self.apple_mail_handle.is_none() {
                        log::info!("启动 Apple Mail 监控");
                        let tx = self.msg_tx.clone();
                        self.apple_mail_handle = Some(tokio::spawn(async move {
                            if let Err(e) = apple_mail::monitor(tx).await {
                                log::error!("Apple Mail 监控出错: {}", e);
                            }
                        }));
                    }
                }
                MonitorCommand::StopAppleMail => {
                    if let Some(handle) = self.apple_mail_handle.take() {
                        log::info!("停止 Apple Mail 监控");
                        handle.abort();
                    }
                }
                MonitorCommand::StartOutlook => {
                    if self.outlook_handle.is_none() {
                        log::info!("启动 Spotlight 邮件监控（兼容 Outlook）");
                        let tx = self.msg_tx.clone();
                        self.outlook_handle = Some(tokio::spawn(async move {
                            if let Err(e) = outlook::monitor(tx).await {
                                log::error!("Spotlight 邮件监控出错: {}", e);
                            }
                        }));
                    }
                }
                MonitorCommand::StopOutlook => {
                    if let Some(handle) = self.outlook_handle.take() {
                        log::info!("停止 Spotlight 邮件监控");
                        handle.abort();
                    }
                }
                MonitorCommand::Shutdown => {
                    log::info!("MonitorActor 收到关闭命令");
                    // 停止所有监控
                    if let Some(h) = self.imessage_handle.take() {
                        h.abort();
                    }
                    if let Some(h) = self.apple_mail_handle.take() {
                        h.abort();
                    }
                    if let Some(h) = self.outlook_handle.take() {
                        h.abort();
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
