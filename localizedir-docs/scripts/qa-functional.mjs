// Functional probes: search modal, pagination links, TOC, and a one-shot
// screenshot of the 404 page + search results overlay.
import { chromium } from "playwright";

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await ctx.newPage();
const out = {};

// --- Search: open modal, type a domain term, count results ---
await page.goto("http://localhost:4321/", { waitUntil: "networkidle" });
await page.keyboard.press("Control+k");
await page.waitForTimeout(400);
const dialogVisible = await page.locator("[role='dialog'], .search-panel, starlight-menu-button").count();
await page.keyboard.type("fallback");
await page.waitForTimeout(900);
out.searchResults = await page.evaluate(() => {
	const items = document.querySelectorAll(".search-panel .result, [role='listbox'] li, .pagefind-ui__result");
	return { count: items.length, texts: [...items].slice(0, 3).map((i) => i.textContent.trim().slice(0, 60)) };
});
await page.screenshot({ path: "/tmp/render/search-modal.png" });

// --- Pagination on the last sidebar page ---
await page.goto("http://localhost:4321/explanation/design-decisions/", { waitUntil: "networkidle" });
out.pagination = await page.evaluate(() => {
	const links = [...document.querySelectorAll(".pagination-links a, [class*='pagination'] a")];
	return links.map((a) => ({ text: a.textContent.trim(), href: a.getAttribute("href") }));
});

// --- TOC present on long page ---
await page.goto("http://localhost:4321/reference/localize-dir-trait/", { waitUntil: "networkidle" });
out.toc = await page.evaluate(() => {
	const toc = document.querySelectorAll("nav .right-sidebar various, starlight-toc a, .right-sidebar a");
	return { links: toc.length };
});

// --- 404 page: status + render ---
const resp = await page.goto("http://localhost:4321/definitely-missing-page/", { waitUntil: "networkidle" });
out.notFoundStatus = resp.status();
out.notFoundH1 = await page.evaluate(() => document.querySelector("h1")?.textContent?.trim() ?? "none");
await page.screenshot({ path: "/tmp/render/notfound.png", fullPage: true });

console.log(JSON.stringify(out, null, 2));
await browser.close();
