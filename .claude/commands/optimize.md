# /optimize - 性能优化分析

## 用途
识别性能瓶颈并提供具体的优化建议。

## 分析步骤

### 1. Rust 后端性能分析

#### 识别热点代码
```bash
# 使用 cargo-flamegraph 生成火焰图
cargo flamegraph --bin cacrfeedformula

# 使用 criterion 进行基准测试
cargo bench
```

#### 检查点
- [ ] 是否有不必要的克隆？
- [ ] 是否可以使用并行计算（Rayon）？
- [ ] 是否可以添加缓存（Moka）？
- [ ] 数据库查询是否优化？
- [ ] 是否有 N+1 查询问题？

### 2. React 前端性能分析

#### 使用 React DevTools Profiler
```typescript
// 识别重渲染问题
// 检查是否需要：
// - React.memo
// - useCallback
// - useMemo
```

#### 检查点
- [ ] 组件是否过度渲染？
- [ ] 是否需要虚拟滚动（大列表）？
- [ ] 是否需要代码分割（lazy loading）？
- [ ] 是否有内存泄漏？

### 3. 数据库性能分析

#### SQLite 优化
```sql
-- 检查查询计划
EXPLAIN QUERY PLAN SELECT ...;

-- 检查索引使用
PRAGMA index_list('table_name');

-- 检查表统计信息
ANALYZE;
```

#### 检查点
- [ ] 是否有缺失的索引？
- [ ] 是否有未使用的索引？
- [ ] 查询是否可以优化？
- [ ] 是否需要批量操作？

## 优化建议模板

```markdown
## 性能优化建议

### 高优先级（立即修复）
1. **配方优化计算耗时过长**
   - 问题：单次优化耗时 15 秒
   - 原因：未使用缓存，重复计算
   - 建议：添加 Moka 缓存，缓存原料数据
   - 预期改进：耗时减少到 3 秒

2. **原料列表渲染卡顿**
   - 问题：1000+ 原料渲染缓慢
   - 原因：��使用虚拟滚动
   - 建议：使用 react-window
   - 预期改进：渲染时间从 2 秒降到 200ms

### 中优先级（计划修复）
1. **数据库查询慢**
   - 问题：配方列表查询耗时 500ms
   - 原因：缺少索引
   - 建议：添加 species_code 索引
   - 预期改进：查询时间降到 50ms

### 低优先级（可选优化）
1. **启动时间优化**
   - 问题：应用启动耗时 3 秒
   - 原因：启动时加载所有数据
   - 建议：延迟加载非关键数据
   - 预期改进：启动时间降到 1.5 秒
```

## 性能基准

### 目标指标
- 应用启动时间: < 2 秒
- 配方优化计算: < 5 秒
- 数据库查询: < 100ms
- UI 响应时间: < 100ms
- 大列表渲染: < 500ms

### 测量工具
- Rust: cargo-flamegraph, criterion
- React: React DevTools Profiler
- 数据库: EXPLAIN QUERY PLAN
- 整体: Chrome DevTools Performance

## 何时使用
- 发现性能问题时
- 定期性能审查
- 重大功能上线前
- 用户反馈卡顿时

## 相关 Skills
- rust-optimization
- react-typescript-development
- sqlite-optimization
- performance-standards (规范文件)
