# 配置说明

## 优先级
默认值 < 配置文件 < 环境变量
- UI 修改会写入配置文件
- API Key 仅存系统安全存储，不写入配置文件

## 环境变量
- DEEPSEEK_API_KEY: API 密钥
- DEEPSEEK_BASE_URL: 自定义 API 入口（可选）
- DEEPSEEK_MODEL: 默认模型（默认 deepseek-chat）
- DEEPSEEK_TIMEOUT_MS: 请求超时
- DEEPSEEK_MAX_RETRIES: 最大重试次数
- DEEPSEEK_TEMPERATURE: 生成温度
- DEEPSEEK_TOP_P: 采样 top_p
- SUGGESTION_COUNT: 建议数量
- CONTEXT_MAX_MESSAGES: 上下文最大消息数
- CONTEXT_MAX_CHARS: 上下文最大字符数
- POLL_INTERVAL_MS: 监听轮询间隔
- LOG_LEVEL: 日志级别（trace/debug/info/warn/error）
- LOG_TO_FILE: 是否写入文件

## 默认值
- DEEPSEEK_BASE_URL = https://api.deepseek.com
- DEEPSEEK_MODEL = deepseek-chat
- DEEPSEEK_TIMEOUT_MS = 12000
- DEEPSEEK_MAX_RETRIES = 2
- DEEPSEEK_TEMPERATURE = 0.7
- DEEPSEEK_TOP_P = 1.0
- SUGGESTION_COUNT = 3
- CONTEXT_MAX_MESSAGES = 10
- CONTEXT_MAX_CHARS = 2000
- POLL_INTERVAL_MS = 800
- LOG_LEVEL = info
- LOG_TO_FILE = false
