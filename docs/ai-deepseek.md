# DeepSeek API 规范

## 基础信息
- API 与 OpenAI 兼容
- Base URL: https://api.deepseek.com
- 兼容模式可使用 https://api.deepseek.com/v1（v1 与模型版本无关）
- Endpoint: POST /chat/completions
- 鉴权: Authorization: Bearer <API_KEY>

## 模型
- deepseek-chat（MVP 默认）
- deepseek-reasoner（暂不使用）

## 请求结构（最小集）
```
{
  "model": "deepseek-chat",
  "messages": [
    {"role": "system", "content": "你是回复建议助手"},
    {"role": "user", "content": "晚上一起吃饭吗？"}
  ],
  "temperature": 0.7,
  "top_p": 1.0,
  "stream": false
}
```

## 响应结构（最小集）
```
{
  "id": "...",
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "好的，几点？"
      }
    }
  ]
}
```

## 失败与重试
- 超时 12s，最多重试 2 次
- 5xx 视为可重试
- 4xx 直接失败并降级为模板回复

## 生成策略
- 3 条建议（正式/中性/轻松）
- 上下文最多 10 条或 2000 字符
- 若上下文不足，补充简短 system 提示
