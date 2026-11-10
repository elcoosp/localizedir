// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import astroTakumi from 'astro-takumi';
import fs from 'node:fs';
import { makeRender } from './src/og-render.mjs';
import linksValidator from "starlight-links-validator";
import imageZoom from "starlight-image-zoom";

const __logoB64 = fs.readFileSync("./src/assets/logo.png").toString("base64");
const __render = makeRender({
  logoDataUrl: "data:image/png;base64," + __logoB64,
  accentHex: "#a27d11",
  accentHexA12: "#a27d111a",
  domain: "localizedir.vercel.app",
  projectName: "localizedir",
});

export default defineConfig({
	vite: {
		ssr: { external: ["jsdom", "astro-takumi"] },
	},
        // Canonical site URL. GitHub Pages is the convention for this repository;
        // update this value (and add `base` if needed) when the production host
        // is chosen. Required so Starlight's injected sitemap integration works.
        site: 'https://localizedir.vercel.app',
        integrations: [
                starlight({
			routeMiddleware: "./src/routeData.ts",
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark'],
                        },
                        title: "localizedir",
                        description:
                                "Compile-time localized directory inclusion for Rust: embed your localized directories once and generate locale-aware accessors.",
                        favicon: '/favicon.png',

                        logo: { src: './src/assets/logo.png' },
                        customCss: ["./src/styles/custom.css"],
                        social: [
                                {
                                        icon: "github",
                                        label: "GitHub",
                                        href: "https://github.com/elcoosp/localizedir",
                                },
                        ],
                        defaultLocale: "root",
                        locales: { root: { label: "English", lang: "en" } },
                        plugins: [linksValidator(), imageZoom()],
                        sidebar: [
                                {
                                        label: "Tutorials",
                                        items: [{ label: "Getting started", slug: "tutorials/getting-started" }],
                                },
                                {
                                        label: "How-to guides",
                                        items: [
                                                { label: "Organize your directories", slug: "guides/organize-your-directories" },
                                                { label: "Control directory names", slug: "guides/control-directory-names" },
                                                { label: "Handle unknown locales", slug: "guides/handle-unknown-locales" },
                                                { label: "Fix compile errors", slug: "guides/fix-compile-errors" },
                                        ],
                                },
                                {
                                        label: "Reference",
                                        items: [
                                                { label: "Attribute options", slug: "reference/attribute" },
                                                { label: "The LocalizeDir trait", slug: "reference/localize-dir-trait" },
                                                { label: "Naming conventions", slug: "reference/naming-conventions" },
                                                { label: "Compile error catalogue", slug: "reference/compile-error-catalogue" },
                                                { label: "include_dir interop", slug: "reference/include-dir-interop" },
                                                { label: "Release notes", slug: "reference/release-notes" },
                                        ],
                                },
                                {
                                        label: "Explanation",
                                        items: [
                                                { label: "How it works", slug: "explanation/how-it-works" },
                                                { label: "Design decisions", slug: "explanation/design-decisions" },
                                        ],
                                },
                        ],
                }),
        
		astroTakumi({
			options: {
				domBackend: "linkedom",
				fonts: [fs.readFileSync("./node_modules/@fontsource/roboto/files/roboto-latin-400-normal.woff2")],
				width: 1200,
				height: 630,
				format: "png",
			},
			render: __render,
		}),
	],
});
