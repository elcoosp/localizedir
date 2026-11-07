// Captures deviceScaleFactor=2 clips of the header (logo + title) and a
// content area, in both themes, for the 2× sharpness inspection.
import { chromium } from "playwright";

const browser = await chromium.launch();
for (const theme of ["light", "dark"]) {
	const ctx = await browser.newContext({
		viewport: { width: 1440, height: 900 },
		deviceScaleFactor: 2,
		reducedMotion: "reduce",
	});
	const page = await ctx.newPage();
	await page.addInitScript((t) => {
		try {
			localStorage.setItem("starlight-theme", t);
		} catch {}
	}, theme);
	await page.goto("http://localhost:4321/", { waitUntil: "networkidle" });
	await page.evaluate(() => document.fonts.ready);
	const header = page.locator("header");
	await header.screenshot({
		path: `.qa/logo-header--${theme}.png`,
	});
	await ctx.close();
}
await browser.close();
console.log("2x header captures written");
