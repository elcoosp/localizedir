import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import linksValidator from "starlight-links-validator";
import imageZoom from "starlight-image-zoom";

export default await defineDocsConfig({
  llmsTxt: {},
  changelog: { repo: 'elcoosp/localizedir' },
  site: 'https://localizedir.vercel.app',
  domain: 'localizedir.vercel.app',
  title: "localizedir",
  description: "Compile-time localized directory inclusion for Rust: embed your localized directories once and generate locale-aware accessors.",
  sidebar: [],
  extraStarlight: {
                        expressiveCode: {
                        	themes: ['github-light', 'github-dark']
                        },
                        title: "localizedir",
                        description:
                                "Compile-time localized directory inclusion for Rust: embed your localized directories once and generate locale-aware accessors.",
                        favicon: '/favicon.png',
                        social: [
                                {
                                        icon: "github",
                                        label: "GitHub",
                                        href: "https://github.com/elcoosp/localizedir"
                                },
                        ],

                        plugins: [linksValidator(), imageZoom()],
                        sidebar: [
                                {
                                        label: "Tutorials",
                                        items: [{ label: "Getting started", slug: "tutorials/getting-started" }]
                                },
                                {
                                        label: "How-to guides",
                                        items: [
                                                { label: "Organize your directories", slug: "guides/organize-your-directories" },
                                                { label: "Control directory names", slug: "guides/control-directory-names" },
                                                { label: "Handle unknown locales", slug: "guides/handle-unknown-locales" },
                                                { label: "Fix compile errors", slug: "guides/fix-compile-errors" },
                                        ]
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
                                        ]
                                },
                                {
                                        label: "Explanation",
                                        items: [
                                                { label: "How it works", slug: "explanation/how-it-works" },
                                                { label: "Design decisions", slug: "explanation/design-decisions" },
                                        ]
                                },
                        ]
                },
});
