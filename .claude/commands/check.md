# /check - 代码质量和安全检查

## 用途
执行全面的代码质量和安全检查，包括 lint、类型检查、安全审计等。

## 执行步骤

### 1. Rust 后端检查
```bash
# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 格式检查
cargo fmt -- --check

# 安全审计
cargo audit

# 类型检查
cargo check --all-targets
```

### 2. TypeScript 前端检查
```bash
# ESLint 检查
cd frontend && npm run lint

# TypeScript 类型检查
cd frontend && npm run type-check

# 格式检查
cd frontend && npm run format:check
```

### 3. 测试覆盖率检查
```bash
# Rust 测试覆盖率
cargo tarpaulin --out Html --output-dir coverage

# TypeScript 测试覆盖率
cd frontend && npm run test:coverage
```

### 4. 依赖检查
```bash
# Rust 依赖更新检查
cargo outdated

# TypeScript 依赖检查
cd frontend && npm outdated
```

## 输出报告

检查完成后，生成报告：

```markdown
## 代码质量检查报告

### Rust 后端
- ✅ Clippy: 无警告
- ✅ 格式: 符合规范
- ✅ 安全审计: 无已知漏洞
- ✅ 类型检查: 通过

### TypeScript 前端
- ✅ ESLint: 无错误
- ✅ 类型检查: 通过
- ✅ 格式: 符合规范

### 测试覆盖率
- Rust: 85% (目标: 80%)
- TypeScript: 82% (目标: 80%)

### 依赖状态
- Rust: 3 个依赖可更新
- TypeScript: 5 个依赖可更新

## 建议
1. 更新过时的依赖
2. 继续保持测试覆盖率
```

## 何时使用
- 提交代码前
- Pull Request 前
- 定期代码质量检查
- CI/CD 流程中

## 相关 Skills
- code-review
- security-review
- testing-strategy
