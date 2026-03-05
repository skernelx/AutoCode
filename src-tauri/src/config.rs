use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 粘贴行为模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PasteMode {
    /// 智能检测 — 避免与系统 AutoFill 冲突
    Smart,
    /// 总是自动粘贴
    Always,
    /// 只显示悬浮窗，手动点击
    FloatingOnly,
    /// 只复制到剪贴板
    ClipboardOnly,
}

impl Default for PasteMode {
    fn default() -> Self {
        Self::Smart
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 是否监听 iMessage 短信
    pub listen_imessage: bool,
    /// 是否监听 Apple Mail
    pub listen_apple_mail: bool,
    /// 是否监听 Spotlight 邮件源（兼容 Outlook 等客户端）
    pub listen_outlook: bool,
    /// 粘贴行为
    pub paste_mode: PasteMode,
    /// 自动粘贴后是否按回车
    pub auto_enter: bool,
    /// 开机自启
    pub launch_at_login: bool,
    /// Smart 模式下等待 macOS AutoFill 的时间（毫秒）
    pub autofill_detect_delay_ms: u64,
    /// 验证码关键词
    pub verification_keywords: Vec<String>,
    /// 验证码正则表达式列表（模板匹配）
    pub verification_patterns: Vec<String>,
    /// 已知验证码发件人地址（白名单）
    pub known_2fa_senders: Vec<String>,
    /// 系统 AutoFill 原生支持的 App Bundle IDs
    pub native_autofill_apps: Vec<String>,
    /// 配置文件版本
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            listen_imessage: true,
            listen_apple_mail: true,
            listen_outlook: true,
            paste_mode: PasteMode::Smart,
            auto_enter: false,
            launch_at_login: false,
            autofill_detect_delay_ms: 1500,
            verification_keywords: vec![
                "验证码".into(),
                "校验码".into(),
                "动态密码".into(),
                "安全码".into(),
                "verification code".into(),
                "security code".into(),
                "confirmation code".into(),
                "one-time code".into(),
                "OTP".into(),
                "captcha".into(),
                "sign-in code".into(),
                "login code".into(),
                "인증".into(),
            ],
            verification_patterns: vec![
                // 中文模板：验证码是 123456
                r"(?:验证码|校验码|动态密码|安全码)[：:是为\s]*(\d{4,8})".into(),
                // 英文模板：Your verification code is 123456
                r"(?:verification|confirmation|security|sign-?in|login)\s*code[：:is\s]+([A-Za-z0-9]{4,8})".into(),
                // OTP 模板：Your OTP is 123456
                r"(?:Your|The)\s+(?:OTP|code|PIN|token)[：:is\s]+([A-Za-z0-9]{4,8})".into(),
                // 反向：123456 是你的验证码
                r"(\d{4,8})\s*(?:是|为|is)?\s*(?:你的|您的|your)\s*(?:验证码|code)".into(),
                // 带有效期：123456（5分钟内有效）
                r"(\d{4,8})\s*[（(].*(?:有效|valid|expire)".into(),
                // Microsoft 特有：Security code: 123456
                r"(?:Security code|安全代码|Sign-in code|登录代码)[：:\s]+(\d{4,8})".into(),
            ],
            known_2fa_senders: vec![
                "noreply@microsoft.com".into(),
                "account-security-noreply@accountprotection.microsoft.com".into(),
                "no-reply@accounts.google.com".into(),
                "noreply@github.com".into(),
                "verify@twitter.com".into(),
                "noreply@apple.com".into(),
                "noreply@steam.com".into(),
            ],
            native_autofill_apps: vec![
                "com.apple.Safari".into(),
                "com.apple.systempreferences".into(),
                "com.apple.AppStore".into(),
            ],
            version: 1,
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("无法获取系统配置目录")?
            .join("autocode");
        Ok(dir.join("config.toml"))
    }

    /// 获取日志目录
    #[allow(dead_code)]
    pub fn log_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("无法获取系统配置目录")?
            .join("autocode")
            .join("logs");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        // 验证延迟时间范围（100ms - 10000ms）
        if self.autofill_detect_delay_ms < 100 || self.autofill_detect_delay_ms > 10000 {
            anyhow::bail!(
                "autofill_detect_delay_ms 必须在 100-10000 之间，当前值: {}",
                self.autofill_detect_delay_ms
            );
        }

        // 验证正则表达式的有效性
        for (idx, pattern) in self.verification_patterns.iter().enumerate() {
            regex::Regex::new(pattern).with_context(|| {
                format!("正则表达式 #{} 无效: {}", idx + 1, pattern)
            })?;
        }

        // 验证关键词不为空
        if self.verification_keywords.is_empty() {
            anyhow::bail!("verification_keywords 不能为空");
        }

        Ok(())
    }

    /// 从磁盘加载配置
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            log::info!("创建了默认配置文件: {:?}", path);
            return Ok(config);
        }

        let content =
            fs::read_to_string(&path).with_context(|| format!("读取配置文件失败: {:?}", path))?;

        let config: Self =
            toml::from_str(&content).with_context(|| "解析配置文件失败，使用默认配置")?;

        // 验证配置
        if let Err(e) = config.validate() {
            log::warn!("配置验证失败: {}，使用默认配置", e);
            return Ok(Self::default());
        }

        Ok(config)
    }

    /// 保存配置到磁盘
    pub fn save(&self) -> Result<()> {
        // 保存前先验证
        self.validate()?;

        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}

/// 线程安全的全局配置
pub type SharedConfig = Arc<RwLock<AppConfig>>;

/// 创建共享配置
pub fn load_shared_config() -> Result<SharedConfig> {
    let config = AppConfig::load()?;
    Ok(Arc::new(RwLock::new(config)))
}
