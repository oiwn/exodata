import { expect, test, type Page } from "@playwright/test";

type ErrorCapture = {
  consoleErrors: string[];
  pageErrors: string[];
};

function captureClientErrors(page: Page): ErrorCapture {
  const consoleErrors: string[] = [];
  const pageErrors: string[] = [];

  page.on("console", (msg) => {
    if (msg.type() === "error") {
      consoleErrors.push(msg.text());
    }
  });

  page.on("pageerror", (err) => {
    pageErrors.push(String(err));
  });

  return { consoleErrors, pageErrors };
}

async function expectNoClientErrors(page: Page, capture: ErrorCapture) {
  await page.waitForLoadState("networkidle");
  await page.waitForTimeout(250);

  expect(
    capture.consoleErrors,
    `Unexpected console errors:\n${capture.consoleErrors.join("\n")}`,
  ).toEqual([]);
  expect(
    capture.pageErrors,
    `Unexpected page errors:\n${capture.pageErrors.join("\n")}`,
  ).toEqual([]);
}

test.beforeEach(async ({ request }) => {
  // cargo-leptos owns server startup; wait until it is reachable.
  await expect
    .poll(
      async () => {
        try {
          return (await request.get("/")).status();
        } catch {
          return 0;
        }
      },
      {
        timeout: 90_000,
        intervals: [250, 500, 1_000, 2_000, 3_000],
      },
    )
    .toBe(200);
});

test("SSR + hydration works on /stellarhosts", async ({ page }) => {
  const capture = captureClientErrors(page);

  await page.goto("/stellarhosts");

  await expect(
    page.getByRole("heading", { name: /stellar hosts catalog/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /select columns|hide column selector/i }),
  ).toBeVisible();
  await expect(page.locator("table thead th").first()).toBeVisible();

  await expectNoClientErrors(page, capture);
});

test("SSR + hydration works on /exoplanets", async ({ page }) => {
  const capture = captureClientErrors(page);

  await page.goto("/exoplanets");

  await expect(
    page.getByRole("heading", { name: /exoplanets catalog/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /select columns|hide column selector/i }),
  ).toBeVisible();
  await expect(page.locator("table thead th").first()).toBeVisible();

  await expectNoClientErrors(page, capture);
});

test("metadata is available after / -> client navigation to /stellarhosts", async ({
  page,
}) => {
  const capture = captureClientErrors(page);

  await page.goto("/");
  await page.getByRole("link", { name: /stellar hosts/i }).first().click();

  await expect(page).toHaveURL(/\/stellarhosts/);
  await expect(
    page.getByRole("heading", { name: /stellar hosts catalog/i }),
  ).toBeVisible();

  const selectorToggle = page.getByRole("button", {
    name: /select columns|hide column selector/i,
  });
  await selectorToggle.click();
  await expect(page.getByRole("button", { name: /select all/i })).toBeVisible();
  await expect(page.locator("label").filter({ hasText: "st_refname" })).toBeVisible();

  await expectNoClientErrors(page, capture);
});
