# Stellune · 星迹

Stellune 是一款神秘浪漫风格的 PC 星座轻娱乐应用，提供今日宇宙简报、星迹回顾、
星座共鸣、情绪陪伴和“星轨百签”。应用以游客模式运行，个人资料和历史记录保存在本地。

> 内容用于轻娱乐与情绪陪伴，不构成专业占星、心理、医疗或决策建议。

## 当前功能

- 今日运势：真实天象事实、确定性娱乐规则与模板文案分层生成
- 星迹回顾：查看最近七日能量与心情记录
- 探索：原创星轨百签、太阳星座轻量共鸣、情绪说明书
- 我的：首次仅需生日，后续可自愿补充昵称、出生时间和城市
- 桌面体验：固定 `960 × 640` 无系统标题栏窗口，支持 DPI/缩放适配
- 本地数据：SQLite 持久化与脱敏 JSONL 核心链路日志

## 技术栈

- Tauri 2
- Vue 3 + TypeScript + Pinia
- Rust + SQLx + SQLite
- Playwright + Vitest

## 本地开发

准备 Node.js、Rust stable 和 Windows WebView2，然后执行：

```powershell
npm install
Copy-Item .env.example .env
npm run tauri dev
```

`.env` 支持以下开发配置：

```text
STELLUNE_ASTROLOGY_API_KEY
STELLUNE_NARRATIVE_API_KEY
STELLUNE_NARRATIVE_BASE_URL
STELLUNE_NARRATIVE_MODEL
```

不要提交真实密钥。正式发布应使用受控代理或本地星历，不应把长期密钥内置到客户端。

## 测试

```powershell
npm run typecheck
npm run test
npm run test:e2e
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Playwright 使用固定模拟数据，不会请求真实星相、大模型、Tauri command 或 SQLite。

## 构建 Windows 安装包

```powershell
npm run tauri build -- --bundles nsis
```

构建结果位于：

```text
src-tauri/target/release/bundle/nsis/
```

详细架构与数据生成逻辑请参阅 [ARCHITECTURE.md](ARCHITECTURE.md) 和
[docs/DATA_GENERATION_LOGIC.md](docs/DATA_GENERATION_LOGIC.md)。
