// Generates dist/llms.txt (site index for AI agents) and dist/llms-full.txt
// (full text of every page) from the built output. Runs after `astro build`.
import { readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const DIST = "dist";

// Editorial order for the index; anything unlisted is appended at the end.
const ORDER = [
        "/",
        "/tutorials/getting-started/",
        "/guides/organize-your-directories/",
        "/guides/control-directory-names/",
        "/guides/handle-unknown-locales/",
        "/guides/fix-compile-errors/",
        "/reference/attribute/",
        "/reference/localize-dir-trait/",
        "/reference/naming-conventions/",
        "/reference/compile-error-catalogue/",
        "/reference/include-dir-interop/",
        "/reference/release-notes/",
        "/explanation/how-it-works/",
        "/explanation/design-decisions/",
];

async function walk(dir) {
        const entries = await readdir(dir, { withFileTypes: true });
        const files = [];
        for (const e of entries) {
                const p = path.join(dir, e.name);
                if (e.isDirectory()) files.push(...(await walk(p)));
                else if (e.name === "index.html") files.push(p);
        }
        return files;
}

function toRoute(file) {
        const rel = path.relative(DIST, path.dirname(file));
        const route = "/" + (rel === "." ? "" : rel + "/");
        // The 404 page is UI, not documentation.
        return route === "/404/" || route === "/404" ? null : route;
}

function extractTitle(html) {
        const m = html.match(/<h1[^>]*>([\s\S]*?)<\/h1>/i);
        if (m) return stripTags(m[1]).trim();
        const t = html.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
        return t ? stripTags(t[1]).trim() : "(untitled)";
}

function extractDescription(html) {
        const m = html.match(
                /<meta[^>]+name=["']description["'][^>]+content=["']([^"']+)["']/i,
        );
        return m ? m[1] : "";
}

function extractMain(html) {
        const m = html.match(/<main[^>]*>([\s\S]*?)<\/main>/i);
        if (!m) return "";
        let text = m[1]
                .replace(/<(script|style|nav|aside)[\s\S]*?<\/\1>/gi, " ")
                .replace(/<\/(p|h1|h2|h3|h4|h5|h6|li|tr|div|pre|blockquote)>/gi, "\n")
                .replace(/<br\s*\/?>/gi, "\n")
                .replace(/<[^>]+>/g, " ")
                .replace(/&amp;/g, "&")
                .replace(/&lt;/g, "<")
                .replace(/&gt;/g, ">")
                .replace(/&quot;/g, '"')
                .replace(/&#39;/g, "'")
                .replace(/[ \t]+/g, " ")
                .replace(/\n{3,}/g, "\n\n");
        return text.trim();
}

function stripTags(html) {
        return html.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim();
}

const pages = [];
for (const file of await walk(DIST)) {
        const route = toRoute(file);
        if (!route) continue;
        const html = await readFile(file, "utf8");
        pages.push({
                route,
                title: extractTitle(html),
                description: extractDescription(html),
                text: extractMain(html),
        });
}

pages.sort(
        (a, b) =>
                (ORDER.indexOf(a.route) === -1 ? 999 : ORDER.indexOf(a.route)) -
                (ORDER.indexOf(b.route) === -1 ? 999 : ORDER.indexOf(b.route)),
);

const base = process.env.SITE_BASE_URL ?? "https://localizedir.example.com";
const lines = [
        "# localizedir",
        "",
        "> Compile-time localized directory inclusion for Rust: one attribute embeds a locale directory tree once and generates locale-aware, non-panicking accessors. This site documents the attribute options, the LocalizeDir trait, naming conventions, the compile-error catalogue, and include_dir interop.",
        "",
        "Documentation follows the Diátaxis framework: tutorials, how-to guides, reference, and explanation.",
        "",
];
for (const p of pages) {
        lines.push(`- [${p.title}](${base}${p.route}): ${p.description}`);
}
lines.push("", "Optional", "", "- [Full documentation text](llms-full.txt): every page, concatenated, for agents that prefer one document.", "");
await writeFile(path.join(DIST, "llms.txt"), lines.join("\n"));

const full = [
        "# localizedir — full documentation",
        "",
        "> Compile-time localized directory inclusion for Rust. Generated from the documentation site; each section below is one page.",
        "",
];
for (const p of pages) {
        full.push(`## ${p.title}`, "", `URL: ${base}${p.route}`, "", p.text, "", "---", "");
}
await writeFile(path.join(DIST, "llms-full.txt"), full.join("\n"));

console.log(`llms.txt: ${pages.length} pages indexed`);
