# Stellune MVP 实现方案

## 1. 目标

在 `<PROJECT_ROOT>` 实现一个可运行的 Stellune（星迹）PC MVP：

- Tauri 2 桌面容器，固定窗口 `960 × 640`。
- Vue 3 + TypeScript + Pinia + Vue Router 前端。
- Rust Command + SQLite 本地数据。
- 今日、星迹、探索、我的、首次引导五个页面。
- 真实星相 Provider 接口和 AstrologyAPI 实现。
- 确定性评分、幸运提示与每日签运。
- 基于结构化语义的模板文案生成。
- Playwright 使用 mock command 数据覆盖核心用户流程。

本次不实现云端账号、付费、大模型远程调用和移动端页面。

## 2. 实现策略

采用双 CommandClient：

```text
Vue / Store
  -> CommandClient
       ├─ TauriCommandClient
       │    -> Tauri invoke
       │    -> Rust Service
       │    -> SQLite / AstrologyAPI
       └─ MockCommandClient
            -> Playwright fixtures
            -> 内存状态
```

页面和 Store 不判断运行环境，也不直接调用 Tauri。生产和 E2E 只在应用启动时选择不同的 CommandClient。

选择理由：

- Playwright 可以在普通浏览器稳定测试完整 Vue 流程。
- 不消耗真实星相 API 配额。
- mock 与生产使用同一 DTO 和方法签名。
- Rust 逻辑继续由单元测试和集成测试覆盖。

## 3. 用户流程

### 3.1 首次引导

1. 应用启动调用 `getAppState`。
2. 没有资料时进入 `/onboarding`。
3. 用户必须填写生日；昵称、出生时间、城市为可选。
4. 保存后计算太阳星座和资料完整度。
5. 进入今日页并加载今日结果。

### 3.2 今日

页面展示：

- 日期、太阳星座、主题、主标题和副标题。
- 综合能量与五维星图。
- 关系、工作、自我三条简报。
- 宇宙回声。
- 五种心情选择及保存操作。
- 幸运色、幸运数字和幸运时段。
- 数据模式标识：太阳星座轻量版或完整星盘版。

今日结果按缓存键稳定。刷新和重启不重新生成。

### 3.3 星迹

- 默认显示最近七天。
- 顶部显示七日平均、主要心情、上升维度。
- 趋势图展示综合分。
- 选择某一天后展示当日主题、心情、宇宙回声和五维分数。
- 没有记录时显示空状态。

### 3.4 探索

- 每天一次签运。
- 未抽取时显示可点击卡背。
- 抽取后显示签级、签名、签诗、解读和建议。
- 同一天再次进入读取原结果。
- 同时展示关系共鸣和情绪说明两张轻量卡片。

### 3.5 我的

- 展示昵称、星座、生日和资料完整度。
- 支持修改生日、昵称、出生时间和城市。
- 展示心情记录数、连续陪伴天数和今日签运状态。
- 展示数据来源为“真实星相 + 娱乐规则 + 生成文案”。

## 4. 前端设计

### 4.1 页面结构

```text
App.vue
  ├─ OnboardingPage
  └─ AppShell
       ├─ SideNavigation
       └─ RouterView
            ├─ TodayPage
            ├─ TrailPage
            ├─ ExplorePage
            └─ ProfilePage
```

### 4.2 视觉基准

参考：

- `<LOCAL_REFERENCE_ASSET>`
- 同目录 `implementation-*-960x640.png`
- 同目录 `design-tokens.css`

视觉要求：

- 深海军蓝背景、低饱和雾紫、暖金主文字。
- 使用真实星空背景图片，不用 CSS 绘制星空。
- 标题使用项目内置中文衬线字体。
- 图标使用项目内置 Phosphor 和 Material Design Icons 字体。
- 所有页面维持 `960 × 640` 信息密度。
- 颜色、字号、间距、圆角、阴影集中到 Token。

### 4.3 星图

使用 Chart.js Radar：

```text
[love, work, wealth, social, inner]
```

同一数组同时驱动：

- 雷达点位。
- 标签及分数。
- 中央综合分。
- `aria-label`。

### 4.4 Store

| Store | 状态与动作 |
|---|---|
| `app` | 启动、当前日期、全局错误 |
| `profile` | 资料读取和保存 |
| `dailyReading` | 今日结果、加载、心情保存 |
| `trail` | 历史读取、选中日期 |
| `explore` | 签运读取和抽取 |

## 5. CommandClient 契约

```ts
interface CommandClient {
  getAppState(): Promise<AppState>
  getProfile(): Promise<Profile | null>
  saveProfile(input: ProfileInput): Promise<Profile>
  getDailyReading(): Promise<DailyReading>
  saveMood(mood: Mood): Promise<MoodEntry>
  getTrail(days: number): Promise<TrailResponse>
  getDailyFortune(): Promise<DailyFortune | null>
  drawDailyFortune(): Promise<DailyFortune>
}
```

`TauriCommandClient` 只做 command 名称和参数映射。`MockCommandClient` 使用相同接口。

## 6. 项目数据

### 6.1 Profile

```ts
interface Profile {
  id: string
  guestId: string
  nickname: string
  birthday: string
  birthTime: string | null
  birthCity: string | null
  zodiac: ZodiacSign
  personalizationMode: "sunSignLite" | "natal"
  completeness: number
  createdAt: string
  updatedAt: string
}
```

完整度：

- 生日：40%。
- 昵称：20%。
- 出生时间：20%。
- 出生城市：20%。

### 6.2 DailyReading

```ts
interface DailyReading {
  id: string
  date: string
  weekday: string
  zodiac: ZodiacSign
  personalizationMode: "sunSignLite" | "natal"
  hero: {
    title: string
    subtitle: string
    theme: string
  }
  scores: {
    overall: number
    dimensions: ScoreDimensions
  }
  briefs: ReadingBrief[]
  echo: string
  lucky: LuckyHints
  source: {
    provider: string
    rulesVersion: string
    narrativeProvider: string
  }
}
```

### 6.3 Trail

```ts
interface TrailEntry {
  date: string
  weekday: string
  theme: string
  scores: DailyScores
  mood: Mood | null
  echo: string
}
```

### 6.4 DailyFortune

```ts
interface DailyFortune {
  id: string
  date: string
  grade: FortuneGrade
  title: string
  verse: string
  interpretation: string
  advice: string
}
```

## 7. Rust 服务

### 7.1 AppBootstrapService

- 初始化数据库和 migration。
- 读取本地资料。
- 返回引导状态。

### 7.2 ProfileService

- 校验生日格式和日期。
- 计算太阳星座。
- 根据资料计算个性化模式和完整度。
- 保存单机资料。

### 7.3 DailyReadingService

```text
读取资料
  -> 计算缓存键
  -> 命中 daily_readings：返回
  -> AstrologyProvider 获取天空快照
  -> 保存 sky_snapshots
  -> RuleEngine 生成分数和语义
  -> LuckyEngine 生成幸运提示
  -> NarrativeRenderer 生成文案
  -> 保存 daily_readings
  -> 返回 DTO
```

没有 API Key 或远程请求失败且没有缓存时，返回可重试错误，不生成未标记假数据。

### 7.4 TrailService

- 查询最近 N 天结果和心情。
- 不足七天时使用已有记录，不伪造历史。
- 计算平均分、主要心情和提升最大的维度。

### 7.5 DailyFortuneService

- 使用 `localDate + guestId + fortuneRulesVersion` 稳定选取签文。
- 先查询数据库；已存在则直接返回。
- 插入时依赖唯一约束防止并发重复抽取。

## 8. 星相与规则

### 8.1 AstrologyAPI

开发环境变量：

```text
STELLUNE_ASTROLOGY_API_KEY
```

第一版调用：

```text
POST https://json.astrologyapi.com/v1/planets/tropical
Header: x-astrologyapi-key
```

请求使用用户本地日期、固定中午时间、资料城市坐标或默认城市坐标。第一版内置少量城市坐标映射；未知城市使用应用配置的默认位置并在来源信息中标记。

adapter 将供应商数据标准化为 `PlanetPosition`，包括把字符串形式的 `isRetro` 转换为布尔值。

### 8.2 轻量模式

- 太阳星座中点作为参考度数。
- 计算行星与参考点的合、六合、刑、拱、冲。
- 计算行星相对太阳星座的整宫位置。
- 相位容许度默认 `6°`。

### 8.3 评分

```text
dimension = clamp(75 + rule impacts, 55, 92)
```

综合分：

```text
love 20% + work 25% + wealth 15% + social 20% + inner 20%
```

每项结果保留规则 id 和加减值。

### 8.4 文案

本次实现 `TemplateNarrativeRenderer`：

- 输入规则产生的主题关键词、正向提示和注意事项。
- 按确定性模板组合标题、简报和宇宙回声。
- 文案随星相事实变化，但不会由随机数生成。

保留 `NarrativeProvider` 接口，后续可接入大模型。

## 9. SQLite

初始 migration 创建：

```text
profiles
sky_snapshots
daily_readings
mood_entries
daily_fortunes
```

为减少第一版表数量，规则 trace 作为 `daily_readings.trace_json` 保存。

关键唯一约束：

- `daily_readings.cache_key`
- `mood_entries.local_date`
- `daily_fortunes(local_date, guest_id, rules_version)`

Token 不进入数据库。

## 10. Playwright E2E

### 10.1 Mock 结构

```text
tests/e2e/
  fixtures/
    profile.json
    daily-reading.json
    trail.json
    daily-fortune.json
  mocks/
    command-client.ts
  specs/
    onboarding.spec.ts
    today.spec.ts
    trail.spec.ts
    explore.spec.ts
    profile.spec.ts
    error-states.spec.ts
```

E2E 模式：

```text
vite --mode e2e
```

只有该模式加载 `MockCommandClient`。

### 10.2 测试固定条件

- Chromium。
- 视口 `960 × 640`。
- 时区 `Asia/Shanghai`。
- 固定时间 `2026-07-23T13:00:00+08:00`。
- 每个测试独立 browser context。
- 禁止真实外部网络。

### 10.3 验收用例

1. 没有资料时进入引导，填写生日后进入今日页。
2. 今日页展示日期、星座、78 分、五维数据、简报和幸运提示。
3. 选择并保存心情后显示保存状态。
4. 星迹页可选择七天中的任意一天并更新详情。
5. 探索页首次抽签后显示结果，再次进入结果不变。
6. 我的页面展示资料完整度并可修改资料。
7. 星相失败显示可重试错误。
8. 页面无横向或纵向溢出，主导航始终可见。

## 11. 文件范围

本次创建：

```text
package.json
vite.config.ts
tsconfig*.json
playwright.config.ts
index.html
src/**
src-tauri/**
tests/e2e/**
public/assets/**
design-qa.md
```

## 12. 验证命令

```powershell
npm install
npm run typecheck
npm run build
npm run test:e2e
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

## 13. 完成标准

- Vue 浏览器版本可运行。
- Tauri Rust 项目可编译和通过测试。
- 五个页面与引导流程可操作。
- Playwright 核心用例通过且不访问真实服务。
- 页面在 `960 × 640` 下无溢出。
- 设计 QA 对比源设计后无 P0/P1/P2 问题。
- 源码、fixture、日志和文档中不含真实 Token。
