import { mkdirSync } from "node:fs";
import { expect, test } from "@playwright/test";

const outputDir = "docs/qa/screenshots";

test("固定窗口视觉验收快照", async ({ page }) => {
  mkdirSync(outputDir, { recursive: true });

  const pages = [
    { route: "/#/", name: "today", title: "把光留给真正值得回应的地方" },
    { route: "/#/trail", name: "trail", title: "时间经过，也留下了属于你的星光" },
    { route: "/#/explore", name: "explore", title: "沿着好奇，听见另一种答案" },
    { route: "/#/profile", name: "profile", title: "我的星迹档案" },
  ];

  for (const item of pages) {
    await page.goto(item.route);
    await expect(page.getByRole("heading", { level: 1 })).toHaveText(item.title);
    await page.screenshot({
      path: `${outputDir}/${item.name}-960x640.png`,
      fullPage: false,
    });
  }

  await page.goto("/?scenario=onboarding#/onboarding");
  await expect(page.getByTestId("onboarding-form")).toBeVisible();
  await page.screenshot({
    path: `${outputDir}/onboarding-960x640.png`,
    fullPage: false,
  });
});
