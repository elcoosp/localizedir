import { defineDocsConfig } from '@elcoosp/docs-toolkit/config';
import imageZoom from "starlight-image-zoom";

export default await defineDocsConfig({
  plugins: [imageZoom()],
  llmsTxt: {},
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
                                        label: "GitHub", translations: { fr: "GitHub" },
                                        href: "https://github.com/elcoosp/localizedir"
                                },
                        ],

                        sidebar: [
                                {
                                        label: "Tutorials",
                                        items: [{ label: "Getting started", slug: "tutorials/getting-started" }], translations: { fr: "Commencer" }
                                },
                                {
                                        label: "How-to guides",
                                        items: [
                                                { label: "Organize your directories", translations: { fr: "Organiser vos répertoires" }, slug: "guides/organize-your-directories" },
                                                { label: "Control directory names", translations: { fr: "Contrôler les noms de répertoires" }, slug: "guides/control-directory-names" },
                                                { label: "Handle unknown locales", translations: { fr: "Gérer les locales inconnues" }, slug: "guides/handle-unknown-locales" },
                                                { label: "Fix compile errors", translations: { fr: "Corriger les erreurs de compilation" }, slug: "guides/fix-compile-errors" },
                                        ]
                                },
                                {
                                        label: "Reference",
                                        items: [
                                                { label: "Attribute options", translations: { fr: "Options d'attribut" }, slug: "reference/attribute" },
                                                { label: "The LocalizeDir trait", translations: { fr: "Le trait LocalizeDir" }, slug: "reference/localize-dir-trait" },
                                                { label: "Naming conventions", translations: { fr: "Conventions de nommage" }, slug: "reference/naming-conventions" },
                                                { label: "Compile error catalogue", translations: { fr: "Catalogue des erreurs de compilation" }, slug: "reference/compile-error-catalogue" },
                                                { label: "include_dir interop", translations: { fr: "Interopérabilité include_dir" }, slug: "reference/include-dir-interop" },
                                                { label: "Release notes", translations: { fr: "Notes de version" }, slug: "reference/release-notes" },
                                        ]
                                },
                                {
                                        label: "Explanation",
                                        items: [
                                                { label: "How it works", translations: { fr: "Fonctionnement" }, slug: "explanation/how-it-works" },
                                                { label: "Design decisions", translations: { fr: "Décisions de conception" }, slug: "explanation/design-decisions" },
                                        ]
                                },
                        ]
                },
});
