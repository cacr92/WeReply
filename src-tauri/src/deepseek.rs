use crate::types::{Config, Suggestion, SuggestionStyle};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

const SYSTEM_PROMPT: &str = "你是回复建议助手。请根据对话内容生成 3 条回复建议，分别为正式、\
中性、轻松风格。返回 JSON 数组，每个元素包含 style(formal|neutral|casual) 与 text。";
const VALIDATION_PROMPT: &str = "请回复一个简短确认词，用于验证连接。";
const DEFAULT_MODELS: [&str; 2] = ["deepseek-chat", "deepseek-reasoner"];

fn cap_timeout_ms(timeout_ms: u64) -> u64 {
    timeout_ms.clamp(2_000, 8_000)
}

pub fn build_request(user_input: &str, model: &str) -> Value {
    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_input}
        ]
    })
}

pub fn build_validation_request(user_input: &str, model: &str) -> Value {
    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": VALIDATION_PROMPT},
            {"role": "user", "content": user_input}
        ]
    })
}

pub fn is_supported_model(model: &str) -> bool {
    DEFAULT_MODELS.iter().any(|item| item == &model)
}

fn default_models() -> Vec<String> {
    DEFAULT_MODELS.iter().map(|model| (*model).to_string()).collect()
}

fn normalize_models(models: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for model in DEFAULT_MODELS {
        if models.iter().any(|item| item == model) {
            normalized.push(model.to_string());
        }
    }
    if normalized.is_empty() {
        default_models()
    } else {
        normalized
    }
}

fn build_chat_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

fn build_models_url(base_url: &str) -> String {
    format!("{}/models", base_url.trim_end_matches('/'))
}

fn parse_models(raw: &str) -> Result<Vec<String>> {
    let value: Value = serde_json::from_str(raw).context("响应 JSON 解析失败")?;
    let Some(items) = value["data"].as_array() else {
        return Ok(Vec::new());
    };
    let mut models = Vec::new();
    for item in items {
        if let Some(id) = item["id"].as_str() {
            models.push(id.to_string());
        }
    }
    Ok(models)
}

pub async fn validate_api_key(config: &Config, api_key: &str) -> Result<()> {
    let timeout_ms = cap_timeout_ms(config.timeout_ms);
    info!("开始验证 DeepSeek API 密钥");
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .context("创建 HTTP 客户端失败")?;
    let url = build_chat_url(&config.base_url);
    let request = build_validation_request("ping", &config.deepseek_model);

    let response = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        client
            .post(url)
            .bearer_auth(api_key)
            .json(&request)
            .send(),
    )
    .await
    .context("DeepSeek 连接超时")?
    .context("DeepSeek 连接失败")?;
    let status = response.status();
    let raw = response.text().await.context("读取 DeepSeek 响应失败")?;
    if !status.is_success() {
        let detail: String = raw.chars().take(200).collect();
        warn!("DeepSeek 验证失败: {}", status);
        anyhow::bail!("DeepSeek 验证失败: {} {}", status, detail);
    }
    info!("DeepSeek 验证成功");
    Ok(())
}

pub async fn generate_suggestions(
    config: &Config,
    api_key: Option<String>,
    context_messages: &[String],
) -> Result<Vec<Suggestion>> {
    let prompt = build_prompt(context_messages);
    let Some(key) = api_key else {
        return Ok(fallback_suggestions(&prompt));
    };

    let client = Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()
        .context("创建 HTTP 客户端失败")?;
    let url = build_chat_url(&config.base_url);
    let request = build_request(&prompt, &config.deepseek_model);

    let response = client
        .post(url)
        .bearer_auth(key)
        .json(&request)
        .send()
        .await
        .context("DeepSeek 请求失败")?;
    let status = response.status();
    let raw = response.text().await.context("读取 DeepSeek 响应失败")?;

    if !status.is_success() {
        warn!("DeepSeek 返回错误: {}", status);
        return Ok(fallback_suggestions(&prompt));
    }

    match parse_response(&raw) {
        Ok(suggestions) if !suggestions.is_empty() => Ok(suggestions),
        Ok(_) => Ok(fallback_suggestions(&prompt)),
        Err(err) => {
            warn!("解析 DeepSeek 响应失败: {}", err);
            Ok(fallback_suggestions(&prompt))
        }
    }
}

pub async fn list_models(config: &Config, api_key: &str) -> Result<Vec<String>> {
    let timeout_ms = cap_timeout_ms(config.timeout_ms);
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .context("创建 HTTP 客户端失败")?;
    let url = build_models_url(&config.base_url);

    let response = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        client.get(url).bearer_auth(api_key).send(),
    )
    .await
    .context("DeepSeek 连接超时")?
    .context("DeepSeek 连接失败")?;
    let status = response.status();
    let raw = response.text().await.context("读取 DeepSeek 响应失败")?;
    if !status.is_success() {
        let detail: String = raw.chars().take(200).collect();
        warn!("DeepSeek 拉取模型失败: {}", status);
        anyhow::bail!("DeepSeek 拉取模型失败: {} {}", status, detail);
    }
    let parsed = parse_models(&raw)?;
    Ok(normalize_models(parsed))
}

fn build_prompt(context_messages: &[String]) -> String {
    if context_messages.is_empty() {
        return "用户未提供上下文，请生成礼貌的确认回复。".to_string();
    }
    let mut lines = Vec::new();
    for (idx, message) in context_messages.iter().enumerate() {
        lines.push(format!("{}: {}", idx + 1, message));
    }
    format!("最近对话：\n{}\n请生成 3 条回复建议。", lines.join("\n"))
}

fn parse_response(raw: &str) -> Result<Vec<Suggestion>> {
    let json_value: Value = serde_json::from_str(raw).context("响应 JSON 解析失败")?;
    let content = json_value["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .trim();
    if content.is_empty() {
        return Ok(Vec::new());
    }

    let cleaned = content
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();
    if let Ok(items) = serde_json::from_str::<Vec<Value>>(cleaned) {
        let mut suggestions = Vec::new();
        for item in items {
            let style = match item["style"].as_str().unwrap_or("neutral") {
                "formal" => SuggestionStyle::Formal,
                "casual" => SuggestionStyle::Casual,
                _ => SuggestionStyle::Neutral,
            };
            let text = item["text"].as_str().unwrap_or("").trim().to_string();
            if !text.is_empty() {
                suggestions.push(Suggestion {
                    id: Uuid::new_v4().to_string(),
                    style,
                    text,
                });
            }
        }
        return Ok(suggestions);
    }

    info!("DeepSeek 返回非 JSON 结构，使用降级解析");
    let suggestions = cleaned
        .lines()
        .filter_map(|line| {
            let text = line.trim_matches(['-', ' ']).trim();
            if text.is_empty() {
                None
            } else {
                Some(Suggestion {
                    id: Uuid::new_v4().to_string(),
                    style: SuggestionStyle::Neutral,
                    text: text.to_string(),
                })
            }
        })
        .collect();
    Ok(suggestions)
}

fn fallback_suggestions(prompt: &str) -> Vec<Suggestion> {
    let summary = summarize_text(prompt);
    vec![
        Suggestion {
            id: Uuid::new_v4().to_string(),
            style: SuggestionStyle::Formal,
            text: format!("好的，我了解了：{}，稍后给您回复。", summary),
        },
        Suggestion {
            id: Uuid::new_v4().to_string(),
            style: SuggestionStyle::Neutral,
            text: format!("收到，我看看 {} 再回复你。", summary),
        },
        Suggestion {
            id: Uuid::new_v4().to_string(),
            style: SuggestionStyle::Casual,
            text: format!("好哒～{} 我等下回你。", summary),
        },
    ]
}

fn summarize_text(text: &str) -> String {
    let trimmed: String = text.chars().take(20).collect();
    if trimmed.is_empty() {
        "消息".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_payload_is_minimal() {
        let req = build_request("hi", "deepseek-chat");
        assert_eq!(req["model"], "deepseek-chat");
        assert_eq!(req["messages"].as_array().unwrap().len(), 2);
        assert!(req.get("temperature").is_none());
        assert!(req.get("top_p").is_none());
        assert!(req.get("n").is_none());
    }

    #[test]
    fn fallback_has_three_styles() {
        let suggestions = fallback_suggestions("hi");
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn build_validation_request_is_minimal() {
        let req = build_validation_request("ping", "deepseek-chat");
        assert_eq!(req["model"], "deepseek-chat");
        assert!(req.get("temperature").is_none());
        assert!(req.get("top_p").is_none());
        assert!(req.get("n").is_none());
    }

    #[test]
    fn normalize_timeout_caps() {
        assert_eq!(cap_timeout_ms(12_000), 8_000);
    }

    #[test]
    fn build_chat_url_trims_slash() {
        let url = build_chat_url("https://api.deepseek.com/");
        assert_eq!(url, "https://api.deepseek.com/chat/completions");
    }

    #[test]
    fn normalize_models_filters_and_fallbacks() {
        let models = normalize_models(vec!["x".to_string()]);
        assert_eq!(models, vec!["deepseek-chat", "deepseek-reasoner"]);
    }
}
