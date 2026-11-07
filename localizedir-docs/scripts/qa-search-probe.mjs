// Search results probe v2 — inspect light DOM after typing.
import { chromium } from "playwright";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto("http://localhost:4321/", { waitUntil: "networkidle" });
await page.getByRole("button", { name: /search/i }).first().click();
await page.waitForTimeout(400);
await page.keyboard.type("fallback", { delay: 40 });
await page.waitForTimeout(1500);

const probe = await page.evaluate(() => {
	const hits = [...document.querySelectorAll("body *")]
		.filter((el) => (el.textContent ?? "").toLowerCase().includes("getting started") && el.children.length === 0)
		.map((el) => el.tagName + "." + el.className).slice(0, 5);
	const dialogs = [...document.querySelectorAll("[role='dialog'], dialog, .search-panel")].map(
		(el) => el.tagName + "." + (typeof el.className === "string" ? el.className : ""),
	);
	const pagefindEls = [...document.querySelectorAll("[class*='pagefind'], [data-pagefind]")].map(
		(el) => el.tagName + "." + el.className,
	);
	const bodyText = document.body.innerText;
	return {
		dialogs,
		pagefindEls: pagefindEls.slice(0, 6),
		sampleHits: hits,
		hasResultsWord: bodyText.includes("fallback"),
		snippet: bodyText.split("\n").filter((l) => l.toLowerCase().includes("fallback")).slice(0, 6),
	};
});
console.log(JSON.stringify(probe, null, 2));
await page.screenshot({ path: "/tmp/render/search-typed.png" });
await browser.close();
