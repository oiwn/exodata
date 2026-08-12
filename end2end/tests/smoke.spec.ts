import {
  expect,
  test,
  type APIRequestContext,
  type Page,
} from "@playwright/test";

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

async function firstCatalogValue(
  request: APIRequestContext,
  endpoint: string,
  field: string,
): Promise<string> {
  const response = await request.get(`${endpoint}?limit=1`);
  expect(response.ok()).toBeTruthy();

  const payload = await response.json();
  const value = payload.data?.[0]?.[field];
  expect(typeof value).toBe("string");

  return value;
}

async function expectNoDocumentOverflow(page: Page) {
  const dimensions = await page.evaluate(() => ({
    viewportWidth: window.innerWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(
    dimensions.viewportWidth + 1,
  );
}

async function expectTableWrapperContained(page: Page) {
  const wrapper = page.locator(
    ".planet-provenance__table-wrap, .host-provenance__table-wrap",
  );

  await expect(wrapper).toBeVisible();

  const dimensions = await wrapper.evaluate((element) => {
    const style = window.getComputedStyle(element);
    return {
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
      viewportWidth: window.innerWidth,
      overflowX: style.overflowX,
    };
  });

  expect(dimensions.clientWidth).toBeLessThanOrEqual(
    dimensions.viewportWidth + 1,
  );
  expect(dimensions.scrollWidth).toBeGreaterThanOrEqual(dimensions.clientWidth);
  expect(dimensions.overflowX).toBe("auto");
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

test("catalog table interactions preserve query state", async ({ page }) => {
  for (const route of ["/stellarhosts", "/exoplanets"]) {
    const capture = captureClientErrors(page);
    await page.goto(route);

    await expect(page.locator("table thead th").first()).toBeVisible();
    await page.locator("table thead th").first().click();
    await expect(page).toHaveURL(new RegExp(`${route}\\?sort=`));

    const filter = page.locator("table thead input").first();
    await filter.fill("Kepler");
    await filter.press("Enter");
    await expect(page).toHaveURL(/filter=Kepler/);

    await filter.fill("");
    await filter.press("Enter");
    await expect(page).not.toHaveURL(/filter=/);

    await page
      .getByRole("button", { name: /select columns|hide column selector/i })
      .click();
    await expect(page.locator('input[type="checkbox"]').first()).toBeVisible();

    await page.getByRole("button", { name: "Next" }).last().click();
    await expect(page).toHaveURL(/page=2/);
    await expectNoClientErrors(page, capture);
  }
});

test("catalog tables return 404 for invalid and out-of-range pages", async ({
  page,
}) => {
  for (const route of ["/stellarhosts", "/exoplanets"]) {
    for (const pageParam of ["0", "not-a-page", "999999"]) {
      const response = await page.goto(`${route}?page=${pageParam}`);
      expect(response?.status()).toBe(404);
      await expect(page.getByRole("heading", { name: "Not Found" })).toBeVisible();
    }
  }
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

test("detail provenance sections stay within a mobile viewport", async ({
  page,
  request,
}) => {
  const exoplanet = await firstCatalogValue(request, "/rest/exoplanets", "pl_name");
  const host = await firstCatalogValue(request, "/rest/stellarhosts", "hostname");
  const capture = captureClientErrors(page);

  await page.setViewportSize({ width: 375, height: 812 });

  for (const route of [
    `/exoplanets/${encodeURIComponent(exoplanet)}`,
    `/stellarhosts/${encodeURIComponent(host)}`,
  ]) {
    await page.goto(route);
    await expect(
      page.getByText("Evidence Summary", { exact: true }),
    ).toBeVisible();
    await expectNoDocumentOverflow(page);
    await expectTableWrapperContained(page);
  }

  await expectNoClientErrors(page, capture);
});
