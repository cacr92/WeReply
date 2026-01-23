# 数据库与数据访问规范

## 数据库技术栈

- **数据库**: SQLite (嵌入式关系数据库)
- **ORM**: SQLx 0.7 (编译时类型安全的异步数据库库)
- **迁移**: SQLx 内置迁移系统

## Migration 规范

### 文件命名
- **格式**: `YYYYMMDDHHMMSS_description.up.sql` / `.down.sql`
- **描述**: 使用下划线分隔的英文

**示例**:
```
20250114120000_add_formula_version_column.up.sql
20250114120000_add_formula_version_column.down.sql
```

### SQL 编写规范
- 使用 `IF NOT EXISTS` / `IF EXISTS` 确保幂等性
- 添加适当的索引
- 使用事务包装多步操作

**✓ 正确示例**:
```sql
-- up.sql
BEGIN TRANSACTION;

ALTER TABLE formulas ADD COLUMN version INTEGER NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_formulas_version ON formulas(version);

COMMIT;

-- down.sql
BEGIN TRANSACTION;

DROP INDEX IF EXISTS idx_formulas_version;

ALTER TABLE formulas DROP COLUMN version;

COMMIT;
```

### 迁移最佳实践
- 每个迁移只做一件事
- 提供完整的 up 和 down 脚本
- 测试迁移的可逆性
- 在迁移中添加注释说明

## SQLx 使用规范

### 查询宏使用
使用编译时检查的 `sqlx::query_as!` 宏：

**✓ 正确示例**:
```rust
pub async fn get_formula(&self, id: i64) -> Result<Formula> {
    let formula = sqlx::query_as!(
        Formula,
        "SELECT id, name, species_code, created_at, updated_at 
         FROM formulas 
         WHERE id = ?",
        id
    )
    .fetch_one(&self.pool)
    .await?;
    
    Ok(formula)
}
```

### 避免 SELECT *
明确指定需要的列：

**✗ 错误示例**:
```rust
// ✗ 不要使用 SELECT *
sqlx::query_as!(
    Formula,
    "SELECT * FROM formulas WHERE id = ?",
    id
)
```

**✓ 正确示例**:
```rust
// ✓ 明确指定列
sqlx::query_as!(
    Formula,
    "SELECT id, name, species_code, created_at, updated_at 
     FROM formulas 
     WHERE id = ?",
    id
)
```

### 参数绑定
使用 `?` 占位符进行参数绑定：

**✓ 正确示例**:
```rust
sqlx::query_as!(
    Formula,
    "SELECT * FROM formulas WHERE species_code = ? AND status = ?",
    species_code,
    status
)
```

## 事务处理规范

### 何时使用事务
- 多个相关的数据库操作
- 需要保证数据一致性
- 复杂的业务逻辑

### 事务使用模式
**✓ 正确示例**:
```rust
pub async fn create_formula_with_details(
    &self,
    dto: CreateFormulaDto,
) -> Result<i64> {
    let mut tx = self.pool.begin().await?;
    
    // 1. 插入主记录
    let formula_id = sqlx::query!(
        "INSERT INTO formulas (name, species_code, description) 
         VALUES (?, ?, ?)",
        dto.name, dto.species_code, dto.description
    )
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();
    
    // 2. 插入关联记录
    for material in dto.materials {
        sqlx::query!(
            "INSERT INTO formula_materials (formula_id, material_code, proportion) 
             VALUES (?, ?, ?)",
            formula_id, material.code, material.proportion
        )
        .execute(&mut *tx)
        .await?;
    }
    
    // 3. 提交事务
    tx.commit().await?;
    
    Ok(formula_id)
}
```

### 事务错误处理
- 使用 `?` 操作符自动回滚
- 显式调用 `tx.commit()` 提交
- 不要忘记 `commit()`

**✗ 错误示例**:
```rust
pub async fn create_formula(dto: CreateFormulaDto) -> Result<i64> {
    let mut tx = self.pool.begin().await?;
    
    let formula_id = sqlx::query!(...)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
    
    // ✗ 忘记 commit，事务会自动回滚
    Ok(formula_id)
}
```

**✓ 正确示例**:
```rust
pub async fn create_formula(dto: CreateFormulaDto) -> Result<i64> {
    let mut tx = self.pool.begin().await?;
    
    let formula_id = sqlx::query!(...)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
    
    // ✓ 显式提交事务
    tx.commit().await?;
    
    Ok(formula_id)
}
```

## 查询优化规范

### 使用索引
为频繁查询的字段添加索引：

**✓ 正确示例**:
```sql
-- 为外键添加索引
CREATE INDEX IF NOT EXISTS idx_formula_materials_formula_id 
ON formula_materials(formula_id);

-- 为常用查询字段添加索引
CREATE INDEX IF NOT EXISTS idx_formulas_species_code 
ON formulas(species_code);

-- 为组合查询添加复合索引
CREATE INDEX IF NOT EXISTS idx_formulas_species_status 
ON formulas(species_code, status);
```

### 避免 N+1 查询
使用 JOIN 或批量查询：

**✗ 错误示例**:
```rust
// ✗ N+1 查询问题
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

**✓ 正确示例**:
```rust
// ✓ 使用 JOIN 一次性获取所有数据
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

### 批量操作
使用批量插入/更新：

**✗ 错误示例**:
```rust
// ✗ 循环插入
for material in materials {
    sqlx::query!(
        "INSERT INTO materials (code, name, price) VALUES (?, ?, ?)",
        material.code, material.name, material.price
    )
    .execute(&self.pool)
    .await?;
}
```

**✓ 正确示例**:
```rust
// ✓ 批量插入
use sqlx::QueryBuilder;

pub async fn batch_insert_materials(
    &self,
    materials: Vec<MaterialDto>,
) -> Result<usize> {
    let mut query_builder = QueryBuilder::new(
        "INSERT INTO materials (code, name, price) "
    );
    
    query_builder.push_values(materials, |mut b, material| {
        b.push_bind(material.code)
         .push_bind(material.name)
         .push_bind(material.price);
    });
    
    let result = query_builder.build().execute(&self.pool).await?;
    Ok(result.rows_affected() as usize)
}
```

## 数据库连接池管理

### 连接池配置
```rust
use sqlx::sqlite::SqlitePoolOptions;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)  // 最大连接数
        .min_connections(1)  // 最小连接数
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

### 连接池使用
- 使用 `Arc<SqlitePool>` 在多个服务间共享
- 不要为每个请求创建新的连接池
- 连接池会自动管理连接的创建和回收

**✓ 正确示例**:
```rust
pub struct FormulaRepository {
    pool: Arc<SqlitePool>,
}

impl FormulaRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
    
    pub async fn get_formula(&self, id: i64) -> Result<Formula> {
        // 从连接池获取连接
        sqlx::query_as!(...)
            .fetch_one(&*self.pool)  // 使用 &*self.pool
            .await
    }
}
```

## 数据类型映射

### SQLite 到 Rust 类型映射

| SQLite 类型 | Rust 类型 | 说明 |
|------------|----------|------|
| INTEGER | i64 | 整数 |
| REAL | f64 | 浮点数 |
| TEXT | String | 文本 |
| BLOB | Vec<u8> | 二进制数据 |
| NULL | Option<T> | 可空值 |

### 日期时间处理
使用 `chrono` 库处理日期时间：

**✓ 正确示例**:
```rust
use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
pub struct Formula {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## 常见陷阱

### 忘记提交事务
**✗ 错误示例**:
```rust
let mut tx = pool.begin().await?;
sqlx::query!(...).execute(&mut *tx).await?;
// ✗ 忘记 commit，事务会自动回滚
Ok(())
```

**✓ 正确示例**:
```rust
let mut tx = pool.begin().await?;
sqlx::query!(...).execute(&mut *tx).await?;
tx.commit().await?;  // ✓ 显式提交
Ok(())
```

### 在循环中执行查询
**✗ 错误示例**:
```rust
for id in ids {
    let item = sqlx::query!(...).fetch_one(&pool).await?;
    // ✗ N+1 查询问题
}
```

**✓ 正确示例**:
```rust
// ✓ 使用 IN 子句一次性查询
let items = sqlx::query!(
    "SELECT * FROM items WHERE id IN (?)",
    ids.join(",")
)
.fetch_all(&pool)
.await?;
```

### 不使用参数绑定
**✗ 错误示例**:
```rust
// ✗ SQL 注入风险
let sql = format!("SELECT * FROM formulas WHERE name = '{}'", name);
sqlx::query(&sql).fetch_all(&pool).await?;
```

**✓ 正确示例**:
```rust
// ✓ 使用参数绑定
sqlx::query_as!(
    Formula,
    "SELECT * FROM formulas WHERE name = ?",
    name
)
.fetch_all(&pool)
.await?;
```
