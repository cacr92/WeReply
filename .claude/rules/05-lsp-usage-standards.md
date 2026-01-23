# LSP 使用规范

## 总则

**强制要求**：在处理任何代码相关任务时，必须主动使用LSP工具来提高准确性和效率。不需要用户提醒，应该将LSP作为标准工作流程的一部分。

## LSP 工具概述

LSP (Language Server Protocol) 提供以下核心功能：

| 操作 | 用途 | 何时使用 |
|------|------|----------|
| `hover` | 查看类型定义和文档 | 需要了解函数/类型的详细信息时 |
| `goToDefinition` | 跳转到符号定义 | 需要查看函数/类型/变量的源代码时 |
| `findReferences` | 查找所有引用 | 评估修改影响范围时 |
| `documentSymbol` | 获取文件符号列表 | 快速了解文件结构时 |
| `goToImplementation` | 查找trait/interface实现 | 需要查看具体实现时 |
| `prepareCallHierarchy` | 获取调用层次 | 分析函数调用关系时 |
| `incomingCalls` | 查找调用者 | 了解谁在使用这个函数时 |
| `outgoingCalls` | 查找被调用者 | 了解函数调用了哪些其他函数时 |

## 必须使用 LSP 的场景

### 1. 修改代码前的准备工作

**场景**：用户要求修改某个函数或组件

**操作流程**：
1. 使用 `goToDefinition` 找到目标代码位置
2. 使用 `documentSymbol` 了解文件整体结构
3. 使用 `hover` 查看类型定义和文档
4. 使用 `findReferences` 评估修改影响范围

**示例**：
```typescript
// 用户要求：修改 MaterialService 的 getMaterial 方法

// 步骤1: 跳转到定义
LSP({
  operation: "goToDefinition",
  filePath: "src/material/material_service.rs",
  line: 50,
  character: 10
})

// 步骤2: 查看类型信息
LSP({
  operation: "hover",
  filePath: "src/material/material_service.rs",
  line: 50,
  character: 10
})

// 步骤3: 查找所有引用
LSP({
  operation: "findReferences",
  filePath: "src/material/material_service.rs",
  line: 50,
  character: 10
})
```

### 2. 理解现有代码结构

**场景**：需要了解某个模块或文件的组织结构

**操作流程**：
1. 使用 `documentSymbol` 获取文件的所有符号（函数、类型、方法等）
2. 对关键符号使用 `hover` 查看详细信息
3. 使用 `goToDefinition` 查看依赖的类型定义

**示例**：
```rust
// 用户要求：理解 FormulaService 的实现

// 步骤1: 获取文件符号列表
LSP({
  operation: "documentSymbol",
  filePath: "src/formula/formula_service.rs",
  line: 1,
  character: 1
})

// 步骤2: 查看关键方法的信息
LSP({
  operation: "hover",
  filePath: "src/formula/formula_service.rs",
  line: 380,
  character: 10
})
```

### 3. 重构代码

**场景**：重命名函数、修改接口、移动代码

**操作流程**：
1. 使用 `findReferences` 找到所有使用位置
2. 评估影响范围
3. 使用 `goToDefinition` 确认依赖关系
4. 执行修改

**示例**：
```typescript
// 用户要求：重命名 useMaterials hook

// 步骤1: 查找所有引用
LSP({
  operation: "findReferences",
  filePath: "frontend/src/App.tsx",
  line: 103,
  character: 10
})

// 分析影响范围后再执行修改
```

### 4. 调试和问题排查

**场景**：代码出现错误或行为异常

**操作流程**：
1. 使用 `hover` 确认类型是否正确
2. 使用 `goToDefinition` 查看相关定义
3. 使用 `incomingCalls`/`outgoingCalls` 分析调用链
4. 使用 `findReferences` 查找可能的问题点

### 5. 添加新功能

**场景**：需要添加新的函数或组件

**操作流程**：
1. 使用 `documentSymbol` 了解现有代码结构
2. 使用 `goToDefinition` 查看相关类型定义
3. 使用 `findReferences` 查找类似实现作为参考
4. 编写新代码

## 最佳实践

### 1. 优先使用 LSP 而非盲目搜索

**✗ 错误做法**：
```
直接使用 Grep 搜索函数名，然后猜测是哪个文件
```

**✓ 正确做法**：
```
使用 LSP goToDefinition 精确定位到定义位置
```

### 2. 评估修改影响前必须使用 findReferences

**✗ 错误做法**：
```
修改函数签名后才发现破坏了很多调用点
```

**✓ 正确做法**：
```
修改前使用 findReferences 查看所有使用位置，评估影响范围
```

### 3. 理解类型前必须使用 hover

**✗ 错误做法**：
```
假设某个参数是 string 类型，实际是 Code 类型别名
```

**✓ 正确做法**：
```
使用 hover 查看确切的类型定义和文档
```

### 4. 分析代码结构时优先使用 documentSymbol

**✗ 错误做法**：
```
逐行阅读整个文件来了解结构
```

**✓ 正确做法**：
```
使用 documentSymbol 快速获取文件的组织结构
```

## 工作流程示例

### 示例1: 修改 Rust 函数

```
用户需求：修改 formula_service.rs 中的 optimize_formula 方法

步骤1: 定位代码
LSP(goToDefinition) -> 找到方法位置

步骤2: 了解当前实现
LSP(hover) -> 查看方法签名和文档
LSP(documentSymbol) -> 了解方法在类中的位置

步骤3: 评估影响
LSP(findReferences) -> 查找所有调用点

步骤4: 执行修改
根据LSP提供的信息进行准确修改

步骤5: 验证
再次使用 findReferences 确认所有调用点是否需要更新
```

### 示例2: 添加 TypeScript 组件

```
用户需求：添加新的配方管理组件

步骤1: 了解现有组件结构
LSP(documentSymbol) -> 查看类似组件的结构

步骤2: 查看类型定义
LSP(goToDefinition) -> 跳转到 Formula 类型定义
LSP(hover) -> 查看类型的详细信息

步骤3: 查找示例用法
LSP(findReferences) -> 查找 Formula 类型的使用示例

步骤4: 编写新组件
基于LSP提供的准确信息编写代码
```

## 常见错误和解决方案

### 错误1: LSP 返回 "server is starting"

**原因**：LSP 服务器正在启动或索引中

**解决方案**：
```rust
// 等待几秒后重试
Bash("sleep 3")
// 然后再次调用 LSP
```

### 错误2: LSP 返回 "No hover information available"

**原因**：光标位置不在符号上，或文件未完全索引

**解决方案**：
- 调整 character 参数，确保在符号上
- 检查 line 和 character 是否正确（1-based）
- 等待索引完成后重试

### 错误3: 忘记使用 LSP 导致修改错误

**原因**：凭感觉修改代码，没有使用LSP验证

**解决方案**：
- 养成修改前先用LSP查看的习惯
- 在任务分解中明确标注LSP使用步骤

## 自我检查清单

在开始任何代码修改任务前，问自己：

- [ ] 我是否使用了 `goToDefinition` 找到了目标代码？
- [ ] 我是否使用了 `hover` 确认了类型信息？
- [ ] 我是否使用了 `documentSymbol` 了解了文件结构？
- [ ] 我是否使用了 `findReferences` 评估了修改影响？
- [ ] 如果涉及调用关系，我是否使用了 `incomingCalls`/`outgoingCalls`？

**如果任何一项回答"否"，立即补充 LSP 调用！**

## 记录规范

在使用LSP时，应该在响应中说明：

**✓ 正确示例**：
```
现在我要使用LSP查看FormulaService的结构...
[调用LSP]
根据LSP返回的信息，我发现FormulaService有以下方法...
```

**✗ 错误示例**：
```
我看了一下FormulaService...（没有说明如何看的）
```

## 总结

LSP是提高代码修改准确性的关键工具。**必须主动使用，不等待用户提醒**。将LSP集成到每个代码任务的工作流程中，确保：

1. **修改前先了解** - 使用LSP查看现有结构
2. **修改中验证** - 使用LSP确认类型和签名
3. **修改后检查** - 使用LSP评估影响范围

这样可以：
- ✅ 减少错误
- ✅ 提高效率
- ✅ 避免破坏现有代码
- ✅ 准确评估影响范围
