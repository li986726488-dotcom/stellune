# ARCHITECTURE.md

本文档记录 Stellune 的技术架构和关键数据链路。编码约束见根目录 `AGENTS.md`。

## 技术栈

| 层级 | 技术 |
|---|---|
| 应用容器 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Vite |
| 状态与路由 | Pinia + Vue Router |
| 后端 | Rust + Tokio |
| 网络 | `reqwest` |
| 数据库 | SQLite + `sqlx` |
| 前端测试 | Playwright |
| 后端测试 | Rust unit/integration test |

PC 主窗口固定为 `960 × 640`，默认 `resizable: false`。

## 总体结构

```text
Vue Page
  -> Pinia Store
  -> Typed Command Client
  -> Tauri Command
  -> Application Service
       ├─ Domain Rules
       ├─ AstrologyProvider
       ├─ NarrativeProvider
       └─ SQLite Repository
```

- 前端只消费项目 DTO。
- service 负责流程、缓存和事务。
- domain 负责确定性计算。
- provider 负责外部服务适配。
- repository 负责本地持久化。

## 页面

| 路由 | 页面 |
|---|---|
| `/onboarding` | 首次填写生日和可选资料 |
| `/` | 今日宇宙简报 |
| `/trail` | 七日和历史回望 |
| `/explore` | 每日签运、太阳星座轻量共鸣与情绪说明书 |
| `/profile` | 个人资料与设置 |

## 目录建议

```text
src/
  pages/
  components/
  stores/
  services/
    commands.ts
    errors.ts
  types/
  styles/
    tokens.css
    global.css

src-tauri/src/
  commands/
  services/
  domain/
    explore.rs
    models.rs
    rules.rs
  infrastructure/
    astrology.rs
    database.rs
src-tauri/migrations/
```

## 个性化模式

| 模式 | 资料 | 计算范围 |
|---|---|---|
| `sunSignLite` | 生日 | 太阳星座参考点和轻量解读 |
| `natal` | 生日、准确时间、地点 | 本命盘和每日行运 |

资料不足时不得猜测上升或宫位，UI 应明确显示“太阳星座轻量版”。

## 今日数据链路

```text
读取 Profile
  -> 生成缓存键
  -> 命中本地结果：直接返回
  -> 未命中：
       获取星相
       -> 保存原始响应
       -> 标准化行星位置
       -> 计算相位和相对宫位
       -> 计算五维分数
       -> 生成幸运提示
       -> 形成 ReadingDraft
       -> 生成并校验文案
       -> 保存结果和追踪
       -> 返回页面 DTO
```

缓存键由以下内容组成：

```text
localDate
+ profileFingerprint
+ personalizationMode
+ rulesVersion
+ astrologyProviderVersion
+ narrativePromptVersion
```

同一缓存键只生成一次。

## 星相 Provider

开发阶段使用 AstrologyAPI：

- `planets/tropical` 获取行星位置。
- `natal_transits/daily` 在资料完整时获取个人行运。
- 服务配置由 `infrastructure/config.rs::ServiceConfig` 统一读取，运行时环境变量优先。
- 内部测试包可以在编译进程中提供 `STELLUNE_ASTROLOGY_API_KEY`、
  `STELLUNE_NARRATIVE_API_KEY`、`STELLUNE_NARRATIVE_BASE_URL` 和
  `STELLUNE_NARRATIVE_MODEL`，通过 `option_env!` 生成安装包内的离线兜底配置；
  明文不写入 Git 文件或构建日志。

供应商响应先转换为项目模型：

```ts
interface PlanetPosition {
  planet: Planet
  longitude: number
  degreeInSign: number
  sign: ZodiacSign
  speed: number
  retrograde: boolean
}
```

注意：

- `isRetro` 可能是字符串，adapter 负责转成布尔值。
- 供应商 `house` 不能直接当作用户个人宫位。
- 编译期内置密钥可被逆向提取，只允许内部临时测试；正式发布包不得使用该方式，
  应改为受控代理或本地星历。

## 评分

五维为：

```text
love / work / wealth / social / inner
```

基础公式：

```text
score = clamp(
  baseline
  + aspectImpacts
  + houseImpacts
  + retrogradeModifiers,
  55,
  92
)
```

默认基线为 `75`，综合分为：

```text
round(
  love   * 0.20 +
  work   * 0.25 +
  wealth * 0.15 +
  social * 0.20 +
  inner  * 0.20
)
```

每次计算保存规则 id、相位角度、orb、维度加减值和 `rulesVersion`。大模型不参与评分。

## 文案

当前 `TemplateNarrativeRenderer` 使用去身份化的 `ReadingDraft` 语义：

```ts
interface ReadingDraft {
  date: string
  zodiac: ZodiacSign
  personalizationMode: "sunSignLite" | "natal"
  themeKeywords: string[]
  scores: ScoreDimensions
  relationshipFacts: SemanticFact[]
  workFacts: SemanticFact[]
  selfFacts: SemanticFact[]
}
```

它只生成标题、副标题、三条简报和宇宙回声。MVP 不请求大模型；后续接入模型时仍必须使用同一草稿，并在失败时回退到模板。

## 本地日志

Rust 侧通过 `infrastructure/logger` 输出结构化 JSONL。日志目录始终由
`app.path().app_data_dir()/logs` 计算，不写入项目目录、SQLite 或前端存储。

```json
{
  "timestamp": "2026-07-23T18:00:00+08:00",
  "level": "info",
  "event": "external.response",
  "correlationId": "reading-...",
  "payload": {}
}
```

日志按自然日写入 `stellune-YYYY-MM-DD.jsonl`，启动时清理七天前的同名日志文件。
日志失败属于非阻塞诊断能力，不能中断今日简报生成。

当前记录以下核心链路：

- `external.request` / `external.response`：星相接口地址、请求体、状态码和原始响应。
- `external.normalized`：转换后的行星位置。
- `reading.cache.*`：今日缓存命中或未命中。
- `rules.calculated`：标准化星相、规则命中、分数和幸运提示。
- `generation.input` / `generation.output`：模板生成输入和最终文案。
- `compatibility.calculated`：双方太阳星座、各因子得分、总分和规则版本。
- 情绪说明书复用 `generation.input/output`，输入为心情、主题和分数，输出为结构化建议。
- `reading.persisted` / `reading.discarded` / `reading.fallback`：保存、版本竞争丢弃和失败回退。

`external_request`、`external_response`、`generation_input` 和
`generation_output` 是 Provider 无关接口。未来接入 DeepSeek 时使用相同事件，
将 `provider/generator` 标记为模型服务即可，不新增平行日志通道。

写入前递归脱敏名称包含 `apiKey`、`token`、`secret`、`authorization`、
`password` 或 `credential` 的字段。禁止记录请求 Header 和环境变量中的真实密钥。
星相请求中的 `lat`、`lon`、`tzone` 会以 `[REDACTED]` 落盘，避免通过坐标反推出出生城市。
生成日志只保留日期、星座、模式、版本和规则数据，不记录昵称、生日或出生城市。

## 幸运提示与每日签运

- 幸运色、数字和时段由确定性 `LuckyEngine` 生成。
- 每项结果保存 `basis` 和 `certainty: "entertainment-rule"`。
- 幸运时段在 `07:00—23:00` 内推算月亮日内位置；仅接近太阳星座合相、六合或三合
  且容许度不超过 `3°` 时返回具体窗口，否则返回“全天平稳”。
- 每日签运首次使用 `hash(localDate + guestId + fortuneRulesVersion) % 100` 从
  `domain/fortune_catalog.rs` 的原创“星轨百签”中生成稳定编号；主动重抽加入随机
  nonce，并在命中当前签时顺延一位，确保结果发生变化。
- 星轨百签参考传统编号签的结构，但 100 支标题、签诗、解读和建议均为项目原创，
  不使用具体宗教名号或复制传统签文。
- `DailyFortune` 保存 `number`、`catalog`、`grade`、签诗、解读和建议。
- 七个签级在 100 支签中按 `6 / 16 / 20 / 25 / 15 / 16 / 2` 分布，
  下签内容只提供风险提醒和可执行的自我照顾建议。
- 每次抽取后覆盖写入当天数据库记录，重开应用恢复最后一次结果。

## 探索页生成

探索页的关系共鸣由 `domain/explore.rs` 确定性计算，不由前端拼分数：

```text
raw =
  元素关系（18 / 30 / 35）
  + 宫位模式（7 / 9 / 12 / 15）
  + 阴阳极性（6 / 10）
  + 黄道距离（6 / 8 / 10 / 14 / 16 / 18 / 20）

score = 45 + round((raw - 39) × 50 / 39)
```

`39..78` 是遍历 78 组无序星座组合得到的实际原始分范围，线性归一化后完整使用
`45..95` 展示区间，避免最高等级永远无法出现。等级依次为
`45..54 / 55..69 / 70..79 / 80..89 / 90..95`。因子仍记录归一化前的
可解释分值，当前规则版本为 `2026.07.2`。

当前模式固定标记为
`sunSignCompatibility` 和 `sun-sign-entertainment`。它只表示太阳星座层面的
娱乐共鸣，不等同于双方本命盘合盘。

情绪说明书只使用用户主动选择或当天已保存的 `MoodEntry`，不推断情绪。
`EmotionGuideEngine` 将心情、今日主题和五维分数档位转换为需要、标题、解释和
一个可执行建议。结果使用现有 `generation.input/output` 记录，当前
`generator` 为 `emotion-template`；未来模型只允许润色文案，不改写规则结果。

## 数据库

| 表 | 内容 |
|---|---|
| `profiles` | 本地游客资料 |
| `sky_snapshots` | 星相原始响应和标准化数据 |
| `daily_readings` | 每日页面数据 |
| `mood_entries` | 每日心情 |
| `daily_fortunes` | 每日签运 |

规则追踪保存在 `daily_readings.trace_json`，第一版不单独建表。

关键唯一性：

- `daily_readings.cache_key`
- `mood_entries.local_date`
- `daily_fortunes(local_date, guest_id, fortune_rules_version)`

认证信息永不入库。

## Tauri Commands

| command | 用途 |
|---|---|
| `get_app_state` | 获取启动状态 |
| `get_profile` / `save_profile` | 读取和保存资料 |
| `get_daily_reading` | 读取或生成今日结果 |
| `save_mood` | 保存当天心情 |
| `get_today_mood` | 读取当天已保存心情，供今日页与探索页同步 |
| `get_trail` | 获取历史摘要 |
| `get_daily_fortune` | 获取已有签运 |
| `draw_daily_fortune` | 首次抽取或主动重抽签运 |
| `get_compatibility` | 按用户与对方太阳星座计算轻量共鸣 |
| `get_emotion_guide` | 按当天心情与今日简报生成情绪说明书 |

command 只做参数校验和 service 调用。

## 错误处理

前端统一接收：

```ts
interface AppError {
  code: string
  message: string
  retryable: boolean
}
```

- 有今日缓存时优先返回缓存。
- 无缓存且星相服务失败时显示可重试错误，不生成假数据。
- 文案服务失败时使用模板 fallback。
- 重复抽签返回已保存结果。
- SQLite 失败不能假装保存成功。

## 测试

```text
Rust unit
  -> 相位、评分、幸运、签运

Rust integration
  -> service、command、repository、migration

Fixture tests
  -> 星相和文案 provider

Playwright
  -> Vue 页面、路由、store、command mock
```

### Playwright E2E

浏览器 E2E 不启动真实 Tauri 后端，而是替换 command transport：

```text
Vue / Pinia
  -> CommandClient interface
       ├─ TauriCommandClient   生产环境
       └─ MockCommandClient    Playwright E2E
                                  -> JSON fixtures
                                  -> isolated in-memory state
```

只有 `e2e` 构建模式可以启用 `MockCommandClient`。生产构建不得通过 URL 参数或运行时开关切换到 mock。

建议目录：

```text
playwright.config.ts
tests/e2e/
  fixtures/
    profile.json
    daily-reading.json
    trail.json
    daily-fortune.json
    errors.json
  mocks/
    command-client.ts
  stellune.spec.ts
  visual.spec.ts
```

测试约束：

- 固定视口 `960 × 640`。
- 固定浏览器时区 `Asia/Shanghai` 和测试日期。
- 每个测试创建独立 browser context 和 mock 状态。
- 使用 `data-testid` 定位核心控件。
- 不请求真实星相、模型或其他外部网络。
- fixture 中同时保留页面数据和必要的 `basedOn` 来源字段。
- 用例结束后不保留跨测试状态。

核心覆盖：

1. 首次进入填写生日后进入今日页。
2. 今日页渲染综合分、五维星图、简报和幸运提示。
3. 心情保存后按钮状态和星迹记录同步变化。
4. 星迹页切换七日记录。
5. 每日签运重抽不会重复当前签，重复进入恢复当天最后一次结果。
6. 修改资料后生成状态更新，旧结果仍可追溯。
7. 星相失败、文案 fallback、空历史和重复抽签状态。

执行命令：

```powershell
npm run test:e2e
npm run test:e2e -- --project=chromium
```

Playwright 验证 Vue 浏览器流程，不替代真实 Tauri、Rust service 和 SQLite integration test。

## 设计 Token

`src/styles/tokens.css` 统一管理窗口、颜色、字体、间距、圆角、阴影和动效。组件不得新增近似硬编码颜色。

星图使用固定顺序：

```text
love -> work -> wealth -> social -> inner
```

点位、标签、分数和无障碍文本必须来自同一数据数组。
