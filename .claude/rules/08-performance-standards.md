# 性能优化规范 - WeReply

## 桌面应用性能特点

本项目是 Tauri 桌面应用，性能优化重点：
- ✅ 重点：低延迟响应、内存使用、IPC 通信效率、DeepSeek API 响应时间
- ✅ 优势：无网络延迟、本地计算、直接系统访问
- ⚠️ 注意：消息监听延迟、Agent 通信开销、窗口跟随性能

---

## 一、Rust 后端性能优化

### 1. 并发处理

使用 Tokio 异步运行时优化并发：

**✓ 正确示例**：
```rust
use tokio::task;

pub async fn process_multiple_messages(
    messages: Vec<String>,
) -> Vec<Result<Suggestion>> {
    // 并发处理多个消息
    let mut tasks = Vec::new();

    for message in messages {
        let task = task::spawn(async move {
            generate_suggestion_for_message(message).await
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    results
}
```

### 2. 缓存策略

使用 Moka 缓存频繁访问的数据：

**✓ 正确示例**：
```rust
use moka::future::Cache;
use std::time::Duration;

pub struct SuggestionCache {
    cache: Cache<String, Vec<Suggestion>>,
}

impl SuggestionCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100)  // 最多缓存 100 条建议
                .time_to_live(Duration::from_secs(300))  // 5 分钟过期
                .build(),
        }
    }

    pub async fn get_or_generate(
        &self,
        context_key: String,
        generator: impl Future<Output = Result<Vec<Suggestion>>>,
    ) -> Result<Vec<Suggestion>> {
        self.cache
            .try_get_with(context_key, generator)
            .await
            .map_err(|e| anyhow!("缓存获取失败: {}", e))
    }

    pub fn invalidate(&self, key: &str) {
        self.cache.invalidate(key);
    }
}
```

**缓存策略**：
- **上下文建议**：缓存 5 分钟（相同上下文可能重复使用）
- **用户配置**：缓存 1 小时（变化不频繁）
- **API 密钥状态**：不缓存（安全敏感）

### 3. 避免不必要的克隆

**✓ 正确示例**：
```rust
// 使用引用
pub fn calculate_context_hash(messages: &[String]) -> String {
    messages.iter().fold(String::new(), |acc, msg| {
        format!("{}{}", acc, msg)
    })
}

// 移动所有权（如果不再需要原数据）
pub fn process_suggestions(suggestions: Vec<Suggestion>) -> Vec<ProcessedSuggestion> {
    suggestions.into_iter()
        .map(|s| process_suggestion(s))
        .collect()
}
```

**✗ 错误示例**：
```rust
// 不必要的克隆
pub fn calculate_context_hash(messages: Vec<String>) -> String {
    let cloned = messages.clone();  // ✗ 不必要
    cloned.iter().fold(String::new(), |acc, msg| {
        format!("{}{}", acc, msg)
    })
}
```

### 4. IPC 通信优化

**批量处理消息**：
```rust
pub async fn batch_process_agent_messages(
    agent: &mut AgentConnection,
    max_batch_size: usize,
) -> Result<Vec<AgentMessage>> {
    let mut messages = Vec::new();

    // 批量读取消息
    while messages.len() < max_batch_size {
        match tokio::time::timeout(
            Duration::from_millis(10),
            agent.receive_message()
        ).await {
            Ok(Ok(msg)) => messages.push(msg),
            _ => break,
        }
    }

    Ok(messages)
}
```

**消息压缩**（可选）：
```rust
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

pub fn compress_message(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress_message(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
```

### 5. DeepSeek API 调用优化

**连接池复用**：
```rust
use reqwest::Client;
use std::sync::Arc;

pub struct DeepSeekService {
    client: Arc<Client>,  // 复用 HTTP 客户端
    api_key: String,
}

impl DeepSeekService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .pool_max_idle_per_host(10)  // 连接池配置
                    .timeout(Duration::from_secs(30))
                    .build()
                    .unwrap()
            ),
            api_key,
        }
    }

    pub async fn generate_suggestions(
        &self,
        context: Vec<String>,
        style: SuggestionStyle,
    ) -> Result<Vec<Suggestion>> {
        let request_body = json!({
            "model": "deepseek-chat",
            "messages": [
                {
                    "role": "system",
                    "content": format!("生成{}风格的回复建议", style.to_string())
                },
                {
                    "role": "user",
                    "content": context.join("\n")
                }
            ],
            "n": 3,
            "temperature": 0.7,
        });

        let response = self.client
            .post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        // 解析响应
        let result: serde_json::Value = response.json().await?;
        let suggestions = result["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|choice| {
                Suggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: choice["message"]["content"].as_str().unwrap().to_string(),
                    style: style.clone(),
                    confidence: 0.9,
                }
            })
            .collect();

        Ok(suggestions)
    }
}
```

**并发限制**（防止过载）：
```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct RateLimitedDeepSeekService {
    service: DeepSeekService,
    semaphore: Arc<Semaphore>,
}

impl RateLimitedDeepSeekService {
    pub fn new(service: DeepSeekService, max_concurrent: usize) -> Self {
        Self {
            service,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn generate_suggestions(
        &self,
        context: Vec<String>,
        style: SuggestionStyle,
    ) -> Result<Vec<Suggestion>> {
        let _permit = self.semaphore.acquire().await?;
        self.service.generate_suggestions(context, style).await
    }
}
```

---

## 二、React 前端性能优化

### 1. 使用 memo 避免不必要的重渲染

```typescript
import { memo } from 'react';

interface SuggestionItemProps {
  suggestion: Suggestion;
  onSelect: (id: string) => void;
}

// ✓ 使用 memo 包装纯组件
export const SuggestionItem = memo<SuggestionItemProps>(({ suggestion, onSelect }) => {
  return (
    <div className="suggestion-item" onClick={() => onSelect(suggestion.id)}>
      <p>{suggestion.content}</p>
      <span className="confidence">{(suggestion.confidence * 100).toFixed(0)}%</span>
    </div>
  );
});

SuggestionItem.displayName = 'SuggestionItem';
```

### 2. 使用 useCallback 和 useMemo

```typescript
import { useCallback, useMemo } from 'react';

export const AssistantPanel: React.FC = () => {
  const { suggestions, loading } = useSuggestions();

  // ✓ 使用 useMemo 缓存计算结果
  const sortedSuggestions = useMemo(() => {
    return [...suggestions].sort((a, b) => b.confidence - a.confidence);
  }, [suggestions]);

  const averageConfidence = useMemo(() => {
    if (suggestions.length === 0) return 0;
    return suggestions.reduce((sum, s) => sum + s.confidence, 0) / suggestions.length;
  }, [suggestions]);

  // ✓ 使用 useCallback 缓存回调函数
  const handleSelect = useCallback((id: string) => {
    const suggestion = suggestions.find(s => s.id === id);
    if (suggestion) {
      commands.writeToWeChatInput(suggestion.content);
    }
  }, [suggestions]);

  const handleEdit = useCallback((id: string, newContent: string) => {
    // 编辑逻辑
  }, []);

  return (
    <div className="assistant-panel">
      <p>平均置信度: {(averageConfidence * 100).toFixed(0)}%</p>
      {sortedSuggestions.map(s => (
        <SuggestionItem
          key={s.id}
          suggestion={s}
          onSelect={handleSelect}
        />
      ))}
    </div>
  );
};
```

### 3. 虚拟滚动（大列表优化）

对于历史消息列表，使用虚拟滚动：

```typescript
import { FixedSizeList } from 'react-window';

interface VirtualizedMessageListProps {
  messages: WeChatMessage[];
}

export const VirtualizedMessageList: React.FC<VirtualizedMessageListProps> = ({
  messages,
}) => {
  const Row = ({ index, style }: any) => {
    const message = messages[index];
    return (
      <div style={style} className="message-row">
        <span className="sender">{message.sender}</span>: {message.content}
      </div>
    );
  };

  return (
    <FixedSizeList
      height={400}
      itemCount={messages.length}
      itemSize={50}
      width="100%"
    >
      {Row}
    </FixedSizeList>
  );
};
```

### 4. 防抖和节流

**防抖搜索**：
```typescript
import { useMemo, useState } from 'react';
import { debounce } from 'lodash-es';

export const ConfigSearch: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [results, setResults] = useState<Config[]>([]);

  // 防抖搜索
  const debouncedSearch = useMemo(
    () =>
      debounce(async (term: string) => {
        if (term.length === 0) {
          setResults([]);
          return;
        }
        const result = await commands.searchConfigs(term);
        if (result.success) {
          setResults(result.data);
        }
      }, 300),
    []
  );

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSearchTerm(value);
    debouncedSearch(value);
  };

  return <Input value={searchTerm} onChange={handleSearch} placeholder="搜索配置..." />;
};
```

**节流窗口跟随**：
```typescript
import { useEffect } from 'react';
import { throttle } from 'lodash-es';

export const useWindowFollow = (enabled: boolean) => {
  useEffect(() => {
    if (!enabled) return;

    const updatePosition = throttle(async () => {
      try {
        const result = await commands.getWeChatWindowPosition();
        if (result.success && result.data) {
          const { x, y, width } = result.data;
          const appWindow = getCurrentWindow();
          await appWindow.setPosition({
            x: x + width + 10,  // 微信窗口右侧
            y: y,
          });
        }
      } catch (error) {
        console.error('窗口跟随失败:', error);
      }
    }, 500);  // 每 500ms 最多执行一次

    const interval = setInterval(updatePosition, 500);

    return () => {
      clearInterval(interval);
      updatePosition.cancel();
    };
  }, [enabled]);
};
```

### 5. 懒加载和代码分割

```typescript
import { lazy, Suspense } from 'react';
import { Spin } from 'antd';

// 懒加载配置对话框
const ConfigDialog = lazy(() => import('./ConfigDialog'));
const HistoryPanel = lazy(() => import('./HistoryPanel'));

export const App: React.FC = () => {
  const [showConfig, setShowConfig] = useState(false);
  const [showHistory, setShowHistory] = useState(false);

  return (
    <div>
      {showConfig && (
        <Suspense fallback={<Spin size="large" />}>
          <ConfigDialog onClose={() => setShowConfig(false)} />
        </Suspense>
      )}

      {showHistory && (
        <Suspense fallback={<Spin size="large" />}>
          <HistoryPanel onClose={() => setShowHistory(false)} />
        </Suspense>
      )}
    </div>
  );
};
```

---

## 三、微信监听性能优化

### 1. 消息去重

```rust
use std::collections::HashSet;

pub struct MessageDeduplicator {
    seen_hashes: HashSet<u64>,
    max_size: usize,
}

impl MessageDeduplicator {
    pub fn new(max_size: usize) -> Self {
        Self {
            seen_hashes: HashSet::new(),
            max_size,
        }
    }

    pub fn is_duplicate(&mut self, message: &str) -> bool {
        let hash = self.calculate_hash(message);

        if self.seen_hashes.contains(&hash) {
            return true;
        }

        self.seen_hashes.insert(hash);

        // 限制大小，避免内存无限增长
        if self.seen_hashes.len() > self.max_size {
            self.seen_hashes.clear();
        }

        false
    }

    fn calculate_hash(&self, message: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        hasher.finish()
    }
}
```

### 2. 上下文裁剪

```rust
pub fn trim_context(
    messages: Vec<String>,
    max_messages: usize,
    max_total_length: usize,
) -> Vec<String> {
    // 1. 限制消息数量
    let mut trimmed: Vec<String> = messages
        .into_iter()
        .rev()
        .take(max_messages)
        .rev()
        .collect();

    // 2. 限制总长度
    let mut total_length = 0;
    let mut result = Vec::new();

    for message in trimmed.iter().rev() {
        if total_length + message.len() > max_total_length {
            break;
        }
        total_length += message.len();
        result.push(message.clone());
    }

    result.reverse();
    result
}
```

---

## 四、启动性能优化

### 1. 延迟加载非关键数据

```rust
#[tauri::command]
#[specta::specta]
pub async fn initialize_app(
    state: State<'_, AppState>,
) -> ApiResponse<AppInitData> {
    // 只加载启动必需的数据
    let api_key_exists = match ApiKeyManager::get_deepseek_api_key() {
        Ok(_) => true,
        Err(_) => false,
    };

    let config = state.config_manager.get();

    // 其他数据延迟加载
    api_ok(AppInitData {
        api_key_exists,
        config,
    })
}

// 在用户需要时才加载历史消息
#[tauri::command]
#[specta::specta]
pub async fn load_message_history(
    state: State<'_, AppState>,
) -> ApiResponse<Vec<WeChatMessage>> {
    // 延迟加载历史消息
    state.context_manager.get_history().await
}
```

### 2. 预热连接

```rust
pub async fn preheat_deepseek_connection(
    service: &DeepSeekService,
) -> Result<()> {
    // 发送一个简单请求预热连接
    let _ = service.generate_suggestions(
        vec!["test".to_string()],
        SuggestionStyle::Formal,
    ).await;

    Ok(())
}
```

---

## 五、内存优化

### 1. 及时释放大对象

```rust
pub async fn process_large_context(context: Vec<String>) -> Result<()> {
    // 分批处理
    for chunk in context.chunks(100) {
        process_chunk(chunk).await?;
        // chunk 处理完后自动释放
    }
    // context 在函数结束时释放
    Ok(())
}
```

### 2. 限制缓存大小

```rust
pub struct BoundedHistoryManager {
    messages: Vec<WeChatMessage>,
    max_size: usize,
}

impl BoundedHistoryManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_size,
        }
    }

    pub fn add_message(&mut self, message: WeChatMessage) {
        self.messages.push(message);

        // 超出限制时删除旧消息
        if self.messages.len() > self.max_size {
            self.messages.remove(0);
        }
    }

    pub fn get_recent(&self, count: usize) -> &[WeChatMessage] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }
}
```

---

## 六、性能监控

### 1. 使用 tracing 记录性能指标

```rust
use tracing::{info, instrument};
use std::time::Instant;

#[instrument(skip(self))]
pub async fn generate_suggestions(
    &self,
    context: Vec<String>,
    style: SuggestionStyle,
) -> Result<Vec<Suggestion>> {
    let start = Instant::now();

    let result = self.perform_generation(context, style).await?;

    let duration = start.elapsed();
    info!(
        duration_ms = duration.as_millis(),
        suggestion_count = result.len(),
        "建议生成完成"
    );

    Ok(result)
}
```

### 2. 前端性能监控

```typescript
export const usePerformanceMonitor = (componentName: string) => {
  useEffect(() => {
    const start = performance.now();

    return () => {
      const duration = performance.now() - start;
      if (duration > 1000) {
        console.warn(`${componentName} 渲染耗时: ${duration}ms`);
      }
    };
  }, [componentName]);
};

// 使用
export const AssistantPanel: React.FC = () => {
  usePerformanceMonitor('AssistantPanel');

  return (
    // ...
  );
};
```

---

## 七、性能基准

### WeReply 目标性能指标
- **消息监听延迟**：< 500ms（从微信消息到 Rust 接收）
- **Agent 通信延迟**：< 100ms（IPC 往返）
- **DeepSeek API 响应**：< 3s（建议生成）
- **UI 响应时间**：< 100ms（用户操作到界面更新）
- **窗口跟随延迟**：< 200ms（微信窗口移动到助手窗口跟随）
- **启动时间**：< 2s（应用启动到可用）

### 性能测试
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_suggestion_generation_performance() {
        let service = setup_test_service().await;
        let context = vec!["你好".to_string(), "最近怎么样".to_string()];

        let start = Instant::now();
        let result = service.generate_suggestions(context, SuggestionStyle::Friendly).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_secs() < 5, "建议生成耗时超过 5 秒");
    }

    #[tokio::test]
    async fn test_ipc_communication_performance() {
        let mut agent = setup_test_agent().await;
        let message = AgentMessage::MessageNew {
            content: "test".to_string(),
            sender: "user".to_string(),
            timestamp: 0,
        };

        let start = Instant::now();
        agent.send_message(&message).await.unwrap();
        let response = agent.receive_message().await.unwrap();
        let duration = start.elapsed();

        assert!(duration.as_millis() < 200, "IPC 通信耗时超过 200ms");
    }
}
```

---

## 八、常见性能陷阱

### 1. 在循环中执行异步操作
**✗ 错误**：
```rust
for id in ids {
    let item = repository.get(id).await?;  // ✗ 串行执行
}
```

**✓ 正确**：
```rust
use futures::future::join_all;

let futures = ids.iter().map(|id| repository.get(*id));
let items = join_all(futures).await;  // ✓ 并行执行
```

### 2. 过度使用 clone
**✗ 错误**：
```rust
let cloned = data.clone();  // ✗ 不必要的克隆
process(cloned);
```

**✓ 正确**：
```rust
process(&data);  // ✓ 使用引用
```

### 3. 不使用缓存
**✗ 错误**：
```rust
// 每次都调用 API
pub async fn get_suggestions(&self, context: Vec<String>) -> Result<Vec<Suggestion>> {
    self.deepseek_service.generate_suggestions(context, style).await
}
```

**✓ 正确**：
```rust
// 使用缓存
pub async fn get_suggestions(&self, context: Vec<String>) -> Result<Vec<Suggestion>> {
    let cache_key = calculate_context_hash(&context);
    self.cache.get_or_generate(cache_key, async {
        self.deepseek_service.generate_suggestions(context, style).await
    }).await
}
```

### 4. 组件过度渲染
**✗ 错误**：
```typescript
// 每次父组件渲染都会创建新函数
<SuggestionItem onClick={() => handleClick()} />
```

**✓ 正确**：
```typescript
const handleClick = useCallback(() => { }, []);
<SuggestionItem onClick={handleClick} />
```

### 5. 阻塞异步运行时
**✗ 错误**：
```rust
pub async fn process_data() {
    std::thread::sleep(Duration::from_secs(1));  // ✗ 阻塞整个运行时
}
```

**✓ 正确**：
```rust
pub async fn process_data() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // ✓ 异步睡眠
}
```

---

## WeReply 特定性能优化

### 1. 微信消息监听优化
- 使用消息去重减少重复处理
- 批量处理消息减少 IPC 开销
- 限制上下文大小减少内存使用

### 2. DeepSeek API 调用优化
- 复用 HTTP 连接池
- 实现请求缓存（相同上下文）
- 并发限制（防止过载）
- 设置合理超时

### 3. 窗口跟随优化
- 使用节流减少更新频率
- 仅在窗口移动时更新
- 使用硬件加速（如可用）

---

## 参考资源

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [React Performance Optimization](https://react.dev/learn/render-and-commit)
- [Tokio Performance Guide](https://tokio.rs/tokio/topics/performance)
- [Tauri Performance Best Practices](https://tauri.app/v1/guides/building/performance/)
- [Moka Cache Documentation](https://docs.rs/moka/)
