# Stellune Design QA

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

final result: passed
