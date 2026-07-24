import { expect, test, type Page } from "@playwright/test";

async function assertFixedViewport(page: Page) {
  const metrics = await page.evaluate(() => ({
    innerWidth: window.innerWidth,
    innerHeight: window.innerHeight,
    bodyWidth: document.body.scrollWidth,
    bodyHeight: document.body.scrollHeight,
    htmlWidth: document.documentElement.scrollWidth,
    htmlHeight: document.documentElement.scrollHeight,
  }));
  expect(metrics.innerWidth).toBe(960);
  expect(metrics.innerHeight).toBe(640);
  expect(metrics.bodyWidth).toBeLessThanOrEqual(960);
  expect(metrics.bodyHeight).toBeLessThanOrEqual(640);
  expect(metrics.htmlWidth).toBeLessThanOrEqual(960);
  expect(metrics.htmlHeight).toBeLessThanOrEqual(640);
}

test.beforeEach(async ({ page }) => {
  await page.route("https://**/*", (route) => route.abort());
});

test("首次启动只要求生日，并进入今日页", async ({ page }) => {
  await page.goto("/?scenario=onboarding#/onboarding");
  await expect(page.getByTestId("window-minimize")).toBeVisible();
  await expect(page.getByTestId("window-close")).toBeVisible();
  const dragRegion = page.locator("[data-tauri-drag-region]");
  await expect(dragRegion).toHaveCSS("height", "32px");
  await expect(dragRegion).toHaveCSS("width", "960px");
  await expect(page.getByTestId("onboarding-form")).toBeVisible();
  await page.getByTestId("birthday-input").click();
  await page.getByTestId("calendar-title").click();
  while ((await page.getByTestId("calendar-year-1998").count()) === 0) {
    await page.getByTestId("calendar-previous").click();
  }
  await page.getByTestId("calendar-year-1998").click();
  await page.getByTestId("calendar-month-10").click();
  await page.getByTestId("calendar-day-1998-10-08").click();
  await expect(page.getByText("天秤座")).toBeVisible();
  await page.getByTestId("onboarding-submit").click();
  await expect(page.getByTestId("today-title")).toHaveText(
    "把光留给真正值得回应的地方",
  );
  await expect(page.getByText("今日宇宙简报")).toBeVisible();
  await assertFixedViewport(page);
});

test("WebView 可用宽度略小时窗口控制仍完整位于视口内", async ({ page }) => {
  await page.setViewportSize({ width: 930, height: 640 });
  await page.goto("/#/explore");

  const closeButton = page.getByTestId("window-close");
  await expect(closeButton).toBeVisible();
  const closeBox = await closeButton.boundingBox();
  expect(closeBox).not.toBeNull();
  expect(closeBox!.x + closeBox!.width).toBeLessThanOrEqual(930);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(930);
});

test("今日页展示稳定数据，并能记录心情", async ({ page }) => {
  await page.goto("/#/");
  await expect(page.getByText("78", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("幸运色 · 晨雾蓝")).toBeVisible();
  await expect(page.getByText("幸运数字 · 9")).toBeVisible();
  await page.getByTestId("mood-平静").click();
  await expect(page.getByTestId("save-mood")).toBeEnabled();
  await page.getByTestId("save-mood").click();
  await expect(page.getByTestId("save-mood")).toContainText("今日星迹已点亮");
  await assertFixedViewport(page);
});

test("星迹页可切换七日记录并更新回望内容", async ({ page }) => {
  await page.goto("/#/trail");
  await expect(page.getByTestId("trail-average")).toHaveText("76");
  await page.getByTestId("trail-day-2026-07-17").click();
  await expect(page.getByText("7月17日 · 呼吸")).toBeVisible();
  await expect(
    page.getByText("慢一点也没关系，先让心重新找到可以停靠的位置。"),
  ).toBeVisible();
  await assertFixedViewport(page);
});

test("探索页可以重新抽签并保存当天最新结果", async ({ page }) => {
  await page.goto("/#/explore");
  await expect(page.getByText("轻触抽取今日签")).toBeVisible();
  const resonanceSign = page.getByRole("combobox", { name: "选择对方星座" });
  await expect(resonanceSign).toHaveText("双子座");
  await expect(page.getByTestId("compatibility-score")).toHaveText("91");
  await expect(page.getByTestId("compatibility-title")).toHaveText(
    "风与风的默契",
  );
  await resonanceSign.press("ArrowDown");
  await expect(page.getByRole("listbox")).toBeVisible();
  await resonanceSign.press("Enter");
  await expect(resonanceSign).toHaveText("巨蟹座");
  await expect(page.getByTestId("compatibility-score")).toHaveText("48");
  await expect(page.getByTestId("compatibility-title")).toHaveText(
    "差异里的吸引力",
  );
  await resonanceSign.click();
  await expect(
    page.getByRole("option", { name: "巨蟹座", selected: true }),
  ).toBeVisible();
  await page.getByRole("option", { name: "狮子座" }).click();
  await expect(resonanceSign).toHaveText("狮子座");
  await expect(page.getByTestId("compatibility-score")).toHaveText("95");

  await expect(page.getByTestId("emotion-guide-title")).toHaveText(
    "先听见此刻的自己",
  );
  await expect(page.getByTestId("emotion-开心")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await page.getByTestId("emotion-疲惫").click();
  await expect(page.getByTestId("emotion-guide-title")).toHaveText(
    "有力量，也要允许自己停一停",
  );
  await expect(page.getByTestId("emotion-guide-action")).toHaveText(
    "取消或推迟一件今天并不必要的任务。",
  );
  await expect(page.getByTestId("emotion-疲惫")).toHaveAttribute(
    "aria-pressed",
    "true",
  );

  await resonanceSign.press("ArrowDown");
  await resonanceSign.press("End");
  await expect(page.getByRole("option", { name: "双鱼座" })).toBeInViewport();
  await expect(page.getByRole("option", { name: "双鱼座" })).toHaveClass(
    /active/,
  );
  await resonanceSign.press("Enter");
  await expect(resonanceSign).toHaveText("双鱼座");
  await expect(page.getByTestId("compatibility-score")).toHaveText("53");
  await resonanceSign.click();
  await expect(
    page.getByRole("option", { name: "双鱼座", selected: true }),
  ).toBeInViewport();
  await resonanceSign.press("Escape");

  const fortuneCard = page.getByTestId("daily-fortune-card");
  await expect(page.locator(".fortune-card-scene")).toHaveCSS("perspective", "900px");
  await expect(fortuneCard).toHaveAttribute("data-revealed", "false");
  await expect(fortuneCard).toHaveCSS("animation-name", "none");
  await expect(fortuneCard).toHaveCSS("--motion-fortune-reveal-angle", "1260deg");
  await expect(fortuneCard).toHaveCSS("transform-style", "preserve-3d");
  await expect(page.locator(".fortune-card-front")).toHaveCSS(
    "backface-visibility",
    "hidden",
  );
  await fortuneCard.click();
  await expect(fortuneCard).toHaveAttribute("data-revealed", "true");
  await expect(page.getByRole("button", { name: "星轨正在旋转…" })).toBeDisabled();
  await expect(fortuneCard).toHaveCSS("animation-name", "fortune-reveal");
  await expect(fortuneCard).toHaveCSS("animation-duration", "1.6s");
  await expect(page.getByTestId("draw-fortune")).toContainText("重新抽签", {
    timeout: 2500,
  });
  expect(
    await fortuneCard.evaluate((element) => getComputedStyle(element).transform),
  ).not.toBe("none");
  await expect(page.getByTestId("fortune-title")).toHaveText("云开见月");
  await expect(page.getByTestId("fortune-number")).toHaveText("第 47 签 · 上上签");
  await page.getByTestId("draw-fortune").click();
  await expect(page.getByTestId("draw-fortune")).toBeDisabled();
  await expect(page.getByTestId("draw-fortune")).toContainText("星轨正在旋转…");
  await expect(fortuneCard).toHaveCSS("animation-name", "fortune-reveal");
  await expect(page.getByTestId("draw-fortune")).toContainText("重新抽签", {
    timeout: 2500,
  });
  await expect(page.getByTestId("fortune-title")).toHaveText("春水绕石");
  await expect(page.getByTestId("fortune-number")).toHaveText("第 48 签 · 中上签");
  await expect(page.getByText("每次重抽都会更新今日保存的签")).toBeVisible();
  await page.getByTestId("nav-profile").click();
  await page.getByTestId("nav-today").click();
  await expect(page.getByTestId("mood-疲惫")).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.getByTestId("save-mood")).toContainText("今日星迹已点亮");
  await page.getByTestId("nav-explore").click();
  await expect(page.getByTestId("fortune-title")).toHaveText("春水绕石");
  await expect(page.getByText("每次重抽都会更新今日保存的签")).toBeVisible();
  await assertFixedViewport(page);
});

test("减少动态效果环境仍保留完整多圈翻牌且不会卡住", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#/explore");
  const fortuneCard = page.getByTestId("daily-fortune-card");

  await fortuneCard.click();

  await expect(fortuneCard).toHaveAttribute("data-revealed", "true");
  await expect(fortuneCard).toHaveCSS(
    "animation-name",
    "fortune-reveal-reduced",
  );
  await expect(fortuneCard).toHaveCSS("animation-duration", "1.6s");
  await expect(fortuneCard).toHaveCSS(
    "--motion-fortune-reveal-angle",
    "1260deg",
  );
  await expect(page.getByTestId("fortune-title")).toHaveText("云开见月");
  await expect(page.getByTestId("draw-fortune")).toContainText("重新抽签", {
    timeout: 2500,
  });
});

test("探索页请求失败时恢复已确认内容并显示提示", async ({ page }) => {
  await page.goto("/?scenario=explore-error#/explore");
  const resonanceSign = page.getByRole("combobox", { name: "选择对方星座" });
  await expect(page.getByTestId("compatibility-score")).toHaveText("91");

  await resonanceSign.click();
  await page.getByRole("option", { name: "巨蟹座" }).click();
  await expect(page.getByTestId("compatibility-error")).toContainText(
    "关系共鸣暂时没有回应。",
  );
  await expect(resonanceSign).toHaveText("双子座");
  await expect(page.getByTestId("compatibility-score")).toHaveText("91");

  await page.getByTestId("emotion-疲惫").click();
  await expect(page.getByTestId("emotion-error")).toContainText(
    "情绪说明书暂时没有回应。",
  );
  await expect(page.getByTestId("emotion-疲惫")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await expect(page.getByTestId("emotion-guide-title")).toHaveText(
    "先听见此刻的自己",
  );
  await assertFixedViewport(page);
});

test("快速切换心情时最后一次选择同时写入探索页和今日页", async ({
  page,
}) => {
  await page.goto("/?scenario=rapid-mood#/explore");
  await page.getByTestId("emotion-疲惫").click();
  await page.getByTestId("emotion-低落").click();
  await expect(page.getByTestId("emotion-guide-title")).toHaveText(
    "低落不是对今天的结论",
  );
  await page.getByTestId("nav-today").click();
  await expect(page.getByTestId("mood-低落")).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await assertFixedViewport(page);
});

test("旧心情生成完成时不会覆盖随后失败的新选择", async ({ page }) => {
  await page.goto("/?scenario=rapid-mood-failure#/explore");
  await page.getByTestId("emotion-疲惫").click();
  await page.getByTestId("emotion-低落").click();
  await expect(page.getByTestId("emotion-error")).toContainText(
    "最新心情暂时保存失败。",
  );
  await expect(page.getByTestId("emotion-疲惫")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await expect(page.getByTestId("emotion-低落")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await page.getByTestId("nav-today").click();
  await expect(page.getByTestId("mood-疲惫")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await expect(page.getByTestId("mood-低落")).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  await expect(page.getByTestId("save-mood")).toContainText("点亮今日星迹");
  await assertFixedViewport(page);
});

test("我的页可以修改本地个人资料", async ({ page }) => {
  await page.goto("/#/profile");
  await expect(page.getByText("星际旅人")).toBeVisible();
  await page.getByTestId("edit-profile").click();
  const form = page.getByTestId("profile-form");
  await form.getByLabel("称呼").fill("月光旅人");
  await page.getByTestId("profile-birth-time").click();
  await page.getByTestId("time-hour-08").click();
  await page.getByTestId("time-minute-30").click();
  await page.getByTestId("time-confirm").click();
  await page.getByTestId("time-clear-inline").click();
  await expect(page.getByTestId("profile-birth-time")).toContainText("未填写");
  await page.getByTestId("profile-birth-time").click();
  await page.getByTestId("time-hour-08").click();
  await page.getByTestId("time-minute-30").click();
  await page.getByTestId("time-confirm").click();
  await form.getByLabel("出生城市").fill("杭州");
  await page.getByTestId("save-profile").click();
  await expect(page.getByTestId("profile-change-confirmation")).toBeVisible();
  await page.getByTestId("confirm-profile-recalibration").click();
  await expect(page.getByText("月光旅人")).toBeVisible();
  await expect(page.getByText("100%")).toBeVisible();
  await expect(page.getByText("杭州")).toBeVisible();
  await page.getByTestId("nav-today").click();
  await expect(page.getByTestId("today-title")).toHaveText("新资料校准后的星图已抵达");
  await assertFixedViewport(page);
});

test("只修改称呼时直接保存且不触发今日校准确认", async ({ page }) => {
  await page.goto("/#/profile");
  await page.getByTestId("edit-profile").click();
  const form = page.getByTestId("profile-form");
  await form.getByLabel("称呼").fill("微光旅人");
  await page.getByTestId("save-profile").click();
  await expect(page.getByTestId("profile-change-confirmation")).toHaveCount(0);
  await expect(page.getByText("微光旅人")).toBeVisible();
  await assertFixedViewport(page);
});

test("切换已访问的标签页时保留内容并静默刷新", async ({ page }) => {
  await page.goto("/?scenario=slow-tabs#/explore");
  await expect(page.getByRole("heading", { name: "今日一签" })).toBeVisible();

  await page.getByTestId("nav-trail").click();
  await expect(page.getByTestId("trail-average")).toBeVisible();

  await page.evaluate(() => {
    const state = window as typeof window & { __tabLoadingFlashes?: string[] };
    state.__tabLoadingFlashes = [];
    new MutationObserver(() => {
      const text = document.body.textContent ?? "";
      for (const label of ["答案正在靠近", "正在收拢星光"]) {
        if (text.includes(label) && !state.__tabLoadingFlashes?.includes(label)) {
          state.__tabLoadingFlashes?.push(label);
        }
      }
    }).observe(document.body, { childList: true, subtree: true });
  });

  await page.getByTestId("nav-explore").click();
  await expect(page.getByRole("heading", { name: "今日一签" })).toBeVisible();

  await page.getByTestId("nav-trail").click();
  await expect(page.getByTestId("trail-average")).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        (window as typeof window & { __tabLoadingFlashes?: string[] })
          .__tabLoadingFlashes ?? [],
    ),
  ).toEqual([]);
  await assertFixedViewport(page);
});

test("真实天象失败时展示可重试错误状态", async ({ page }) => {
  await page.goto("/?scenario=astrology-error#/");
  await expect(page.getByText("今天的星光暂时迟到")).toBeVisible();
  await expect(page.getByText("真实星相暂时没有抵达，请稍后再试。")).toBeVisible();
  await expect(page.getByRole("button", { name: "重新读取" })).toBeVisible();
  await assertFixedViewport(page);
});
