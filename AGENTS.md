# AGENTS.md

修改代码前先阅读本文件和根目录 `ARCHITECTURE.md`。本文件只记录开发约束；产品细节以需求文档为准。

## 项目简介

Stellune（星迹）是一款 `Tauri 2 + Vue 3 + TypeScript + Rust` 的轻娱乐星座运势应用。第一版面向 PC 游客用户，资料和历史数据保存在本地，核心页面包括今日、星迹、探索、我的和首次引导。

## 核心边界

- 真实星相、娱乐规则、生成文案必须分层。
- 综合分和五维分数由确定性规则计算，大模型不得直接给分。
- 幸运色、数字和时段属于娱乐规则，不能描述成天文学事实。
- 同一资料、日期和规则版本下，今日结果必须稳定。
- 今日签运允许用户主动重抽；每次必须避开当前签，并覆盖保存当天最新结果。
- 资料不完整时使用太阳星座轻量模式，不能声称是完整本命盘。
- 不做账号、云同步、付费、广告、社交和严肃决策预测。

## 禁止事项

### 安全与隐私

- 禁止提交 AstrologyAPI、模型服务或代理服务 Token。
- 禁止把密钥写入前端、构建产物、数据库、fixture、日志或文档。
- 禁止 Vue 直接请求星相或模型服务。
- 禁止上传与当前生成无关的资料、心情和历史记录。
- 开发凭据只从未提交的环境变量读取：

```text
STELLUNE_ASTROLOGY_API_KEY
STELLUNE_NARRATIVE_API_KEY
```

- 正式安装包不得内置第三方长期密钥；发布前使用受控代理或本地星历。

### 架构

- 页面组件不得直接调用 Tauri `invoke`。
- store 不得直接访问 SQLite、文件系统或外部 HTTP。
- command 只做参数校验和 service 调用，不写 SQL、评分或 Prompt。
- repository 只负责持久化，不发网络请求。
- astrology provider 只提供并标准化星相事实。
- narrative provider 只能生成文案，不得修改事实、规则、分数和幸运提示。
- 禁止把供应商原始 JSON 直接传给页面。
- 禁止用 `Math.random()` 生成每日分数、幸运提示或签运。
- 外部服务失败时不得返回未标记的静态假运势。

## 分层

| 层 | 职责 |
|---|---|
| `src/pages` | 页面布局和事件转发 |
| `src/components` | 可复用展示和轻量交互 |
| `src/stores` | UI 状态和应用动作 |
| `src/services/commands.ts` | 类型化 Tauri command 封装 |
| `src/types` | 前端共享 DTO |
| `src-tauri/src/commands` | 参数校验和 service 调用 |
| `src-tauri/src/services` | 用例编排、缓存和事务 |
| `src-tauri/src/domain` | 相位、宫位、评分、幸运规则和领域模型 |
| `src-tauri/src/infrastructure/astrology` | 星相供应商适配 |
| `src-tauri/src/infrastructure/narrative` | 文案生成和校验 |
| `src-tauri/src/infrastructure/logger` | 本地结构化日志、脱敏和保留策略 |
| `src-tauri/src/infrastructure/repositories` | SQLite 读写 |

## 代码规则

### Vue / TypeScript

- 组件使用 `<script setup lang="ts">`。
- props、emits、store、command 参数和返回值必须有显式类型。
- 页面只调用 store；store 只调用 `src/services/commands.ts`。
- 共享 DTO 放入 `src/types/`，供应商字段不得散落在组件中。
- 异步页面覆盖 loading、empty、error、success。
- 日期键统一使用用户本地时区的 `YYYY-MM-DD`。
- 列表 key 使用稳定业务 id，不使用数组下标。

### Rust

- 对外 DTO 使用 `#[serde(rename_all = "camelCase")]`。
- 错误统一为 `AppError { code, message, retryable }`。
- 网络、数据库、文件和反序列化路径禁止 `unwrap()` / `expect()`。
- 领域规则使用纯函数，时间通过 `Clock` 注入。
- 供应商 `"true"` / `"false"` 字符串在 adapter 层转成布尔值。
- 原始响应和标准化结果分别保存，不能覆盖原始证据。
- migration 按序号命名，已发布 migration 不得修改。

### UI

- PC 窗口固定 `960 × 640`，默认不可缩放。
- 颜色、字号、间距、圆角、阴影和窗口尺寸统一放在 `src/styles/tokens.css`。
- 禁止在组件中新增无语义的硬编码颜色。
- 图标使用统一图标库，不用 Unicode 字符代替正式图标。
- 星图点位、分数标签和无障碍文本必须由同一五维数组生成。
- 动效必须支持 `prefers-reduced-motion`。

## 数据与生成

- 业务数据通过 Rust repository 写入 SQLite，不以 `localStorage` 作为正式数据源。
- 每份今日结果保存原始星相、标准化事实、规则命中、分数变化和文案来源。
- 五维固定为 `love`、`work`、`wealth`、`social`、`inner`。
- 调整评分权重或幸运算法时必须升级规则版本。
- 关系共鸣只能由 Rust `domain/explore` 计算，前端不得保存固定分数或复制评分公式。
- 情绪说明书只读取用户主动选择或当天保存的 `MoodEntry`，不得推断用户情绪。
- 探索页与今日页共用当天唯一的 `MoodEntry`。
- 每日签运必须从 `domain/fortune_catalog.rs` 的 100 支原创“星轨百签”读取；
  不得用签级替代独立签文，不得复制具体宗教签谱。
- 调整星轨百签的数量、顺序、内容或签级分布时必须升级
  `FORTUNE_RULES_VERSION`，并保持标题、签诗、解读和建议分别唯一。
- narrative provider 只接收去身份化的 `ReadingDraft`。
- 模型输出必须经过结构校验、长度校验和内容安全校验。
- 模型失败时使用基于同一 `ReadingDraft` 的模板渲染器。
- 外部接口、规则计算和文案生成使用同一 `correlationId` 写入本地 JSONL。
- 日志写入前必须递归脱敏密钥字段；禁止记录昵称、生日、出生城市和可反推城市的请求坐标。
- 模板和未来模型 Provider 统一使用 `generation.input/output`，不建立平行日志格式。

## 错误码

| code | 场景 |
|---|---|
| `VALIDATION_ERROR` | 参数非法 |
| `PROFILE_INCOMPLETE` | 资料不足 |
| `ASTROLOGY_NETWORK_ERROR` | 星相请求失败 |
| `ASTROLOGY_AUTH_ERROR` | 星相认证失败 |
| `ASTROLOGY_QUOTA_EXCEEDED` | 星相配额不足 |
| `ASTROLOGY_RESPONSE_INVALID` | 星相响应异常 |
| `NARRATIVE_GENERATION_ERROR` | 文案生成失败 |
| `DATABASE_ERROR` | 本地数据库失败 |
| `DAILY_FORTUNE_ALREADY_DRAWN` | 当日签运已存在 |
| `INTERNAL_ERROR` | 未分类内部错误 |

## 测试

- 领域规则使用固定时间和 fixture 的 Rust 单元测试。
- provider adapter 使用本地 JSON fixture，不请求真实服务。
- repository 使用临时 SQLite。
- 核心用户流程必须有 Playwright E2E，不能只做组件测试或人工检查。
- Playwright 通过 `MockCommandClient` 注入固定数据，测试中禁止调用真实 AstrologyAPI、大模型、Tauri command 和 SQLite。
- mock 数据放在 `tests/e2e/fixtures/`，按场景命名，不得散落在测试代码中。
- 每个测试使用独立状态；禁止依赖测试执行顺序或上一个用例留下的数据。
- 页面交互使用稳定 `data-testid`，禁止依赖文案、DOM 层级或数组位置定位。
- E2E 固定使用 `960 × 640` 视口和固定本地时间，截图、分数和每日签运不得随运行日期变化。
- Playwright 浏览器测试不能描述为真实 Tauri WebView E2E。
- Tauri command、service 和 SQLite 联动由 Rust integration test 覆盖。
- E2E 至少覆盖：首次引导、今日页、心情记录、七日星迹、每日签运、资料修改和错误状态。
- 必须覆盖每日缓存、跨日、资料变化、API 失败、模型 fallback 和重抽覆盖。
- 测试数据禁止包含真实 Token。

## 文档与检查

- command、数据库、provider 或模块边界变化时更新 `ARCHITECTURE.md`。
- 评分或幸运规则变化时更新规则版本和架构说明。
- 新增外部服务时记录用途、发送字段、认证、超时和隐私边界。

有代码后按修改范围运行：

```powershell
npm run typecheck
npm run test
npm run test:e2e
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

未运行或命令尚不存在时，最终说明必须明确指出。
