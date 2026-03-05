use log;
use regex::Regex;

use crate::config::AppConfig;

/// 提取到的验证码
#[derive(Debug, Clone)]
pub struct VerificationCode {
    pub code: String,
    pub source: String,
    pub confidence: f32,
}

/// 提取策略 trait
trait ExtractionStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn extract(
        &self,
        text: &str,
        sender: Option<&str>,
        config: &AppConfig,
    ) -> Option<VerificationCode>;
    fn confidence(&self) -> f32;
}

/// 策略 1：模板正则匹配（高置信度）
struct TemplateStrategy;

impl ExtractionStrategy for TemplateStrategy {
    fn name(&self) -> &str {
        "模板匹配"
    }

    fn extract(
        &self,
        text: &str,
        _sender: Option<&str>,
        config: &AppConfig,
    ) -> Option<VerificationCode> {
        for pattern_str in &config.verification_patterns {
            match Regex::new(pattern_str) {
                Ok(re) => {
                    if let Some(caps) = re.captures(text) {
                        if let Some(code_match) = caps.get(1) {
                            let code = code_match.as_str().to_string();
                            // 验证码至少包含一个数字
                            if code.chars().any(|c| c.is_ascii_digit()) {
                                log::debug!("模板匹配 '{}' 提取到验证码: {}", pattern_str, code);
                                return Some(VerificationCode {
                                    code,
                                    source: "模板匹配".into(),
                                    confidence: self.confidence(),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("正则表达式无效 '{}': {}", pattern_str, e);
                }
            }
        }
        None
    }

    fn confidence(&self) -> f32 {
        0.95
    }
}

/// 策略 2：发件人白名单 + 宽松规则（高置信度）
struct SenderWhitelistStrategy;

impl ExtractionStrategy for SenderWhitelistStrategy {
    fn name(&self) -> &str {
        "发件人白名单"
    }

    fn extract(
        &self,
        text: &str,
        sender: Option<&str>,
        config: &AppConfig,
    ) -> Option<VerificationCode> {
        let sender = sender?;
        let sender_lower = sender.to_lowercase();

        let is_known = config
            .known_2fa_senders
            .iter()
            .any(|s| sender_lower.contains(&s.to_lowercase()));

        if !is_known {
            return None;
        }

        // 对已知发件人使用更宽松的数字提取
        let re = Regex::new(r"\b(\d{4,8})\b").ok()?;
        // 找到所有数字候选
        let candidates: Vec<&str> = re.find_iter(text).map(|m| m.as_str()).collect();

        if candidates.is_empty() {
            return None;
        }

        // 优先选择 6 位数字（最常见的验证码长度）
        let code = candidates
            .iter()
            .find(|c| c.len() == 6)
            .or_else(|| candidates.first())
            .copied()?;

        log::debug!("发件人白名单 '{}' 提取到验证码: {}", sender, code);
        Some(VerificationCode {
            code: code.to_string(),
            source: format!("已知发件人: {}", sender),
            confidence: self.confidence(),
        })
    }

    fn confidence(&self) -> f32 {
        0.90
    }
}

/// 策略 3：关键词近距离搜索（中置信度）
struct KeywordProximityStrategy;

impl ExtractionStrategy for KeywordProximityStrategy {
    fn name(&self) -> &str {
        "关键词近邻"
    }

    fn extract(
        &self,
        text: &str,
        _sender: Option<&str>,
        config: &AppConfig,
    ) -> Option<VerificationCode> {
        let text_lower = text.to_lowercase();

        for keyword in &config.verification_keywords {
            let keyword_lower = keyword.to_lowercase();
            if let Some(pos) = text_lower.find(&keyword_lower) {
                // 在关键词前后 80 字符范围内搜索
                let byte_start = text
                    .char_indices()
                    .rev()
                    .find(|(i, _)| *i <= pos.saturating_sub(80))
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let byte_end = text
                    .char_indices()
                    .find(|(i, _)| *i >= (pos + keyword.len() + 80).min(text.len()))
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());

                let window = &text[byte_start..byte_end];

                // 在窗口内搜索数字码
                let re = Regex::new(r"\b(\d{4,8})\b").ok()?;
                let candidates: Vec<regex::Match> = re.find_iter(window).collect();

                if let Some(best) = candidates.iter().min_by_key(|m| {
                    // 距离关键词最近的
                    let m_center = m.start() + m.len() / 2;
                    let kw_pos_in_window = if pos >= byte_start {
                        pos - byte_start
                    } else {
                        0
                    };
                    (m_center as i64 - kw_pos_in_window as i64).unsigned_abs()
                }) {
                    let code = best.as_str().to_string();
                    log::debug!("关键词 '{}' 近邻提取到验证码: {}", keyword, code);
                    return Some(VerificationCode {
                        code,
                        source: format!("关键词: {}", keyword),
                        confidence: self.confidence(),
                    });
                }
            }
        }
        None
    }

    fn confidence(&self) -> f32 {
        0.75
    }
}

/// 策略 4：HTML 结构提取（特殊格式邮件）
struct HtmlStructureStrategy;

impl ExtractionStrategy for HtmlStructureStrategy {
    fn name(&self) -> &str {
        "HTML结构"
    }

    fn extract(
        &self,
        text: &str,
        _sender: Option<&str>,
        _config: &AppConfig,
    ) -> Option<VerificationCode> {
        // 检测是否是 HTML 内容
        if !text.contains('<') || !text.contains('>') {
            return None;
        }

        // 提取大字号/加粗元素中的独立数字
        let patterns = [
            // font-size 大于 20px 的元素中的数字
            r#"font-size:\s*(?:2[0-9]|[3-9][0-9]|[1-9]\d{2,})px[^>]*>\s*(\d{4,8})\s*<"#,
            // font-weight bold 的元素中的数字
            r#"font-weight:\s*(?:bold|[6-9]00)[^>]*>\s*(\d{4,8})\s*<"#,
            // 独立 <strong>/<b> 标签中的纯数字
            r#"<(?:strong|b)>\s*(\d{4,8})\s*</(?:strong|b)>"#,
            // 带有 code/otp/pin class 的元素
            r#"class="[^"]*(?:code|otp|pin|verification)[^"]*"[^>]*>\s*(\d{4,8})\s*<"#,
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(code_match) = caps.get(1) {
                        let code = code_match.as_str().to_string();
                        log::debug!("HTML 结构提取到验证码: {}", code);
                        return Some(VerificationCode {
                            code,
                            source: "HTML结构".into(),
                            confidence: self.confidence(),
                        });
                    }
                }
            }
        }
        None
    }

    fn confidence(&self) -> f32 {
        0.85
    }
}

/// 验证码提取引擎 — 多策略组合
pub struct CodeExtractor {
    strategies: Vec<Box<dyn ExtractionStrategy>>,
}

impl CodeExtractor {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(TemplateStrategy),
                Box::new(SenderWhitelistStrategy),
                Box::new(HtmlStructureStrategy),
                Box::new(KeywordProximityStrategy),
            ],
        }
    }

    /// 从文本中提取验证码，返回置信度最高的结果
    pub fn extract(
        &self,
        text: &str,
        sender: Option<&str>,
        config: &AppConfig,
    ) -> Option<VerificationCode> {
        if text.is_empty() {
            return None;
        }

        let mut best: Option<VerificationCode> = None;

        for strategy in &self.strategies {
            if let Some(code) = strategy.extract(text, sender, config) {
                log::info!(
                    "[{}] 提取到验证码: {} (置信度: {:.0}%)",
                    strategy.name(),
                    code.code,
                    code.confidence * 100.0
                );

                match &best {
                    Some(current) if current.confidence >= code.confidence => {}
                    _ => best = Some(code),
                }
            }
        }

        if let Some(ref code) = best {
            log::info!("最终选择验证码: {} (来源: {})", code.code, code.source);
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn test_chinese_template() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        // 中文验证码
        let result = extractor.extract("您的验证码是 384756，请在5分钟内使用", None, &config);
        assert_eq!(result.unwrap().code, "384756");

        let result = extractor.extract("【淘宝】验证码：192837", None, &config);
        assert_eq!(result.unwrap().code, "192837");

        let result = extractor.extract("动态密码为 4829，请勿泄露", None, &config);
        assert_eq!(result.unwrap().code, "4829");
    }

    #[test]
    fn test_english_template() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        let result = extractor.extract("Your verification code is 847291", None, &config);
        assert_eq!(result.unwrap().code, "847291");

        let result = extractor.extract("Your OTP is 293847", None, &config);
        assert_eq!(result.unwrap().code, "293847");

        let result = extractor.extract("Security code: 738291", None, &config);
        assert_eq!(result.unwrap().code, "738291");
    }

    #[test]
    fn test_sender_whitelist() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        let result = extractor.extract(
            "Please use 482917 to sign in.",
            Some("account-security-noreply@accountprotection.microsoft.com"),
            &config,
        );
        assert_eq!(result.unwrap().code, "482917");
    }

    #[test]
    fn test_keyword_proximity() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        let result = extractor.extract(
            "Welcome! Your login code for the service is 918273. It expires in 10 minutes.",
            None,
            &config,
        );
        assert_eq!(result.unwrap().code, "918273");
    }

    #[test]
    fn test_no_false_positive() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        // 普通消息，不应提取出验证码
        let result = extractor.extract("今天天气真好，我们下午3点见面吧", None, &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_html_structure() {
        let extractor = CodeExtractor::new();
        let config = test_config();

        let html = r#"<div><p>Your code:</p><strong>847291</strong></div>"#;
        let result = extractor.extract(html, None, &config);
        assert_eq!(result.unwrap().code, "847291");
    }
}
