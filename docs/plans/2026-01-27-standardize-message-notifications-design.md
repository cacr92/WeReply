# 消息通知统一设计

## 目标
- 前端所有提示统一走 Ant Design `message`，通过 `notify` 封装集中管理。
- 统一文案模板：`<动作>：<原因/建议>`，避免“失败/错误”提示不一致。
- 统一时长：success 2s、info 2s、warning 3s、error 4s、loading 手动关闭。
- 提示语可读、可行动，避免空泛“失败”。

## 架构与组件
- 新增 `src/utils/notify.ts`：封装 `message`，提供 `notify.success/info/warning/error/loading`。
- 封装层提供 `formatNotifyMessage/resolveNotifyDetail` 纯函数，便于测试与复用。
- `src/App.tsx` 替换所有 `message.*` 调用为 `notify.*`，并补齐动作与原因。

## 数据流
- 业务事件 → 组装 `action/detail` → `notify` 统一格式化 → `message` 展示。
- detail 来源于 API 的 `message`、异常 `Error.message` 或本地校验原因。
- 当 detail 缺失时，error 追加默认提示（例如“请稍后重试”）。

## 错误处理
- 仅展示可读错误，不直接输出敏感信息。
- `notify.error` 默认补充可执行建议；若业务已提供完整文案，可显式关闭 fallback。
- 保持现有交互行为不变，仅统一输出格式与时长。

## 测试策略
- 新增 `src/utils/notify.test.ts`，覆盖格式化与 fallback 逻辑。
- 运行 `npm test` 确保前端测试通过。
- 保持现有测试覆盖基线，新增测试用于巩固统一模板。
