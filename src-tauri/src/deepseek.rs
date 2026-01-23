use serde_json::{json, Value};

pub fn build_request(
    user_input: &str,
    suggestion_count: u32,
    model: &str,
    temperature: f32,
    top_p: f32,
) -> Value {
    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "你是回复建议助手"},
            {"role": "user", "content": user_input}
        ],
        "temperature": temperature,
        "top_p": top_p,
        "n": suggestion_count,
        "stream": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_payload() {
        let req = build_request("hi", 3, "deepseek-chat", 0.7, 1.0);
        assert_eq!(req["model"], "deepseek-chat");
        assert_eq!(req["messages"].as_array().unwrap().len(), 2);
    }
}
