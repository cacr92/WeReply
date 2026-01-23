# React TypeScript 前端开发规范 - WeReply

## 组件命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| React 组件 | PascalCase | `AssistantPanel`, `SuggestionList`, `ConfigDialog` |
| 自定义 Hooks | camelCase (use 前缀) | `useSuggestions`, `useWeChatMonitor` |
| 工具函数 | camelCase | `formatMessage`, `validateApiKey` |
| 类型定义 | PascalCase | `Suggestion`, `WeChatMessage` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_SUGGESTIONS`, `DEFAULT_STYLE` |

## UI 与布局规范
- 所有页面必须支持响应式布局（适配桌面与小尺寸窗口）
- 所有页面必须接入项目统一的明暗主题切换系统
- 助手面板必须支持窗口跟随（跟随微信窗口位置）
- 所有文本输入必须支持实时预览和编辑

## 函数组件定义

优先使用函数组件，使用 React.FC 或显式返回类型：

**✓ 正确示例**:
```typescript
import React from 'react';
import { Button, Card } from 'antd';

interface SuggestionItemProps {
  suggestion: Suggestion;
  onSelect?: (id: string) => void;
  onEdit?: (id: string, content: string) => void;
}

export const SuggestionItem: React.FC<SuggestionItemProps> = ({
  suggestion,
  onSelect,
  onEdit,
}) => {
  const handleSelect = () => {
    onSelect?.(suggestion.id);
  };

  return (
    <Card className="suggestion-item">
      <p>{suggestion.content}</p>
      <Button onClick={handleSelect}>使用此建议</Button>
    </Card>
  );
};
```

## React Hooks 使用规范

### Hooks 规则
**必须遵守 React Hooks 规则**:
- 只在顶层调用 Hooks
- 只在 React 函数中调用 Hooks
- 使用 `useCallback` 包装传递给子组件的回调函数
- 使用 `useMemo` 优化昂贵的计算

**✓ 正确示例**:
```typescript
import { useCallback, useMemo } from 'react';

export const SuggestionList: React.FC = () => {
  const { suggestions, loading } = useSuggestions();

  // ✓ 使用 useMemo 优化计算
  const sortedSuggestions = useMemo(() => {
    return [...suggestions].sort((a, b) => b.confidence - a.confidence);
  }, [suggestions]);

  // ✓ 使用 useCallback 优化回调
  const handleSelect = useCallback((id: string) => {
    // 选择建议逻辑
  }, []);

  return (
    <div>
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

**✗ 错误示例**:
```typescript
// ✗ 在循环中使用 Hook
export const BadComponent: React.FC = () => {
  const items = [1, 2, 3];

  return (
    <>
      {items.map(item => {
        const [value, setValue] = useState(0); // ✗ 错误！
        return <div key={item}>{value}</div>;
      })}
    </>
  );
};
```

## TypeScript 类型规范

### 使用生成的类型绑定
- 优先使用 `bindings.ts` 中生成的类型
- 不要手动重复定义后端已有的类型

**✓ 正确示例**:
```typescript
import type { Suggestion, WeChatMessage, SuggestionStyle } from '../bindings';
import { commands } from '../bindings';

export const SuggestionService = {
  async generateSuggestions(
    messages: WeChatMessage[],
    style: SuggestionStyle
  ): Promise<Suggestion[]> {
    const result = await commands.generateSuggestions({
      contextMessages: messages,
      style,
    });

    if (!result.success) {
      throw new Error(result.message);
    }
    return result.data;
  }
};
```

### 类型定义规范
- 接口 (Interface) 用于描述对象形状
- 类型别名 (Type) 用于联合类型、交叉类型

**✓ 正确示例**:
```typescript
// 接口 - 描述对象结构
interface AssistantPanelProps {
  visible: boolean;
  suggestions: Suggestion[];
  onClose?: () => void;
}

// 类型别名 - 联合类型
type SuggestionStyle = 'formal' | 'friendly' | 'humorous';

// 类型别名 - 组合类型
type OptionalSuggestion = Partial<Suggestion>;
```

### 类型转换规范
**禁止使用 `as any` 进行类型转换**：

**✗ 错误示例**:
```typescript
// ✗ 使用 as any 绕过类型检查
const handleData = (data: unknown) => {
  const suggestion = data as any; // ✗ 错误！
  return suggestion.content.toUpperCase();
};
```

**✓ 正确示例**:
```typescript
// ✓ 使用类型守卫
function isSuggestion(data: unknown): data is Suggestion {
  return (
    typeof data === 'object' &&
    data !== null &&
    'content' in data &&
    'id' in data
  );
}

const handleData = (data: unknown) => {
  if (isSuggestion(data)) {
    return data.content.toUpperCase();
  }
  throw new Error('Invalid suggestion data');
};

// ✓ 使用 unknown 而非 any
const processValue = (value: unknown) => {
  if (typeof value === 'string') {
    return value;
  }
  return '';
};
```

## 状态管理规范

### 本地状态 vs 全局状态
- 组件私有状态使用 `useState`
- 跨组件共享状态使用 Context API
- 服务器状态使用 TanStack Query（可选）

**✓ 正确示例**:
```typescript
// App.tsx - 全局状态
const AssistantContext = createContext<AssistantContextType | undefined>(undefined);

export const useAssistant = () => {
  const context = useContext(AssistantContext);
  if (!context) {
    throw new Error('useAssistant must be used within AssistantProvider');
  }
  return context;
};

// 组件中使用
export const SuggestionList: React.FC = () => {
  const { suggestions, reload } = useAssistant(); // 全局状态
  const [selectedId, setSelectedId] = useState<string | null>(null); // 本地状态

  return (
    // ...
  );
};
```

## Ant Design 使用规范

### 消息提示规范
**禁止使用 `console.log`，使用 `message` 组件**:

**✓ 正确示例**:
```typescript
import { message } from 'antd';

const handleWriteToInput = async (content: string) => {
  try {
    await commands.writeToWeChatInput(content);
    message.success('已写入微信输入框');
  } catch (error) {
    message.error(`写入失败: ${error.message}`);
  }
};
```

**✗ 错误示例**:
```typescript
// ✗ 桌面应用禁止使用 console
const handleWriteToInput = async (content: string) => {
  try {
    await commands.writeToWeChatInput(content);
    console.log('写入成功'); // ✗ 错误！
  } catch (error) {
    console.error('写入失败', error); // ✗ 错误！
  }
};
```

### 表单验证规范
使用 Ant Design Form 的内置验证：

**✓ 正确示例**:
```typescript
import { Form, Input, Button } from 'antd';

export const ConfigForm: React.FC = () => {
  const [form] = Form.useForm();

  const handleSubmit = async (values: any) => {
    try {
      await commands.saveConfig(values);
      message.success('配置已保存');
    } catch (error) {
      message.error(`保存失败: ${error}`);
    }
  };

  return (
    <Form form={form} onFinish={handleSubmit}>
      <Form.Item
        name="apiKey"
        label="DeepSeek API 密钥"
        rules={[
          { required: true, message: '请输入 API 密钥' },
          { pattern: /^sk-/, message: 'API 密钥格式错误' }
        ]}
      >
        <Input.Password />
      </Form.Item>
      <Button type="primary" htmlType="submit">保存</Button>
    </Form>
  );
};
```

## Tauri 集成规范

### 命令调用规范
**禁止使用 `@tauri-apps/api` 的 `invoke`，使用生成的 `commands`**:

**✓ 正确示例**:
```typescript
import { commands } from './bindings';

// ✓ 使用生成的类型安全命令
const result = await commands.generateSuggestions({
  contextMessages: ['你好', '最近怎么样'],
  style: 'friendly',
});

if (result.success) {
  console.log(result.data);
}
```

**✗ 错误示例**:
```typescript
import { invoke } from '@tauri-apps/api/core';

// ✗ 不要使用原始 invoke
const result = await invoke('generate_suggestions', {
  request: { contextMessages: [], style: 'friendly' }
});
```

### 事件监听规范
使用 `@tauri-apps/api` 的事件系统：

**✓ 正确示例**:
```typescript
import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';

export const useWeChatMonitor = (onNewMessage: (message: string) => void) => {
  useEffect(() => {
    const unlisten = listen<string>('wechat-message-new', (event) => {
      onNewMessage(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [onNewMessage]);
};
```

## 性能优化规范

### 避免不必要的重新渲染
使用 `React.memo` 包装纯组件：

**✓ 正确示例**:
```typescript
import React, { memo } from 'react';

interface SuggestionItemProps {
  suggestion: Suggestion;
  onSelect: (id: string) => void;
}

// ✓ 使用 memo 避免不必要的重新渲染
export const SuggestionItem = memo<SuggestionItemProps>(({ suggestion, onSelect }) => {
  return (
    <div onClick={() => onSelect(suggestion.id)}>
      <p>{suggestion.content}</p>
    </div>
  );
});

SuggestionItem.displayName = 'SuggestionItem';
```

### 懒加载策略
按需加载组件：

**✓ 正确示例**:
```typescript
import { lazy, Suspense } from 'react';
import { Spin } from 'antd';

// 懒加载配置对话框
const ConfigDialog = lazy(() => import('./ConfigDialog'));

export const App: React.FC = () => {
  const [showConfig, setShowConfig] = useState(false);

  return (
    <div>
      {showConfig && (
        <Suspense fallback={<Spin />}>
          <ConfigDialog onClose={() => setShowConfig(false)} />
        </Suspense>
      )}
    </div>
  );
};
```

## 常见陷阱

### 避免在渲染中创建函数
**✗ 错误示例**:
```typescript
export const SuggestionList: React.FC = () => {
  return (
    <div>
      {suggestions.map(s => (
        <SuggestionItem
          key={s.id}
          suggestion={s}
          onSelect={(id) => handleSelect(id)} // ✗ 每次渲染都创建新函数
        />
      ))}
    </div>
  );
};
```

**✓ 正确示例**:
```typescript
export const SuggestionList: React.FC = () => {
  const handleSelect = useCallback((id: string) => {
    // 处理逻辑
  }, []);

  return (
    <div>
      {suggestions.map(s => (
        <SuggestionItem
          key={s.id}
          suggestion={s}
          onSelect={handleSelect} // ✓ 使用稳定的引用
        />
      ))}
    </div>
  );
};
```

### 避免过度使用 useEffect
**✗ 错误示例**:
```typescript
const [messages, setMessages] = useState<string[]>([]);
const [messageCount, setMessageCount] = useState(0);

useEffect(() => {
  setMessageCount(messages.length); // ✗ 不需要 useEffect
}, [messages]);
```

**✓ 正确示例**:
```typescript
const [messages, setMessages] = useState<string[]>([]);
const messageCount = messages.length; // ✓ 直接计算
```

### 避免在循环中使用 key={index}
**✗ 错误示例**:
```typescript
{suggestions.map((s, index) => (
  <SuggestionItem key={index} suggestion={s} /> // ✗ 使用索引作为 key
))}
```

**✓ 正确示例**:
```typescript
{suggestions.map(s => (
  <SuggestionItem key={s.id} suggestion={s} /> // ✓ 使用唯一 ID
))}
```

## 窗口跟随功能

WeReply 特有的窗口跟随逻辑：

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';

export const useWindowFollow = (targetWindowName: string) => {
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const position = await commands.getWeChatWindowPosition();
        if (position) {
          const appWindow = getCurrentWindow();
          await appWindow.setPosition({
            x: position.x + position.width + 10, // 微信窗口右侧
            y: position.y,
          });
        }
      } catch (error) {
        console.error('窗口跟随失败:', error);
      }
    }, 500); // 每 500ms 更新一次位置

    return () => clearInterval(interval);
  }, [targetWindowName]);
};
```
