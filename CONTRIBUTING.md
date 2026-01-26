# Contributing

感谢参与 WeReply。

## 工作流
- 使用 feature 分支：`feat/...`、`fix/...`、`refactor/...`。
- 单一主题提交，提交信息遵循 Conventional Commits。
- 避免直接在 `main` 上修改。

## 开发准备
- Node.js + Rust 工具链。
- 安装依赖：`npm install`。

## 常用命令
```bash
npm run tauri:dev
npm run lint
npm test
npm run tauri:build
cargo test -p wereply
cargo run -p wereply --bin generate_bindings
```

## 平台相关
- Windows Agent 变更建议运行：
  `python -m unittest discover -s platform_agents/windows/tests`
- macOS Agent 需验证 Accessibility 权限逻辑与输入写入流程。

## 变更说明
- IPC 协议、Agent 行为或配置策略变更，请同步更新 README。
- 跨平台改动需注明 Windows/macOS 影响范围。
