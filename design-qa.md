# Stellune Design QA

## 全局界面与探索页验收

**Comparison target**

- Source visual truth: `<LOCAL_REFERENCE_ASSET>`
- Rendered implementation: `<PROJECT_ROOT>\docs\qa\screenshots\today-960x640.png`
- Full-view comparison: `<PROJECT_ROOT>\docs\qa\today-reference-comparison.png`
- Native-select issue reference: `<USER_REFERENCE_ASSET>`
- P3 custom-select implementation: `<PROJECT_ROOT>\docs\qa\screenshots\explore-p3-select-open.png`
- Select-state comparison: `<PROJECT_ROOT>\docs\qa\explore-select-comparison.png`
- Local route: `http://127.0.0.1:4173/#/`
- State: 已完成游客资料、今日简报已生成、今日心情尚未保存

**Normalization**

- Target CSS viewport: `960 × 640`
- Device scale factor: `1`
- Source pixels: `1488 × 1058`
- Implementation pixels: `960 × 640`
- Normalization: source uses cover scaling and centered crop into `960 × 640`; implementation is a direct Playwright capture at `960 × 640`. Both normalized views are placed side by side in the comparison image.

**Findings**

- No remaining actionable P0/P1/P2 mismatch.
- [P3] The source uses a custom luminous constellation treatment with larger point glows; the implementation uses Chart.js points and restrained glow. This is acceptable for the MVP because the plotted vertices now correspond to real dimension scores, remain sharp at the fixed window size, and preserve the reference hierarchy.
- [P3] Generated daily copy intentionally differs from the static source copy. The content structure, line hierarchy, tone, and region density remain aligned with the design.

**Required fidelity surfaces**

- Fonts and typography: the brand mark uses the bundled ZCOOL XiaoWei face; headings and body copy use a complete system Song serif stack so all generated Chinese characters render correctly. Weights, hierarchy, tracking, wrapping, and truncation were checked at `960 × 640`. The dynamic hero title stays on one line.
- Spacing and layout rhythm: fixed frame, left navigation, main reading region, right brief panel, and bottom lucky strip align to the compact target. No page-level horizontal or vertical overflow is present in the Playwright assertions.
- Colors and visual tokens: midnight navy, mist purple, champagne gold, cool gray-blue borders, translucent panels, and glow opacity are consistently driven by project design tokens.
- Image and asset fidelity: the original star-field raster background, bundled icon fonts, zodiac font, and generated desktop app icon are used. No visible emoji, handcrafted SVG placeholder, or text-symbol icon substitutes remain.
- Copy and content: the UI preserves the reference information architecture while using deterministic generated reading fixtures. All Chinese copy in the visible fixture renders without replacement glyphs.
- Navigation state: the active tab now uses the reference's flat mist-purple surface, 2px inset edge, and icon glow; the previous gradient and detached indicator were removed.
- Draw interaction: the fortune card keeps both faces mounted in a 3D scene, rotates `3.5` turns (`1260deg`) over `1600ms`, and fades the result copy in during the final `280ms`. The revealed face remains readable within the fixed frame.
- Profile recalibration: the custom confirmation panel fits within the `960 × 640` window, clearly explains that only today's brief changes, and preserves the same visual language as the profile dialog.
- Async consistency: outdated reading requests are rejected at the database transaction boundary and ignored by the frontend request generation guard, so a slow old-profile response cannot replace the latest profile version.
- Tab transitions: Trail and Explore keep cached content mounted during background refresh, avoiding full-page loading flashes after the first visit.
- Explore select: the Windows-native popup has been replaced by a project-native P3 combobox. The open layer uses the existing midnight panel and mist-purple focus tokens, keeps the selected option on a restrained glow surface, uses the bundled icon font for its gold check, and overlays the following card without clipping. Mouse selection, click-outside close, `ArrowUp`/`ArrowDown`, `Home`/`End`, `Enter`/`Space`, `Escape`, and `Tab` behavior are implemented with combobox/listbox ARIA states.

**Focused region evidence**

- The normalized `1920 × 640` full-view comparison preserves the title, radar labels, navigation icons, right-panel copy, and footer at readable size.
- The Explore dropdown was compared in the same open state using `<PROJECT_ROOT>\docs\qa\explore-select-comparison.png`. The native gray popup, hard white text, and platform selection bar are gone; the P3 layer now belongs to the same visual system as the surrounding card.

**Comparison history**

1. Initial pass — result blocked.
   - [P2] The dynamic hero title wrapped its final character onto a second line.
   - [P2] The radar polygon was visibly smaller than the normalized source constellation.
2. Fixes applied.
   - Reduced the hero title from `34px / 0.11em` tracking to `31px / 0.07em`.
   - Reduced Chart.js radar layout padding from `26px` to `8px`, preserving score-driven vertex positions while enlarging the plotted region.
3. Second pass — result blocked.
   - [P2] The bundled display font did not contain a usable `回` glyph, producing visible replacement squares in the hero and brief panel.
4. Fix applied.
   - Limited ZCOOL XiaoWei to the brand mark and changed generated Chinese headings/body copy to a complete Song serif system stack.
5. Post-fix pass.
   - Evidence: `<PROJECT_ROOT>\docs\qa\today-reference-comparison.png`
   - The title remains intact, the constellation is proportionally closer to the source, all Chinese characters render correctly, and no P0/P1/P2 findings remain.
6. Explore/profile interaction pass.
   - The Explore screen was inspected before and after drawing in the in-app browser at `960 × 640`.
   - The card front and revealed result remain centered without clipping; the selected Explore/Profile tabs now match the reference treatment.
   - The profile recalibration confirmation was inspected with a changed birth time; copy and both actions remain visible without overflow.
7. Explore custom-select pass.
   - The user-provided native-select screenshot and the implemented open state were placed together in `<PROJECT_ROOT>\docs\qa\explore-select-comparison.png`.
   - The menu remains above the emotion card, its selected state and check are visually distinct, and the trigger closes after keyboard selection.
   - The in-app browser console contained no warnings or errors.

**Primary interactions tested**

- First-run birthday onboarding and automatic zodiac preview
- Today reading render and mood save
- Seven-day trail selection
- One draw per day and stable draw result
- 3.5-turn, 1600ms two-sided fortune-card reveal
- Profile edit confirmation, calculation-version bump, and local persistence contract
- Nickname-only profile edit without a calculation-version bump
- Out-of-order profile reading write rejection, one-current-row invariant, and Trail current-version filtering
- Astrology provider failure and retry state
- Custom Explore combobox mouse and keyboard selection
- Navigation among Today, Trail, Explore, and Profile

**Browser and runtime checks**

- Browser-rendered implementation inspected in the in-app browser at `http://127.0.0.1:4173/`.
- Console checked after Today, Trail, Explore, Profile, and Onboarding states; only Vite connection debug messages were present.
- Playwright fixed-window suite: 9 passed.
- Rust unit/integration suite: 11 passed.

**Follow-up polish**

- If the MVP later adds a custom canvas constellation renderer, point halos and radial guide decoration can be brought even closer to the source without changing the score contract.

## P3 出生日期与时间选择器验收

- Source visual truth: `<LOCAL_GENERATED_ASSET>`
- Implementation screenshots:
  - `<PROJECT_ROOT>\docs\qa\screenshots\p3-date-picker-profile.png`
  - `<PROJECT_ROOT>\docs\qa\screenshots\p3-time-picker-profile.png`
- Combined comparison: `<PROJECT_ROOT>\docs\qa\screenshots\p3-date-picker-comparison.png`
- Viewport: `960 × 640` CSS px
- Source: `1536 × 1024` px，按 `960 × 640` 等比归一化
- Implementation: `960 × 640` px，`devicePixelRatio: 1`
- State: “我的”资料编辑弹窗，日期或时间选择器展开

## Full-view comparison evidence

归一化后的并排图显示：实现保留了参考图的深蓝半透明面板、紫色焦点描边、金色选中圆环、月历标题与左右导航、底部“清除/今天”操作。弹层与固定窗口、资料编辑卡、左侧导航的比例协调，日期弹层边界为窗口内 `bottom: 634px`。

参考图是概念稿，没有完整呈现出生城市字段；实现保留现有产品字段和两列表单，这是既有数据结构约束，不属于视觉回退。

## Focused region evidence

时间选择器作为参考图未完整给出的扩展状态单独验收。实现使用同一 P3 容器、标题层级、紫色选中态和金色操作文字；资料页向下展开，边界为 `top: 385px`、`bottom: 635px`，没有越出固定窗口。

## Required fidelity surfaces

- Fonts and typography: 沿用项目宋体显示体系；标题、字段、日历数字和辅助文字层级清晰，无截断或异常换行。
- Spacing and layout rhythm: 日期网格、周标题、页头和页脚间距一致；弹层与触发器对齐，关键操作未被窗口裁切。
- Colors and visual tokens: 颜色、尺寸、圆角和阴影均由 `tokens.css` 的语义 token 驱动；选中、悬停、焦点状态与参考图同属雾紫和浅金体系。
- Image quality and asset fidelity: 该控件没有自定义位图资产；图标使用项目现有 Phosphor 图标库，没有 Unicode 或临时 CSS 图标。
- Copy and content: 日期使用 `YYYY/MM/DD`，提供“清除/今天”；时间提供“清除/完成”，与资料填写语义一致。

## Comparison history

1. First pass — blocked
   - [P1] `.profile-modal header` 误作用于内嵌日历头，年月标题被压成三行。
   - [P2] 资料弹窗垂直居中，日期弹层底部超出 `640px`。
2. Fixes
   - 将资料头部规则收窄为 `.profile-modal > header`。
   - 通过 `--profile-modal-top` 将资料弹窗上移到参考图位置。
   - 资料页时间弹层改为向下展开，并压缩滚动列高度。
3. Final pass
   - 年月单行展示。
   - 日期与时间弹层均处于 `960 × 640` 边界内。
   - 浏览器控制台 error/warning 为 0。

## Findings

没有剩余 P0、P1 或 P2 问题。

## Follow-up polish

- [P3] 参考概念稿的日期弹层更紧凑；当前实现稍宽，以提高 42 个日期按钮的可读性和键盘焦点面积。

## Verification

- `npm run typecheck`
- `npm run test`
- `npm run test:e2e` — 12 passed
- 资料修改定向回归 — 2 passed，包含时间清除后重新选择
- 浏览器控制台 error/warning — 0

final result: passed
