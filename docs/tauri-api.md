# Tauri Commands & Events

## Commands
- `get_config() -> Config`
- `set_config(config: Config) -> bool`
- `start_listening() -> bool`
- `stop_listening() -> bool`
- `pause_listening() -> bool`
- `resume_listening() -> bool`
- `write_suggestion(chat_id: string, text: string) -> bool`
- `get_status() -> Status`

## Events
- `status.changed`
- `suggestions.updated`
- `error.raised`

## Types
### Config
```
{
  "deepseek_model": "deepseek-chat",
  "suggestion_count": 3,
  "context_max_messages": 10,
  "context_max_chars": 2000,
  "poll_interval_ms": 800,
  "temperature": 0.7,
  "top_p": 1.0
}
```

### Status
```
{
  "state": "idle" | "listening" | "generating" | "paused" | "error",
  "platform": "windows" | "macos",
  "agent_connected": true,
  "last_error": ""
}
```

### Suggestion
```
{
  "id": "sug_001",
  "style": "formal" | "neutral" | "casual",
  "text": "好的，几点？"
}
```

### SuggestionsUpdated
```
{
  "chat_id": "chat_001",
  "suggestions": [Suggestion]
}
```
