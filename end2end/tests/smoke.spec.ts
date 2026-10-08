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

test("SEO detail metadata is clean during client navigation", async ({ page, request }) => {
  const capture = captureClientErrors(page);
  const planet = await firstCatalogValue(request, "/rest/exoplanets", "pl_name");
  const host = await firstCatalogValue(request, "/rest/stellarhosts", "hostname");
  for (const prefix of ["", "/zh-CN", "/ja"]) {
    const lang = prefix.slice(1) || "en";
    for (const [entity, name] of [["exoplanets", planet], ["stellarhosts", host]]) {
      const validPath = `${prefix}/${entity}/${encodeURIComponent(name)}`;
      const missingPath = `${prefix}/${entity}/Missing-seo-record`;
      const response = await page.goto(validPath);
      expect(response?.status()).toBe(200);
      const serverHtml = await response!.text();
      const serverHead = serverHtml.split("</head>")[0];
      expect(serverHead.match(/name="description"/g)).toHaveLength(1);
      expect(serverHead.match(/rel="canonical"/g)).toHaveLength(1);
      const htmlTag = serverHtml.match(/<html\b[^>]*>/)![0];
      expect(htmlTag.match(/\blang=/g)).toHaveLength(1);
      expect(htmlTag).toContain(`lang="${lang}"`);
      await page.waitForFunction(() => !document.documentElement.classList.contains("pre-hydration"));
      await expect(page.locator("html")).toHaveAttribute("lang", lang);
      await expect(page.locator('head meta[name="description"]')).toHaveCount(1);
      await expect(page.locator('head link[rel="canonical"]')).toHaveCount(1);
      const title = `${name} ${entity === "exoplanets" ? "Exoplanet" : "Stellar Host"} | Exodata`;
      await expect(page).toHaveTitle(title);
      const encodedName = encodeURIComponent(name).replace(/[!'()*]/g, (character) => `%${character.charCodeAt(0).toString(16).toUpperCase()}`);
      const canonical = `https://exodata.space/${entity}/${encodedName}`;
      await expect(page.locator('head link[rel="canonical"]')).toHaveAttribute("href", canonical);
      const schema = JSON.parse((await page.locator('script[type="application/ld+json"]').textContent())!);
      expect(schema.url).toBe(canonical);
      const missingResponse = await request.get(missingPath);
      expect(missingResponse.status()).toBe(404);
      const missingHtml = await missingResponse.text();
      const missingHead = missingHtml.split("</head>")[0];
      expect(missingHead).toContain("noindex");
      expect(missingHead).not.toContain('name="description"');
      expect(missingHead).not.toContain('rel="canonical"');
      expect(missingHead).not.toContain("Missing-seo-record");
      expect(missingHtml).not.toContain("application/ld+json");
      await page.evaluate((href) => {
        const link = document.createElement("a");
        link.href = href;
        link.textContent = "SEO navigation probe";
        link.id = "seo-navigation-probe";
        document.body.append(link);
      }, missingPath);
      await page.locator("#seo-navigation-probe").click();
      await expect(page).toHaveURL(new RegExp(`${missingPath}$`));
      await expect(page.getByRole("heading", { name: "Not Found", exact: true })).toBeVisible();
      await expect(page).toHaveTitle("Not Found | Exodata");
      await expect(page.locator('head meta[name="robots"]')).toHaveAttribute("content", "noindex");
      await expect(page.locator('head meta[name="description"]')).toHaveCount(0);
      await expect(page.locator('head link[rel="canonical"]')).toHaveCount(0);
      await expect(page.locator('script[type="application/ld+json"]')).toHaveCount(0);
      await page.evaluate((href) => {
        const link = document.getElementById("seo-navigation-probe") as HTMLAnchorElement;
        link.href = href;
      }, validPath);
      await page.locator("#seo-navigation-probe").click();
      await expect(page).toHaveURL(new RegExp(`${prefix}/${entity}/`));
      await expect(page.locator('head meta[name="description"]')).toHaveCount(1);
      await expect(page.locator('head link[rel="canonical"]')).toHaveCount(1);
      await expect(page.locator('head meta[name="robots"]')).toHaveCount(0);
      await expect(page).toHaveTitle(title);
      await expect(page.locator('script[type="application/ld+json"]')).toHaveCount(1);
      await page.evaluate(() => document.getElementById("seo-navigation-probe")?.remove());
    }
  }
  await expectNoClientErrors(page, capture);
});

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

test("SEO detail HTTP variants and sitemap URLs agree", async ({ request }) => {
  const names: Record<string, string> = {
    stellarhosts: await firstCatalogValue(request, "/rest/stellarhosts", "hostname"),
    exoplanets: await firstCatalogValue(request, "/rest/exoplanets", "pl_name"),
  };
  const index = await (await request.get("/sitemap-index.xml")).text();
  const shardPaths = [...index.matchAll(/<loc>([^<]+)<\/loc>/g)]
    .map((match) => new URL(match[1]).pathname)
    .filter((path) => /sitemap-(stellarhosts|exoplanets)-/.test(path));
  const sitemap = (await Promise.all(shardPaths.map(async (path) => (await request.get(path)).text()))).join("\n");
  for (const [entity, name] of Object.entries(names)) {
    const encoded = encodeURIComponent(name).replace(/[!'()*]/g, (character) => `%${character.charCodeAt(0).toString(16).toUpperCase()}`);
    expect(sitemap).toContain(`<loc>https://exodata.space/${entity}/${encoded}</loc>`);
    for (const prefix of ["", "/zh-CN", "/ja"]) {
      const path = `${prefix}/${entity}/${encoded}`;
      for (const method of ["GET", "HEAD"]) {
        const redirect = await request.fetch(`${path}/?filter=a%2Bb`, { method, maxRedirects: 0 });
        expect(redirect.status()).toBe(301);
        expect(redirect.headers().location).toBe(`${path}?filter=a%2Bb`);
      }
      const legacy = encoded.replace(/-/g, "%2D");
      const valid = await request.get(`${prefix}/${entity}/${legacy}`);
      expect(valid.status()).toBe(200);
      const html = await valid.text();
      expect(html).toContain(`href="https://exodata.space/${entity}/${encoded}"`);
      for (const suffix of ["%ZZ", "%FF"]) {
        expect((await request.get(`${prefix}/${entity}/${suffix}`)).status()).toBe(400);
      }
      for (const suffix of ["%3Cscript%3E", "Missing-seo+record", "Missing-seo-record/extra"]) {
        expect((await request.get(`${prefix}/${entity}/${suffix}`)).status()).toBe(404);
      }
    }
  }
});

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

test("exoplanets date sort defaults survive overrides and hidden columns", async ({
  page,
  request,
}) => {
  const capture = captureClientErrors(page);
  const response = await request.get(
    "/rest/exoplanets?columns=pl_name,rowupdate&sort_by=rowupdate&order=desc&limit=50",
  );
  expect(response.ok()).toBeTruthy();
  const payload = await response.json();
  const expectedDates = payload.data.map((row: { rowupdate: string }) => row.rowupdate);
  expect(expectedDates.length).toBeGreaterThan(0);

  const ssr = await request.get("/exoplanets");
  expect(ssr.status()).toBe(200);
  const html = await ssr.text();
  expect(html).toContain("Updated");
  expect(html).toContain(expectedDates[0]);

  await page.goto("/");
  await page.locator('nav a[href="/exoplanets"]').first().click();
  const updated = page.getByRole("columnheader", { name: /Updated/ });
  await expect(updated).toContainText("↓");
  await expect.poll(async () => page.locator("table tbody tr td:last-child").allTextContents())
    .toEqual(expectedDates);
  const defaultNames = await page.locator("table tbody tr td:first-child").allTextContents();

  await updated.click();
  await expect(page).toHaveURL(/sort=rowupdate&order=asc/);
  await expect(updated).toContainText("↑");
  await page.goBack();
  await expect(updated).toContainText("↓");
  await expect.poll(async () => page.locator("table tbody tr td:first-child").allTextContents())
    .toEqual(defaultNames);

  const name = page.locator("table thead th").first();
  await name.click();
  await expect(page).toHaveURL(/sort=pl_name&order=asc/);
  await name.click();
  await expect(page).toHaveURL(/sort=pl_name&order=desc/);
  await name.click();
  await expect(page).toHaveURL(/sort=rowupdate&order=desc/);
  await expect(updated).toContainText("↓");

  await page.goto("/exoplanets?columns=pl_name");
  await expect(page.locator("table thead tr:first-child th")).toHaveCount(1);
  await expect(page.getByText("Sorted by Updated (newest first)")).toBeVisible();
  await expect.poll(async () => page.locator("table tbody tr td:first-child").allTextContents())
    .toEqual(defaultNames);
  await page.reload();
  await expect(page.getByText("Sorted by Updated (newest first)")).toBeVisible();

  await page.getByRole("button", { name: "Next" }).last().click();
  await expect(page).toHaveURL(/page=2&sort=rowupdate&order=desc&columns=pl_name/);
  await page.reload();
  await expect(page.getByText("Sorted by Updated (newest first)")).toBeVisible();

  await page.goto("/exoplanets?filter=Kepler");
  await expect(updated).toContainText("↓");
  await expect(page.locator("table tbody tr").first()).toContainText("Kepler");
  await page.goto("/exoplanets?sort=disc_year&order=asc");
  await expect(page.getByRole("columnheader", { name: /Disc\. year/ })).toContainText("↑");
  await page.reload();
  await expect(page.getByRole("columnheader", { name: /Disc\. year/ })).toContainText("↑");
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
