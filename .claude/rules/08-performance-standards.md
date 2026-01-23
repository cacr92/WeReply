# 性能优化规范

## 桌面应用性能特点

本项目是 Tauri 桌面应用，性能优化重点：
- ✅ 重点：Rust 后端计算性能、React 渲染性能、数据库查询优化
- ✅ 优势：无网络延迟、本地计算、直接文件访问
- ⚠️ 注意：内存使用、启动时间、响应速度

---

## 一、Rust 后端性能优化

### 1. 并行计算

使用 Rayon 进行数据并行处理：

**✓ 正确示例**：
```rust
use rayon::prelude::*;

pub fn calculate_nutrition_batch(
    materials: &[Material],
) -> Vec<NutritionResult> {
    materials
        .par_iter()  // 并行迭代器
        .map(|material| calculate_nutrition(material))
        .collect()
}

pub fn optimize_formulas_batch(
    formulas: &[FormulaDto],
) -> Vec<Result<OptimizationResult>> {
    formulas
        .par_iter()
        .map(|formula| optimize_formula(formula))
        .collect()
}
```

**适用场景**：
- 批量计算营养成分
- 批量优化配方
- 大量数据处理

### 2. 缓存策略

使�� Moka 缓存频繁访问的数据：

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct MaterialService {
    cache: Cache<String, Material>,
    repository: Arc<MaterialRepository>,
}

impl MaterialService {
    pub fn new(repository: Arc<MaterialRepository>) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1000)  // 最多缓存 1000 个原料
                .time_to_live(Duration::from_secs(3600))  // 1 小时过期
                .build(),
            repository,
        }
    }

    pub async fn get_material(&self, code: &str) -> Result<Material> {
        self.cache
            .try_get_with(code.to_string(), async {
                self.repository.find_by_code(code).await
            })
            .await
            .map_err(|e| anyhow!("获取原料失败: {}", e))
    }

    pub fn invalidate_cache(&self, code: &str) {
        self.cache.invalidate(code);
    }
}
```

**缓存策略**：
- **原料数据**：缓存 1 小时（变化不频繁）
- **品种数据**：缓存 1 小时
- **配方数据**：不缓存（经常变化）
- **营养标准**：缓存 24 小时（很少变化）

### 3. 避免不必要的克隆

**✓ 正确示例**：
```rust
// 使用引用
pub fn calculate_total_cost(materials: &[Material]) -> f64 {
    materials.iter().map(|m| m.price).sum()
}

// 移动所有权（如果不再需要原数据）
pub fn process_formulas(formulas: Vec<Formula>) -> Vec<ProcessedFormula> {
    formulas.into_iter()
        .map(|f| process_formula(f))
        .collect()
}
```

**✗ 错误示例**：
```rust
// 不必要的克隆
pub fn calculate_total_cost(materials: Vec<Material>) -> f64 {
    let cloned = materials.clone();  // ✗ 不必要
    cloned.iter().map(|m| m.price).sum()
}
```

### 4. 使用 Cow 优化字符串处理

```rust
use std::borrow::Cow;

pub fn normalize_material_code(code: &str) -> Cow<str> {
    if code.chars().all(|c| c.is_uppercase()) {
        // 已经是大写，不需要分配新字符串
        Cow::Borrowed(code)
    } else {
        // 需要转换，分配新字符串
        Cow::Owned(code.to_uppercase())
    }
}
```

### 5. 批量数据库操作

使用 QueryBuilder 进行批量插入：

```rust
use sqlx::QueryBuilder;

pub async fn batch_insert_materials(
    &self,
    materials: Vec<MaterialDto>,
) -> Result<usize> {
    let mut query_builder = QueryBuilder::new(
        "INSERT INTO materials (code, name, price, protein, energy) "
    );

    query_builder.push_values(materials, |mut b, material| {
        b.push_bind(material.code)
         .push_bind(material.name)
         .push_bind(material.price)
         .push_bind(material.protein)
         .push_bind(material.energy);
    });

    let result = query_builder.build().execute(&self.pool).await?;
    Ok(result.rows_affected() as usize)
}
```

### 6. 优化 HiGHS 求解器使用

```rust
pub async fn optimize_formula_with_timeout(
    &self,
    dto: OptimizeFormulaDto,
    timeout_secs: u64,
) -> Result<OptimizationResult> {
    // 使用 tokio::time::timeout 防止长时间阻塞
    tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        self.optimize_formula(dto)
    )
    .await
    .map_err(|_| anyhow!("优化超时"))?
}

pub fn optimize_formula_parallel(
    &self,
    formulas: Vec<OptimizeFormulaDto>,
) -> Vec<Result<OptimizationResult>> {
    // 使用 Rayon 并行优化多个配方
    formulas
        .par_iter()
        .map(|dto| self.optimize_formula_sync(dto))
        .collect()
}
```

---

## 二、React 前端性能优化

### 1. 使用 memo 避免不必要的重渲染

```typescript
import { memo } from 'react';

interface MaterialRowProps {
  material: Material;
  onSelect: (code: string) => void;
}

// ✓ 使用 memo 包装纯组件
export const MaterialRow = memo<MaterialRowProps>(({ material, onSelect }) => {
  return (
    <tr onClick={() => onSelect(material.code)}>
      <td>{material.code}</td>
      <td>{material.name}</td>
      <td>{material.price}</td>
    </tr>
  );
});

MaterialRow.displayName = 'MaterialRow';
```

### 2. 使用 useCallback 和 useMemo

```typescript
import { useCallback, useMemo } from 'react';

export const FormulaList: React.FC = () => {
  const { formulas, loading } = useFormulas();

  // ✓ 使用 useMemo 缓存计算结果
  const totalCost = useMemo(() => {
    return formulas.reduce((sum, f) => sum + f.cost, 0);
  }, [formulas]);

  const sortedFormulas = useMemo(() => {
    return [...formulas].sort((a, b) => b.created_at.localeCompare(a.created_at));
  }, [formulas]);

  // ✓ 使用 useCallback 缓存回调函数
  const handleDelete = useCallback((id: number) => {
    // 删除逻辑
  }, []);

  const handleEdit = useCallback((id: number) => {
    // 编辑逻辑
  }, []);

  return (
    <div>
      <p>总成本: {totalCost}</p>
      {sortedFormulas.map(f => (
        <FormulaCard
          key={f.id}
          formula={f}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      ))}
    </div>
  );
};
```

### 3. 虚拟滚动大列表

对于超过 100 项的列表，使用虚拟滚动：

```typescript
import { FixedSizeList } from 'react-window';

interface VirtualizedListProps {
  materials: Material[];
  onSelect: (material: Material) => void;
}

export const VirtualizedMaterialList: React.FC<VirtualizedListProps> = ({
  materials,
  onSelect,
}) => {
  const Row = ({ index, style }: any) => {
    const material = materials[index];
    return (
      <div style={style} onClick={() => onSelect(material)}>
        {material.name} - {material.price}
      </div>
    );
  };

  return (
    <FixedSizeList
      height={600}
      itemCount={materials.length}
      itemSize={50}
      width="100%"
    >
      {Row}
    </FixedSizeList>
  );
};
```

### 4. 懒加载和代码分割

```typescript
import { lazy, Suspense } from 'react';
import { Spin } from 'antd';

// 懒加载大组件
const FormulaOptimization = lazy(() => import('./FormulaOptimization'));
const PremixDesign = lazy(() => import('./PremixDesign'));
const ReportGeneration = lazy(() => import('./ReportGeneration'));

export const App: React.FC = () => {
  return (
    <Suspense fallback={<Spin size="large" />}>
      <Routes>
        <Route path="/optimize" element={<FormulaOptimization />} />
        <Route path="/premix" element={<PremixDesign />} />
        <Route path="/report" element={<ReportGeneration />} />
      </Routes>
    </Suspense>
  );
};
```

### 5. 防抖和节流

```typescript
import { useMemo } from 'react';
import { debounce } from 'lodash-es';

export const MaterialSearch: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');

  // 防抖搜索
  const debouncedSearch = useMemo(
    () =>
      debounce(async (term: string) => {
        const results = await commands.searchMaterials(term);
        setResults(results.data);
      }, 300),
    []
  );

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSearchTerm(value);
    debouncedSearch(value);
  };

  return <Input value={searchTerm} onChange={handleSearch} />;
};
```

### 6. 优化 Ant Design Table

```typescript
export const MaterialTable: React.FC = () => {
  const columns = useMemo<ColumnsType<Material>>(
    () => [
      {
        title: '原料代码',
        dataIndex: 'code',
        key: 'code',
        width: 120,
        fixed: 'left',
      },
      {
        title: '原料名称',
        dataIndex: 'name',
        key: 'name',
        width: 200,
      },
      // ... 其他列
    ],
    []
  );

  return (
    <Table
      columns={columns}
      dataSource={materials}
      rowKey="code"
      pagination={{
        pageSize: 50,
        showSizeChanger: true,
        showTotal: (total) => `共 ${total} 条`,
      }}
      scroll={{ y: 600 }}  // 固定表头
      virtual  // 启用虚拟滚动（Ant Design 5.x）
    />
  );
};
```

---

## 三、数据库性能优化

### 1. 索引策略

```sql
-- 为外键添加索引
CREATE INDEX IF NOT EXISTS idx_formula_materials_formula_id
ON formula_materials(formula_id);

CREATE INDEX IF NOT EXISTS idx_formula_materials_material_code
ON formula_materials(material_code);

-- 为常用查询字段添加索引
CREATE INDEX IF NOT EXISTS idx_formulas_species_code
ON formulas(species_code);

CREATE INDEX IF NOT EXISTS idx_formulas_created_at
ON formulas(created_at DESC);

-- 为组合查询添加复合索引
CREATE INDEX IF NOT EXISTS idx_formulas_species_status
ON formulas(species_code, status);
```

### 2. 避免 N+1 查询

**✗ 错误示例**：
```rust
// N+1 查询问题
pub async fn get_formulas_with_materials(&self) -> Result<Vec<FormulaWithDetails>> {
    let formulas = self.get_all_formulas().await?;
    let mut results = Vec::new();

    for formula in formulas {
        // ✗ 每次循环都执行一次查询
        let materials = self.get_materials(formula.id).await?;
        results.push(FormulaWithDetails { formula, materials });
    }

    Ok(results)
}
```

**✓ 正确示例**：
```rust
// 使用 JOIN 一次性获取所有数据
pub async fn get_formulas_with_materials(&self) -> Result<Vec<FormulaWithDetails>> {
    let rows = sqlx::query!(
        "SELECT
            f.id, f.name, f.species_code,
            fm.material_code, fm.proportion
         FROM formulas f
         LEFT JOIN formula_materials fm ON f.id = fm.formula_id
         ORDER BY f.id"
    )
    .fetch_all(&self.pool)
    .await?;

    // 组装数据
    let mut results = Vec::new();
    // ... 数据组装逻辑

    Ok(results)
}
```

### 3. 使用 SQLite 优化选项

```rust
pub async fn create_optimized_pool(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect(database_url)
        .await?;

    // 启用 WAL 模式（提高并发性能）
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;

    // 设置缓存大小（单位：页，每页 4KB）
    sqlx::query("PRAGMA cache_size = -64000")  // 64MB
        .execute(&pool)
        .await?;

    // 启用外键约束
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    // 设置同步模式（NORMAL 平衡性能和安全性）
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;

    Ok(pool)
}
```

### 4. 批量操作

```rust
// 批量插入
pub async fn batch_insert_formula_materials(
    &self,
    formula_id: i64,
    materials: Vec<FormulaMaterialDto>,
) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    let mut query_builder = QueryBuilder::new(
        "INSERT INTO formula_materials (formula_id, material_code, proportion) "
    );

    query_builder.push_values(materials, |mut b, material| {
        b.push_bind(formula_id)
         .push_bind(material.code)
         .push_bind(material.proportion);
    });

    query_builder.build().execute(&mut *tx).await?;
    tx.commit().await?;

    Ok(())
}
```

---

## 四、启动性能优化

### 1. 延迟加载非关键数据

```rust
#[tauri::command]
#[specta::specta]
pub async fn initialize_app(
    state: State<'_, TauriAppState>,
) -> ApiResponse<AppInitData> {
    // 只加载启动必需的数据
    let factories = state.factory_service.get_all().await?;
    let current_factory = state.factory_service.get_current().await?;

    // 其他数据延迟加载
    api_ok(AppInitData {
        factories,
        current_factory,
    })
}

// 在用户需要时才加载
#[tauri::command]
#[specta::specta]
pub async fn load_materials(
    state: State<'_, TauriAppState>,
) -> ApiResponse<Vec<Material>> {
    // 延迟加载原料数据
    state.material_service.get_all().await
}
```

### 2. 预加载常用数据

```typescript
export const App: React.FC = () => {
  useEffect(() => {
    // 应用启动时预加载常用数据
    const preloadData = async () => {
      await Promise.all([
        commands.loadMaterials(),
        commands.loadSpecies(),
        commands.loadNutritionStandards(),
      ]);
    };

    preloadData();
  }, []);

  return <AppContent />;
};
```

---

## 五、内存优化

### 1. 及时释放大对象

```rust
pub async fn process_large_dataset(data: Vec<LargeData>) -> Result<()> {
    for chunk in data.chunks(100) {
        process_chunk(chunk).await?;
        // chunk 处理完后自动释放
    }
    // data 在函数结束时释放
    Ok(())
}
```

### 2. 使用流式处理

```rust
use futures::StreamExt;

pub async fn export_formulas_to_file(
    &self,
    file_path: &Path,
) -> Result<()> {
    let mut stream = sqlx::query_as::<_, Formula>(
        "SELECT * FROM formulas"
    )
    .fetch(&self.pool);

    let mut file = File::create(file_path).await?;

    while let Some(formula) = stream.next().await {
        let formula = formula?;
        // 逐条写入，不需要一次性加载所有数据到内存
        write_formula_to_file(&mut file, &formula).await?;
    }

    Ok(())
}
```

---

## 六、性能监控

### 1. 使用 tracing 记录性能指标

```rust
use tracing::{info, instrument};
use std::time::Instant;

#[instrument(skip(self))]
pub async fn optimize_formula(
    &self,
    dto: OptimizeFormulaDto,
) -> Result<OptimizationResult> {
    let start = Instant::now();

    let result = self.perform_optimization(dto).await?;

    let duration = start.elapsed();
    info!(
        duration_ms = duration.as_millis(),
        formula_id = result.formula_id,
        "配方优化完成"
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
```

---

## 七、性能基准

### 目标性能指标
- **应用启动时间**：< 2 秒
- **配方优化计算**：< 5 秒（中等复杂度）
- **数据库查询**：< 100ms（单表查询）
- **UI 响应时间**：< 100ms（用户操作到界面更新）
- **大列表渲染**：< 500ms（1000 项）

### 性能测试
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_optimization_performance() {
        let service = setup_test_service().await;
        let dto = create_test_formula_dto();

        let start = Instant::now();
        let result = service.optimize_formula(dto).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_secs() < 5, "优化耗时超过 5 秒");
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

### 3. 未使用索引的查询
**✗ 错误**：
```sql
SELECT * FROM formulas WHERE LOWER(name) = 'test';  -- ✗ 无法使用索引
```

**✓ 正确**：
```sql
SELECT * FROM formulas WHERE name = 'Test';  -- ✓ 可以使用索引
CREATE INDEX idx_formulas_name ON formulas(name);
```

### 4. 组件过度渲染
**✗ 错误**：
```typescript
// 每次父组件渲染都会创建新函数
<Child onClick={() => handleClick()} />
```

**✓ 正确**：
```typescript
const handleClick = useCallback(() => { }, []);
<Child onClick={handleClick} />
```

---

## 参考资源

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [React Performance Optimization](https://react.dev/learn/render-and-commit)
- [SQLite Performance Tuning](https://www.sqlite.org/pragma.html)
- [Tauri Performance Guide](https://tauri.app/v1/guides/building/performance/)
