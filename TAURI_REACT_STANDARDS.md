# Rust + Tauri + React + Tailwind 通用开发规范

> **快速参考**: 本文件包含 Rust + Tauri + React + Tailwind 技术栈的通用开发规范和最佳实践。

---

## AI 行为准则

### 对话启动要求

每次新对话开始时，AI 必须：

1. **重新完整阅读并理解本规则文件的所有内容**
2. **在回复用户之前，内化所有编码准则和执行协议**
3. **严格按照本规则文件中定义的开发流程执行任务**
4. **在整个对话过程中始终遵守这些规则，不得偏离**

### 沟通规范

- **问候语**: 对话开始时，以"你好！我是 Claude Code，你的 Rust + Tauri + React 开发助手"作为开场白
- **沟通语言**: 使用中文进行交流
- **专业性**: 保持专业、客观、技术导向的沟通风格

---

## 任务处理流程

### 非开发任务（咨询、解释、分析）

1. **需求分析** - 列出需求点
2. **任务分解** - 分解为具体子任务
3. **执行说明** - 说明正在执行的操作
4. **标记进度** - 实时标记完成状态

### 开发任务（代码修改、新功能、Bug修复）

**必须执行"完整开发闭环流程（14步）"**，详见下方。

---

## 完整开发闭环流程（14步强制执行）

**核心原则**：

- ✅ 使用 feature 分支开发，**永不直接修改 main 分支**
- ✅ 自动生成 changelog 和提交信息
- ✅ 所有代码提交到新分支，由用户本地验证后合并
- ✅ 根据修改的文件类型自动调用项目特定 skills

---

### 14步开发流程

#### 步骤 1：创建隔离分支（using-git-worktrees）

**触发时机**: 所有涉及代码修改的任务开始时

```bash
调用 Skill(skill: "using-git-worktrees")
```

#### 步骤 2：需求分析与设计（brainstorming）

**触发条件**: 新功能开发

```bash
调用 Skill(skill: "brainstorming")
```

#### 步骤 3：编写实现计划（writing-plans）

**触发条件**: 复杂功能或重构任务

```bash
调用 Skill(skill: "writing-plans")
```

#### 步骤 4：项目特定技术栈检查（自动识别）

**自动识别规则**:

| 文件特征 | 自动调用 Skill | 用途 |
|---------|---------------|------|
| `src-tauri/src/**/*.rs` | `rust-backend-specialist` | Rust 后端开发规范 |
| `src-tauri/src/**/commands.rs` | `tauri-development` | Tauri 命令规范 |
| `src/**/*.tsx` | `react-typescript-development` | React TypeScript 规范 |
| `src/**/*.ts` | `react-typescript-development` | TypeScript 类型规范 |
| `src/**/*.css` | `tailwind-development` | Tailwind CSS 规范 |
| `src-tauri/src/**/mod.rs` | `rust-backend-specialist` | Rust 模块规范 |
| 任何 API 设计 | `api-design` | API 设计规范 |

#### 步骤 5：TDD - 先写测试（test-driven-development）

```bash
调用 Skill(skill: "test-driven-development")
```

**要求**: 先写失败的测试，确保测试覆盖率 ≥ 80%

#### 步骤 6：实现代码

**强制要求**:

- 遵循所有编码准则
- 使用 LSP 工具理解现有代码结构
- 保持原子化提交原则

#### 步骤 7-10：质量保障流程

7. **security-review** - 检查 API 密钥管理、输入验证、安全漏洞
8. **check** - 运行 `cargo clippy` 和 `npm run lint`
9. **optimize** - 分析性能瓶颈（响应时间、内存使用）
10. **verification-before-completion** - 运行所有测试，验证覆盖率

#### 步骤 11-14：提交和完成

11. **changelog-generator** - 自动生成 CHANGELOG.md 条目
12. **commit** - 生成规范的提交信息（Conventional Commits）
13. **finishing-a-development-branch** - 提供 PR/合并/继续开发选项
14. **auto-merge-and-cleanup** - 自动合并到 main，清理分支和临时文件（强制执行）

---

### 特殊流程

**Bug 修复流程（12步）**: 跳过步骤2和3，使用 `systematic-debugging` 替代

**多任务并行**: 智能任务分解 + 最小化隔离

- **推荐方案**: 使用 Sub-agents - 无物理隔离，简单高效

**并行前提**:

- 任务修改不同文件
- 任务无依赖关系
- 系统资源充足

---

### 强制要求

1. **必须显式调用 Skill 工具** - ✅ `Skill(skill: "xxx")` / ✗ 仅在文字中说明
2. **不得跳过任何步骤** - 即使是简单任务也必须执行完整流程
3. **分支保护** - 绝对禁止直接在 main 分支上修改代码
4. **自动识别技术栈** - 分析文件路径，自动调用相应 skills

---

## 核心编码准则

### 严禁事项（桌面应用专项）

1. **禁止 `console.log/error/warn`** → 使用 `message.error/success/warning`
2. **禁止 `as any` 类型转换** → 使用精确类型断言或类型守卫
3. **禁止原始 `invoke`** → 使用生成的 `commands`
4. **禁止在循环中使用 Hooks**
5. **禁止硬编码 API 密钥** → 使用环境变量或系统密钥链存储
6. **禁止阻塞异步运行时** → 使用 `tokio::spawn` 或 `tokio::time::sleep`

### 必须遵守

1. **使用 specta 导出类型** - 所有 Tauri 命令必须有 `#[specta::specta]` 宏
2. **使用 `message` 组件** - 所有用户反馈通过 Ant Design `message` 组件
3. **使用 tracing 日志** - 使用结构化日志记录
4. **使用 `anyhow::Result + ApiResponse`** - 底层 anyhow，顶层 ApiResponse
5. **使用 JSON 进行 IPC 通信** - Rust ↔ 前端使用 JSON 协议
6. **处理异步异常** - 优雅处理异步操作的错误和超时
7. **主动使用 LSP 工具** - 修改代码前查看、评估影响、理解结构

---

## 核心工作原则

### 1. 上下文驱动开发

在任何代码修改前，必须彻底理解业务需求、现有代码逻辑及其上下文。禁止在信息不完整或存在假设的情况下进行编码。

### 2. 原子化提交

每次修改遵循最小化原则，每次提交仅解决一个明确定义的问题。

### 3. 完整功能交付

一次性完成所有相关功能，不留 `TODO` 或未完成的实现。

### 4. 文档生成限制

严禁主动生成任何文档（README、API文档等）。仅当用户明确要求时才生成。

### 5. 命令执行规范

- ❌ 禁止编写脚本文件（.sh、.bat等）
- ✅ 直接使用工具逐条执行命令
- ✅ 按正确顺序自动执行所有必要命令
- ✅ 对每条命令的执行结果进行验证

---

## 提交前检查清单

### 代码质量

- [ ] `cargo clippy` 无警告
- [ ] `npm run lint` 无错误
- [ ] 所有 Tauri 命令有 `#[specta::specta]` 宏
- [ ] 前端无 `console.log` 和 `as any`
- [ ] 错误处理使用 `message` 组件
- [ ] IPC 通信使用 JSON 协议
- [ ] React Hooks 遵循规则

### 安全检查

- [ ] 无硬编码 API 密钥、密码、tokens
- [ ] API 密钥使用环境变量或系统密钥链存储
- [ ] Tauri 命令参数已验证
- [ ] 日志中无敏感信息
- [ ] 输入已验证（前端 + 后端双重验证）

### 测试覆盖

- [ ] 所有测试通过（`cargo test` 和 `npm test`）
- [ ] 测试覆盖率 ≥ 80%
- [ ] 异步操作异常已测试
- [ ] API 调用失败已测试

### LSP 使用

- [ ] 修改代码前使用 LSP 查看结构和类型
- [ ] 使用 `findReferences` 评估修改影响范围
- [ ] 已更新 CHANGELOG.md 文件

---

## 完整闭环验证

**开发任务完成前，AI 必须确认（14项）**:

- [ ] 步骤 1：已调用 using-git-worktrees 创建分支
- [ ] 步骤 2：已调用 brainstorming（新功能）或 systematic-debugging（Bug修复）
- [ ] 步骤 3：已调用 writing-plans（复杂任务）
- [ ] 步骤 4：已自动调用项目特定 skills（根据文件类型）
- [ ] 步骤 5：已调用 test-driven-development（先写测试）
- [ ] 步骤 6：已实现代码
- [ ] 步骤 7：已调用 security-review（安全审查）
- [ ] 步骤 8：已调用 check（质量检查）
- [ ] 步骤 9：已调用 optimize（性能优化）
- [ ] 步骤 10：已调用 verification-before-completion（完成验证）
- [ ] 步骤 11：已调用 changelog-generator（生成日志）
- [ ] 步骤 12：已调用 commit（提交到 feature 分支）
- [ ] 步骤 13：已调用 finishing-a-development-branch（完成分支）
- [ ] 步骤 14：已调用 auto-merge-and-cleanup（自动合并和清理）

**如果任何一项未完成，任务不算完成，必须继续执行直到完整闭环。**

---

## 重要提醒

- 处理问题时，如果一直无法修复，积极思考新方案
- 不要陷入死循环，及时调整思路和方法
- 遇到困难时，回到需求分析阶段重新审视问题
- 每一步都先输出在做什么，然后开始执行

---

## 详细规范文档

详细技术规范请参考 `.claude/rules/` 目录：

- `01-project-overview.md` - 项目概览和技术栈
- `02-rust-backend-standards.md` - Rust 后端开发规范
- `03-react-frontend-standards.md` - React TypeScript 前端规范
- `04-configuration-standards.md` - 配置管理与环境变量规范
- `05-lsp-usage-standards.md` - LSP 使用规范
- `06-security-standards.md` - 安全开发规范
- `07-testing-standards.md` - 测试规范（80% 覆盖率）
- `08-performance-standards.md` - 性能优化规范
- `09-parallel-agents-standards.md` - 多 Agent 并行工作规范

---

## 技术栈核心规范

### Rust 后端（src-tauri/src）

#### 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `UserManager`, `AuthService` |
| 枚举 | PascalCase | `UserRole`, `AppState` |
| 特征 (Trait) | PascalCase | `Repository`, `Service` |
| 函数 | snake_case | `get_user`, `save_config` |
| 变量 | snake_case | `user_name`, `api_key` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| 模块 | snake_case | `user`, `auth`, `config` |

#### Tauri 命令规范

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct UserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Type, Clone)]
#[specta(inline)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[tauri::command]
#[specta::specta]
pub async fn create_user(
    request: UserRequest,
    state: State<'_, AppState>,
) -> ApiResponse<UserResponse> {
    // 实现...
}
```

#### 异步编程规范

```rust
use tokio::time::{timeout, Duration};

pub async fn process_with_timeout() -> Result<()> {
    timeout(
        Duration::from_secs(10),
        some_async_operation()
    )
    .await
    .map_err(|_| anyhow!("操作超时"))??;

    Ok(())
}
```

#### 错误处理规范

```rust
use anyhow::{Context, Result};

pub async fn process_data() -> Result<UserResponse> {
    let data = fetch_data()
        .context("获取数据失败")?;

    let processed = process_data(data)
        .await
        .context("处理数据失败")?;

    Ok(processed)
}
```

### React 前端（src）

#### 组件命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| React 组件 | PascalCase | `UserCard`, `SettingsPanel` |
| 自定义 Hooks | camelCase (use 前缀) | `useUserData`, `useSettings` |
| 工具函数 | camelCase | `formatDate`, `validateEmail` |
| 类型定义 | PascalCase | `User`, `Settings` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_ITEMS`, `DEFAULT_THEME` |

#### 函数组件定义

```typescript
import React from 'react';
import { Button, Card } from 'antd';

interface UserCardProps {
  user: User;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

export const UserCard: React.FC<UserCardProps> = ({
  user,
  onEdit,
  onDelete,
}) => {
  const handleEdit = () => {
    onEdit?.(user.id);
  };

  return (
    <Card className="user-card">
      <h3>{user.name}</h3>
      <p>{user.email}</p>
      <Button onClick={handleEdit}>编辑</Button>
    </Card>
  );
};
```

#### React Hooks 使用规范

```typescript
import { useCallback, useMemo } from 'react';

export const UserList: React.FC = () => {
  const { users, loading } = useUsers();

  // ✓ 使用 useMemo 优化计算
  const activeUsers = useMemo(() => {
    return users.filter(u => u.status === 'active');
  }, [users]);

  // ✓ 使用 useCallback 优化回调
  const handleSelect = useCallback((id: string) => {
    // 选择逻辑
  }, []);

  return (
    <div>
      {activeUsers.map(user => (
        <UserCard
          key={user.id}
          user={user}
          onSelect={handleSelect}
        />
      ))}
    </div>
  );
};
```

#### TypeScript 类型规范

```typescript
// 使用生成的类型绑定
import type { User, Settings } from '../bindings';
import { commands } from '../bindings';

export const UserService = {
  async getUser(id: string): Promise<User> {
    const result = await commands.getUser({ id });
    if (!result.success) {
      throw new Error(result.message);
    }
    return result.data;
  }
};

// 类型守卫
function isUser(data: unknown): data is User {
  return (
    typeof data === 'object' &&
    data !== null &&
    'id' in data &&
    'name' in data
  );
}
```

#### Ant Design 使用规范

```typescript
import { message } from 'antd';

const handleSave = async (data: User) => {
  try {
    await commands.saveUser(data);
    message.success('保存成功');
  } catch (error) {
    message.error(`保存失败: ${error.message}`);
  }
};
```

### Tailwind CSS 规范

#### 命名规范

- 使用语义化类名，而非样式描述
- 组件级样式使用前缀
- 避免过度嵌套

```tsx
// ✓ 正确
<div className="user-card bg-white rounded-lg shadow-md p-4">
  <h3 className="user-name text-lg font-semibold text-gray-800">
    {user.name}
  </h3>
</div>

// ✗ 避免
<div className="bg-white p-4 rounded-lg shadow-md hover:shadow-lg transition-shadow">
  <h3 className="text-lg font-semibold text-gray-800 hover:text-blue-600">
    {user.name}
  </h3>
</div>
```

#### 响应式设计

```tsx
<div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
  {/* 内容 */}
</div>
```

#### 暗色模式支持

```tsx
<div className="bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100">
  {/* 内容 */}
</div>
```

---

## 配置管理规范

### 环境变量

```bash
# .env.example
API_ENDPOINT=https://api.example.com
API_TIMEOUT=30
MAX_RETRIES=3
LOG_LEVEL=info
```

### API 密钥管理

```rust
use std::env;

pub fn get_api_key() -> Result<String> {
    env::var("API_KEY")
        .context("API_KEY 环境变量未设置")
}

pub fn get_api_endpoint() -> String {
    env::var("API_ENDPOINT")
        .unwrap_or_else(|_| "https://api.example.com".to_string())
}
```

### 用户配置

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub api_endpoint: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub theme: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://api.example.com".to_string(),
            timeout: 30,
            max_retries: 3,
            theme: "light".to_string(),
        }
    }
}
```

---

## 安全开发规范

### API 密钥管理

**禁止硬编码**:
```rust
// ✗ 禁止
const API_KEY: &str = "sk-1234567890abcdef";

// ✓ 正确
let api_key = std::env::var("API_KEY")?;
```

### 输入验证

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: String,
}

#[tauri::command]
pub async fn create_user(
    request: UserRequest,
) -> ApiResponse<UserResponse> {
    if let Err(e) = request.validate() {
        return api_err(format!("验证失败: {}", e));
    }
    // 处理逻辑...
}
```

### 日志安全

```rust
use tracing::{info, error};

pub async fn process_user(user_id: i64) -> Result<()> {
    info!(user_id = user_id, "处理用户请求");
    // ✓ 不记录敏感信息

    // ✗ 错误示例
    // info!("用户数据: {:?}", sensitive_data);

    Ok(())
}
```

---

## 测试规范

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[tokio::test]
    async fn test_create_user() {
        let result = create_user_test_data().await;
        assert!(result.is_ok());
    }
}
```

### 前端测试

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { UserCard } from './UserCard';

describe('UserCard', () => {
  it('should render user information', () => {
    const user = { id: '1', name: 'Test User', email: 'test@example.com' };
    render(<UserCard user={user} />);

    expect(screen.getByText('Test User')).toBeInTheDocument();
    expect(screen.getByText('test@example.com')).toBeInTheDocument();
  });
});
```

### 覆盖率要求

- **整体覆盖率**: >= 80%
- **核心业务逻辑**: >= 90%
- **工具函数**: >= 95%

---

## 性能优化规范

### Rust 后端优化

```rust
// 使用缓存
use moka::future::Cache;

pub struct CacheManager {
    cache: Cache<String, Vec<String>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }
}

// 使用并发
use tokio::task;

pub async fn process_concurrently(items: Vec<String>) -> Vec<Result<String>> {
    let tasks: Vec<_> = items.into_iter()
        .map(|item| task::spawn(async move { process_item(item).await }))
        .collect();

    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await.unwrap());
    }

    results
}
```

### React 前端优化

```typescript
import { memo, useCallback, useMemo } from 'react';

// 使用 memo 避免不必要的重渲染
export const UserItem = memo<UserItemProps>(({ user, onSelect }) => {
  return (
    <div onClick={() => onSelect(user.id)}>
      {user.name}
    </div>
  );
});

// 使用 useCallback 缓存回调
const handleSelect = useCallback((id: string) => {
  // 处理逻辑
}, []);

// 使用 useMemo 缓存计算结果
const filteredUsers = useMemo(() => {
  return users.filter(u => u.active);
}, [users]);
```

### 性能监控

```rust
use tracing::{info, instrument};
use std::time::Instant;

#[instrument(skip(self))]
pub async fn process_data() -> Result<()> {
    let start = Instant::now();

    // 处理逻辑...

    let duration = start.elapsed();
    info!(duration_ms = duration.as_millis(), "处理完成");

    Ok(())
}
```

---

## LSP 使用规范

### 必须使用 LSP 的场景

1. **修改代码前** - 使用 `goToDefinition` 找到目标代码
2. **理解结构** - 使用 `documentSymbol` 了解文件结构
3. **评估影响** - 使用 `findReferences` 查找所有引用
4. **类型确认** - 使用 `hover` 查看类型定义

### LSP 操作示例

```typescript
// 使用 LSP 查看函数定义
LSP({
  operation: "goToDefinition",
  filePath: "src-tauri/src/user/service.rs",
  line: 50,
  character: 10
})

// 使用 LSP 查找所有引用
LSP({
  operation: "findReferences",
  filePath: "src-tauri/src/user/service.rs",
  line: 50,
  character: 10
})
```

---

## 并行工作规范

### Sub-agents 方案（推荐）

```markdown
## 任务分解示例

用户请求: "优化系统性能"

分解:
1. 优化 Rust 后端缓存
2. 优化数据库查询
3. 优化前端渲染

并行执行:
- rust-backend-specialist: 添加 Moka 缓存
- react-frontend-specialist: 添加 memo 和虚拟滚动
```

### 冲突检测

- ✅ 修改不同文件 → 可以并行
- ❌ 修改相同文件 → 串行执行
- ❌ 有依赖关系 → 串行执行

---

## 常见陷阱

### Rust 后端

**✗ 阻塞异步运行时**:
```rust
std::thread::sleep(Duration::from_secs(1)); // ✗ 错误
tokio::time::sleep(Duration::from_secs(1)).await; // ✓ 正确
```

**✗ 过度克隆**:
```rust
let cloned = data.clone(); // ✗ 不必要
process(&data); // ✓ 使用引用
```

### React 前端

**✗ 在循环中使用 Hook**:
```typescript
{items.map(item => {
  const [value, setValue] = useState(0); // ✗ 错误
  return <div key={item}>{value}</div>;
})}
```

**✗ 使用 as any**:
```typescript
const data = response as any; // ✗ 错误
const data = response as User; // ✓ 正确
```

---

## 提交规范

### Conventional Commits

```
feat: 添加用户管理功能
fix: 修复登录状态丢失问题
refactor: 重构用户服务模块
test: 添加用户服务测试
docs: 更新 API 文档
chore: 更新依赖版本
```

### CHANGELOG 生成

```markdown
## v1.2.0 (2024-01-24)

### Features
- 添加用户管理功能
- 支持暗色模式

### Bug Fixes
- 修复登录状态丢失问题
- 修复表单验证错误

### Refactoring
- 重构用户服务模块
- 优化性能瓶颈
```

---

## 总结

本规范为 Rust + Tauri + React + Tailwind 技术栈提供通用的开发指南。核心原则：

1. **安全第一** - 保护 API 密钥、验证输入、防止注入
2. **质量优先** - 测试覆盖率 ≥ 80%，代码审查严格
3. **性能优化** - 合理使用缓存、并发、懒加载
4. **用户体验** - 快速响应、优雅错误处理、友好提示
5. **开发效率** - 使用 LSP、自动化工具、完整闭环流程

遵循这些规范，可以构建高质量、高性能、安全的桌面应用程序。
