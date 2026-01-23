# 安全开发规范

## 桌面应用安全特点

本项目是 Tauri 桌面应用，安全重点与 Web 应用不同：
- ✅ 重点：本地数据保护、API 密钥管理、Tauri 命令安全
- ❌ 不适用：CSRF、CSP（这些是 Web 应用的安全措施）

---

## 强制安全检查清单

### 代码提交前必查
- [ ] 无硬编码密钥（API keys、密码、tokens）
- [ ] 所有 Tauri 命令参数已验证
- [ ] SQL 注入防护（使用 SQLx 参数化查询）
- [ ] 用户输入已验证（前端 + 后端双重验证）
- [ ] 敏感数据已加密存储
- [ ] 错误消息不暴露敏感信息
- [ ] 避免使用 `unsafe` 代码（除非绝对必要）

---

## 1. 密钥管理（桌面应用）

### 环境变量管理
**强制要求**：所有 API 密钥必须使用环境变量，禁止硬编码。

**✓ 正确示例**：
```rust
use std::env;
use anyhow::{Context, Result};

pub fn get_api_key() -> Result<String> {
    env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY 环境变量未设置")
}
```

**✗ 错误示例**：
```rust
// ✗ 禁止硬编码
const API_KEY: &str = "sk-1234567890abcdef";
```

### 敏感配置存储
桌面应用的配置文件应加密存储：

```rust
use tauri::api::path::app_config_dir;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,  // 应加密存储
    pub database_path: String,
}

// 使用 Tauri 的安全存储
pub async fn save_config(config: &AppConfig) -> Result<()> {
    // 实现加密存储逻辑
    Ok(())
}
```

---

## 2. Tauri 命令安全

### 参数验证
所有 Tauri 命令必须验证参数：

**✓ 正确示例**：
```rust
use validator::Validate;

#[derive(Deserialize, Validate, Type)]
pub struct CreateFormulaDto {
    #[validate(length(min = 2, max = 50))]
    pub name: String,

    #[validate(length(equal = 3))]
    pub species_code: String,

    #[validate(range(min = 0.0, max = 100.0))]
    pub proportion: Option<f64>,
}

#[tauri::command]
#[specta::specta]
pub async fn create_formula(
    dto: CreateFormulaDto,
    state: State<'_, TauriAppState>,
) -> ApiResponse<Formula> {
    // 验证输入
    if let Err(e) = dto.validate() {
        return api_err(format!("输入验证失败: {}", e));
    }

    // 处理逻辑...
}
```

### 权限控制
敏感操作应添加权限检查：

```rust
#[tauri::command]
#[specta::specta]
pub async fn delete_all_data(
    state: State<'_, TauriAppState>,
) -> ApiResponse<()> {
    // 桌面应用可以添加二次确认机制
    // 或者要求管理员密码

    // 执行删除...
}
```

---

## 3. SQL 注入防护

### 使用 SQLx 参数化查询
**强制要求**：必须使用 `sqlx::query_as!` 宏，禁止字符串拼接 SQL。

**✓ 正确示例**：
```rust
pub async fn get_formula_by_name(
    &self,
    name: &str,
) -> Result<Option<Formula>> {
    let formula = sqlx::query_as!(
        Formula,
        "SELECT id, name, species_code, created_at, updated_at
         FROM formulas
         WHERE name = ?",
        name  // 参数化查询，自动防止 SQL 注入
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(formula)
}
```

**✗ 错误示例**：
```rust
// ✗ 禁止字符串拼接 SQL
pub async fn get_formula_by_name(&self, name: &str) -> Result<Option<Formula>> {
    let sql = format!("SELECT * FROM formulas WHERE name = '{}'", name);
    // 这会导致 SQL 注入漏洞！
}
```

### 动态查询构建
如需动态构建查询，使用 QueryBuilder：

```rust
use sqlx::QueryBuilder;

pub async fn search_formulas(
    &self,
    filters: FormulaFilters,
) -> Result<Vec<Formula>> {
    let mut query = QueryBuilder::new(
        "SELECT id, name, species_code FROM formulas WHERE 1=1"
    );

    if let Some(name) = filters.name {
        query.push(" AND name LIKE ");
        query.push_bind(format!("%{}%", name));  // 安全的参数绑定
    }

    if let Some(species) = filters.species_code {
        query.push(" AND species_code = ");
        query.push_bind(species);
    }

    query.build_query_as::<Formula>()
        .fetch_all(&self.pool)
        .await
}
```

---

## 4. 输入验证

### 前端验证（第一道防线）
使用 Ant Design Form 验证：

```typescript
<Form.Item
  name="name"
  label="配方名称"
  rules={[
    { required: true, message: '请输入配方名称' },
    { min: 2, max: 50, message: '名称长度为 2-50 个字符' },
    { pattern: /^[\u4e00-\u9fa5a-zA-Z0-9_-]+$/, message: '只能包含中文、字母、数字、下划线和连字符' }
  ]}
>
  <Input />
</Form.Item>
```

### 后端验证（必须有）
**永远不要信任前端验证**，后端必须再次验证：

```rust
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate, Type)]
pub struct CreateFormulaDto {
    #[validate(length(min = 2, max = 50))]
    #[validate(regex = "FORMULA_NAME_REGEX")]
    pub name: String,
}

lazy_static! {
    static ref FORMULA_NAME_REGEX: Regex =
        Regex::new(r"^[\u4e00-\u9fa5a-zA-Z0-9_-]+$").unwrap();
}
```

---

## 5. 敏感数据保护

### 日志中不记录敏感信息
**✓ 正确��例**：
```rust
use tracing::info;

info!(
    formula_id = formula.id,
    formula_name = formula.name,
    "配方创建成功"
);
// ✓ 不记录 API 密钥、密码等敏感信息
```

**✗ 错误示例**：
```rust
// ✗ 禁止记录敏感信息
info!("API Key: {}", api_key);
info!("User password: {}", password);
```

### 错误消息不暴露内部信息
**✓ 正确示例**：
```rust
pub async fn get_formula(id: i64) -> ApiResponse<Formula> {
    match repository.find_by_id(id).await {
        Ok(formula) => api_ok(formula),
        Err(e) => {
            error!(error = %e, "查询配方失败");
            // 返回通用错误消息，不暴露内部细节
            api_err("查询配方失败，请稍后重试".to_string())
        }
    }
}
```

### 数据库敏感字段加密
对于特别敏感的数据（如 API 密钥），应加密存储：

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub fn encrypt_api_key(key: &str) -> Result<Vec<u8>> {
    // 使用 AES-256-GCM 加密
    // 实现加密逻辑
    Ok(vec![])
}

pub fn decrypt_api_key(encrypted: &[u8]) -> Result<String> {
    // 解密逻辑
    Ok(String::new())
}
```

---

## 6. 依赖安全

### 定期更新依赖
```bash
# 检查过时的依赖
cargo outdated

# 检查安全漏洞
cargo audit

# 更新依赖
cargo update
```

### 审查新依赖
添加新依赖前检查：
- [ ] 依赖是否活跃维护
- [ ] 是否有已知安全漏洞
- [ ] 下载量和社区评价
- [ ] 许可证是否兼容

---

## 7. Rust 特定安全

### 避免 unsafe 代码
**原则**：除非绝对必要，否则避免使用 `unsafe`。

**✓ 可接受的 unsafe 使用场景**：
- FFI 调用
- 性能关键路径（经过充分测试）
- 与 C 库交互

**✗ 不可接受的 unsafe 使用**：
- 为了绕过编译器检查
- 未经充分测试的代码
- 有安全替代方案的情况

### 使用 Rust 安全特性
```rust
// ✓ 使用 Option 而非空指针
pub fn find_material(code: &str) -> Option<Material> {
    // ...
}

// ✓ 使用 Result 进行错误处理
pub fn calculate_nutrition(material: &Material) -> Result<Nutrition> {
    // ...
}

// ✓ 使用借用检查器防止数据竞争
pub fn process_formulas(formulas: &[Formula]) -> Vec<Result> {
    // 编译器保证内存安全
}
```

---

## 8. 前端安全（React）

### 避免 XSS（虽然桌面应用风险较低）
**✓ 正确示例**：
```typescript
// ✓ React 默认转义内容
<div>{userInput}</div>

// ✓ 如需渲染 HTML，使用 DOMPurify
import DOMPurify from 'dompurify';
const clean = DOMPurify.sanitize(userInput);
<div dangerouslySetInnerHTML={{ __html: clean }} />
```

**✗ 错误示例**：
```typescript
// ✗ 直接渲染未清理的 HTML
<div dangerouslySetInnerHTML={{ __html: userInput }} />
```

### 验证 Tauri 命令响应
```typescript
const result = await commands.getFormula(id);
if (!result.success) {
  message.error(result.message);
  return;
}

// 验证数据结构
if (!result.data || typeof result.data.id !== 'number') {
  message.error('数据格式错误');
  return;
}

// 使用数据
setFormula(result.data);
```

---

## 9. 文件操作安全

### 路径验证
处理用户提供的文件路径时，验证路径安全：

```rust
use std::path::{Path, PathBuf};

pub fn validate_file_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);

    // 检查路径遍历攻击
    if path.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(anyhow!("不允许使用 .. 路径"));
    }

    // 检查绝对路径
    if path.is_absolute() {
        return Err(anyhow!("不允许使用绝对路径"));
    }

    Ok(path.to_path_buf())
}
```

### 文件类型验证
```rust
pub fn validate_import_file(path: &Path) -> Result<()> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("无效的文件扩展名"))?;

    match extension.to_lowercase().as_str() {
        "xlsx" | "xls" | "csv" => Ok(()),
        _ => Err(anyhow!("不支持的文件类型: {}", extension)),
    }
}
```

---

## 10. 安全响应协议

### 发现漏洞时的处理流程
1. **立即停止工作**
2. **评估漏洞严重性**
   - 高危：可能导致数据泄露、系统破坏
   - 中危：可能导致功能异常
   - 低危：理论风险，实际影响小
3. **修复漏洞**
4. **轮换暴露的密钥**（如适用）
5. **审计整个代码库**查找类似问题
6. **更新安全检查清单**

---

## 安全审查触发条件

以下情况必须进行安全审查：
- [ ] 添加新的 Tauri 命令
- [ ] 修改数据库访问层
- [ ] 处理用户文件上传/导入
- [ ] 集成第三方 API
- [ ] 添加新的配置项
- [ ] 修改认证/授权逻辑（如有）

---

## 提交前安全检查清单

- [ ] 运行 `cargo clippy` 无安全警告
- [ ] 运行 `cargo audit` 无已知漏洞
- [ ] 无硬编码密钥
- [ ] 所有 SQL 查询使用参数化
- [ ] 所有 Tauri 命令参数已验证
- [ ] 敏感数据已加密存储
- [ ] 日志中无敏感信息
- [ ] 错误消息不暴露内部细节
- [ ] 无不必要的 `unsafe` 代码
- [ ] 前端无 `dangerouslySetInnerHTML`（或已清理）

---

## 常见安全陷阱

### 1. 信任前端验证
**✗ 错误**：只在前端验证，后端不验证
**✓ 正确**：前后端都验证，后端验证是最后防线

### 2. 日志记录敏感信息
**✗ 错误**：`info!("API Key: {}", key)`
**✓ 正确**：`info!("API 调用成功")`

### 3. SQL 字符串拼接
**✗ 错误**：`format!("SELECT * FROM users WHERE id = {}", id)`
**✓ 正确**：`sqlx::query_as!(..., "WHERE id = ?", id)`

### 4. 过于详细的错误消息
**✗ 错误**：`api_err(format!("数据库连接失败: {}", db_error))`
**✓ 正确**：`api_err("操作失败，请稍后重试".to_string())`

---

## 参考资源

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Tauri Security Best Practices](https://tauri.app/v1/references/architecture/security/)
- [SQLx Documentation](https://docs.rs/sqlx/)
