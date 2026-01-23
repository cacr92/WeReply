pub struct Config {
    pub deepseek_model: String,
    pub suggestion_count: u32,
    pub context_max_messages: u32,
    pub context_max_chars: u32,
    pub poll_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deepseek_model: "deepseek-chat".to_string(),
            suggestion_count: 3,
            context_max_messages: 10,
            context_max_chars: 2000,
            poll_interval_ms: 800,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.deepseek_model, "deepseek-chat");
        assert_eq!(cfg.suggestion_count, 3);
        assert_eq!(cfg.context_max_messages, 10);
        assert_eq!(cfg.context_max_chars, 2000);
        assert_eq!(cfg.poll_interval_ms, 800);
    }
}
