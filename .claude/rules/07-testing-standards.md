# 测试规范

## 测试覆盖率要求

**强制要求**：所有新功能和 Bug 修复必须达到 **80% 以上测试覆盖率**。

---

## 必需的测试类型

### 1. 单元测试（Unit Tests）
- **范围**：独立函数、工具类、组件
- **工具**：Rust (cargo test)、TypeScript (Vitest)
- **要求**：每个测试应快速执行（< 100ms）

### 2. 集成测试（Integration Tests）
- **范围**：API 端点、数据库操作、服务交互
- **工具**：Rust (tests/ 目录)、TypeScript (Vitest)

### 3. 手动测试
- **范围**：关键用户流程
- **场景**：配方优化、预混料设计、报告生成等核心功能

---

## Rust 单元测试

### 基本测试结构
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_formula_name() {
        // 正常情况
        assert!(validate_formula_name("测试配方").is_ok());

        // 边界情况
        assert!(validate_formula_name("").is_err());
        assert!(validate_formula_name(&"a".repeat(100)).is_err());
    }

    #[test]
    fn test_calculate_nutrition() {
        let material = Material {
            code: "M001".to_string(),
            protein: 18.0,
            energy: 3200.0,
            ..Default::default()
        };

        let result = calculate_nutrition(&material);

        assert_eq!(result.protein, 18.0);
        assert_eq!(result.energy, 3200.0);
    }
}
```

### 异步测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_formula() {
        let pool = setup_test_db().await;
        let repo = FormulaRepository::new(Arc::new(pool));

        let result = repo.get_formula(1).await;

        assert!(result.is_ok());
        let formula = result.unwrap();
        assert_eq!(formula.id, 1);

        cleanup_test_db().await;
    }
}
```

### 测试错误情况
```rust
#[test]
fn test_invalid_proportion() {
    let result = validate_proportion(-0.1);
    assert!(result.is_err());

    let result = validate_proportion(101.0);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_formula_not_found() {
    let pool = setup_test_db().await;
    let repo = FormulaRepository::new(Arc::new(pool));

    let result = repo.get_formula(99999).await;

    assert!(result.is_err());
}
```

---

## Rust 集成测试

### 数据库集成测试
```rust
// tests/formula_integration_test.rs
use cacrfeedformula::*;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // 运行迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    pool
}

#[tokio::test]
async fn test_create_formula_integration() {
    let pool = setup_test_db().await;
    let repo = FormulaRepository::new(Arc::new(pool));

    let dto = CreateFormulaDto {
        name: "测试配方".to_string(),
        species_code: "PIG".to_string(),
        description: Some("测试描述".to_string()),
    };

    let result = repo.create(dto).await;
    assert!(result.is_ok());

    let formula_id = result.unwrap();
    assert!(formula_id > 0);

    // 验证数据已保存
    let formula = repo.get_formula(formula_id).await.unwrap();
    assert_eq!(formula.name, "测试配方");
}

#[tokio::test]
async fn test_formula_with_materials() {
    let pool = setup_test_db().await;
    let service = FormulaService::new(Arc::new(pool));

    // 创建配方和原料
    let formula_id = service.create_formula_with_materials(dto).await.unwrap();

    // 验证关联数据
    let details = service.get_formula_details(formula_id).await.unwrap();
    assert_eq!(details.materials.len(), 3);
}
```

### 事务测试
```rust
#[tokio::test]
async fn test_transaction_rollback() {
    let pool = setup_test_db().await;
    let repo = FormulaRepository::new(Arc::new(pool));

    let dto = CreateFormulaDto {
        name: "测试配方".to_string(),
        species_code: "INVALID".to_string(),  // 无效的品种代码
    };

    // 应该失败并回滚
    let result = repo.create_with_validation(dto).await;
    assert!(result.is_err());

    // 验证数据未保存
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM formulas")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
```

---

## TypeScript 单元测试

### 基本测试结构
```typescript
import { describe, it, expect } from 'vitest';
import { formatCurrency, validateFormulaName } from './utils';

describe('formatCurrency', () => {
  it('should format number as currency', () => {
    expect(formatCurrency(1234.56)).toBe('¥1,234.56');
  });

  it('should handle zero', () => {
    expect(formatCurrency(0)).toBe('¥0.00');
  });

  it('should handle negative numbers', () => {
    expect(formatCurrency(-100)).toBe('-¥100.00');
  });
});

describe('validateFormulaName', () => {
  it('should accept valid names', () => {
    expect(validateFormulaName('测试配方')).toBe(true);
    expect(validateFormulaName('Formula_01')).toBe(true);
  });

  it('should reject invalid names', () => {
    expect(validateFormulaName('')).toBe(false);
    expect(validateFormulaName('a'.repeat(100))).toBe(false);
  });
});
```

### React 组件测试
```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { FormulaCard } from './FormulaCard';

describe('FormulaCard', () => {
  const mockFormula = {
    id: 1,
    name: '测试配方',
    species_code: 'PIG',
    created_at: '2024-01-01T00:00:00Z',
  };

  it('should render formula information', () => {
    render(<FormulaCard formula={mockFormula} />);

    expect(screen.getByText('测试配方')).toBeInTheDocument();
    expect(screen.getByText('PIG')).toBeInTheDocument();
  });

  it('should call onEdit when edit button clicked', () => {
    const onEdit = vi.fn();
    render(<FormulaCard formula={mockFormula} onEdit={onEdit} />);

    const editButton = screen.getByRole('button', { name: /编辑/i });
    fireEvent.click(editButton);

    expect(onEdit).toHaveBeenCalledWith(1);
  });

  it('should call onDelete when delete button clicked', () => {
    const onDelete = vi.fn();
    render(<FormulaCard formula={mockFormula} onDelete={onDelete} />);

    const deleteButton = screen.getByRole('button', { name: /删除/i });
    fireEvent.click(deleteButton);

    expect(onDelete).toHaveBeenCalledWith(1);
  });
});
```

### Hook 测试
```typescript
import { describe, it, expect } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useFormulas } from './useFormulas';

describe('useFormulas', () => {
  it('should fetch formulas on mount', async () => {
    const { result } = renderHook(() => useFormulas());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.formulas).toHaveLength(3);
  });

  it('should handle errors', async () => {
    // Mock 错误响应
    vi.mocked(commands.getFormulas).mockRejectedValue(new Error('Network error'));

    const { result } = renderHook(() => useFormulas());

    await waitFor(() => {
      expect(result.current.error).toBeTruthy();
    });
  });
});
```

---

## Mock 策略

### Mock Tauri 命令
```typescript
import { vi } from 'vitest';

// 在测试文件顶部
vi.mock('../bindings', () => ({
  commands: {
    getFormulas: vi.fn(),
    createFormula: vi.fn(),
    updateFormula: vi.fn(),
    deleteFormula: vi.fn(),
  },
}));

// 在测试中使用
it('should create formula', async () => {
  vi.mocked(commands.createFormula).mockResolvedValue({
    success: true,
    data: { id: 1, name: '测试配方' },
    message: '创建成功',
  });

  const result = await createFormula(dto);
  expect(result.success).toBe(true);
});
```

### Mock 数据库（Rust）
```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        pub FormulaRepository {
            async fn get_formula(&self, id: i64) -> Result<Formula>;
            async fn create(&self, dto: CreateFormulaDto) -> Result<i64>;
        }
    }

    #[tokio::test]
    async fn test_with_mock_repo() {
        let mut mock_repo = MockFormulaRepository::new();

        mock_repo
            .expect_get_formula()
            .with(eq(1))
            .returning(|_| Ok(Formula {
                id: 1,
                name: "测试配方".to_string(),
                ..Default::default()
            }));

        let service = FormulaService::new(Arc::new(mock_repo));
        let result = service.get_formula(1).await;

        assert!(result.is_ok());
    }
}
```

---

## 测试最佳实践

### 1. 测试用户行为而非实现细节

**✓ 正确**：
```typescript
it('should display error when formula name is empty', async () => {
  render(<FormulaForm />);

  const submitButton = screen.getByRole('button', { name: /提交/i });
  await userEvent.click(submitButton);

  expect(screen.getByText(/请输入配方名称/i)).toBeInTheDocument();
});
```

**✗ 错误**：
```typescript
it('should set error state to true', () => {
  const { result } = renderHook(() => useFormula());
  act(() => result.current.setError(true));
  expect(result.current.error).toBe(true);  // 测试实现细节
});
```

### 2. 测试隔离
每个测试应该独立运行，互不影响：

```rust
#[tokio::test]
async fn test_create_formula() {
    let pool = setup_test_db().await;  // 每个测试独立的数据库
    // 测试逻辑...
    cleanup_test_db().await;  // 清理
}
```

### 3. 使用描述性的测试名称

**✓ 正确**：
```rust
#[test]
fn test_validate_formula_name_rejects_empty_string() { }

#[test]
fn test_validate_formula_name_rejects_too_long_string() { }

#[test]
fn test_validate_formula_name_accepts_valid_chinese_name() { }
```

**✗ 错误**：
```rust
#[test]
fn test1() { }

#[test]
fn test2() { }
```

### 4. 测试边界情况
```rust
#[test]
fn test_proportion_boundaries() {
    // 最小值
    assert!(validate_proportion(0.0).is_ok());

    // 最大值
    assert!(validate_proportion(100.0).is_ok());

    // 超出范围
    assert!(validate_proportion(-0.1).is_err());
    assert!(validate_proportion(100.1).is_err());
}
```

### 5. 测试错误路径
不仅测试正常情况，也要测试错误情况：

```rust
#[tokio::test]
async fn test_create_formula_with_invalid_species() {
    let pool = setup_test_db().await;
    let repo = FormulaRepository::new(Arc::new(pool));

    let dto = CreateFormulaDto {
        name: "测试配方".to_string(),
        species_code: "INVALID".to_string(),
    };

    let result = repo.create(dto).await;
    assert!(result.is_err());
}
```

---

## 测试覆盖率验证

### Rust 覆盖率
```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage

# 查看报告
open coverage/index.html
```

### TypeScript 覆盖率
```bash
# 运行测试并生成覆盖率
npm run test:coverage

# 查看报告
open coverage/index.html
```

### 覆盖率目标
- **整体覆盖率**：>= 80%
- **核心业务逻辑**：>= 90%
- **工具函数**：>= 95%

---

## 测试组织结构

### Rust 测试文件组织
```
src/
├── formula/
│   ├── mod.rs
│   ├── repository.rs
│   ├── service.rs
│   └── tests/           # 模块测试
│       ├── mod.rs
│       ├── repository_test.rs
│       └── service_test.rs
tests/                   # 集成测试
├── formula_integration_test.rs
└── common/
    └── mod.rs          # 测试工具函数
```

### TypeScript 测试文件组织
```
frontend/src/
├── components/
│   ├── FormulaCard.tsx
│   └── FormulaCard.test.tsx    # 组件测试
├── hooks/
│   ├── useFormulas.ts
│   └── useFormulas.test.ts     # Hook 测试
└── utils/
    ├── format.ts
    └── format.test.ts          # 工具函数测试
```

---

## 测试失败处理

### 测试失败时的检查清单
1. **检查测试隔离性**
   - 测试是否依赖其他测试的状态？
   - 是否正确清理了测试数据？

2. **验证 Mock 准确性**
   - Mock 的行为是否符合实际？
   - Mock 的返回值是否正确？

3. **检查异步处理**
   - 是否正确使用了 `await`？
   - 是否等待了异步操作完成？

4. **修复实现而非测试**
   - 除非测试本身有错，否则修复实现代码
   - 不要为了通过测试而修改测试

---

## 提交前测试检查清单

- [ ] 所有测试通过（`cargo test` 和 `npm test`）
- [ ] 覆盖率 >= 80%
- [ ] 无跳过的测试（`#[ignore]` 或 `it.skip`）
- [ ] 测试命名清晰描述行为
- [ ] 边界情况已测试
- [ ] 错误路径已测试
- [ ] Mock 使用合理
- [ ] 测试独立运行

---

## 常见测试陷阱

### 1. 测试实现细节而非行为
**✗ 错误**：测试组件的内部状态
**✓ 正确**：测试用户可见的行为

### 2. 测试之间有依赖
**✗ 错误**：test2 依赖 test1 的数据
**✓ 正确**：每个测试独立设置数据

### 3. 过度 Mock
**✗ 错误**：Mock 所有依赖，测试变成空壳
**✓ 正确**：只 Mock 外部依赖（数据库、API）

### 4. 忽略异步问题
**✗ 错误**：忘记 `await` 导致测试不稳定
**✓ 正确**：正确处理所有异步操作

### 5. 测试名称不清晰
**✗ 错误**：`test1`, `test2`
**✓ 正确**：`test_create_formula_with_valid_data`

---

## 参考资源

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Testing Best Practices](https://github.com/goldbergyoni/javascript-testing-best-practices)
