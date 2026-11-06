import type { Site } from "../types/api/Site";
import type { SiteInput } from "../types/api/SiteInput";
import type { SiteForm } from "../types/siteSettings";

export const MAX_LINKS = 8;
export const DEFAULT_ACCENT = "#2563eb";

export const emptySiteForm = (): SiteForm => ({
  host: "",
  title: "",
  description: "",
  aliases: [],
  author: "",
  author_url: "",
  project_name: "",
  project_url: "",
  cta: "",
  lang: "en",
  home: "",
  indexnow_key: "",
  accent: DEFAULT_ACCENT,
  font: "serif",
  logo: "",
  favicon: "",
  og_image: "",
  links: [],
  head: "",
  footer: "",
  credit: true,
});

export const siteToForm = (site: Site): SiteForm => ({
  host: site.host,
  title: site.title,
  description: site.description,
  aliases: [...site.aliases],
  author: site.author,
  author_url: site.author_url ?? "",
  project_name: site.project_name,
  project_url: site.project_url ?? "",
  cta: site.cta,
  lang: site.lang,
  home: site.home ?? "",
  indexnow_key: site.indexnow_key,
  accent: site.brand.accent,
  font: site.brand.font,
  logo: site.brand.logo ?? "",
  favicon: site.brand.favicon ?? "",
  og_image: site.brand.og_image ?? "",
  links: site.brand.links.map((link) => ({ label: link.label, url: link.url })),
  head: site.brand.head,
  footer: site.brand.footer,
  credit: site.brand.credit,
});

/** Returns the trimmed text, or null for blank text, for optional API fields. */
export const optional = (value: string): string | null => {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
};

export const formToSiteInput = (form: SiteForm): SiteInput => ({
  title: form.title,
  description: form.description,
  aliases: form.aliases,
  author: form.author,
  author_url: optional(form.author_url),
  project_name: form.project_name,
  project_url: optional(form.project_url),
  cta: form.cta,
  lang: form.lang.trim(),
  home: optional(form.home),
  indexnow_key: optional(form.indexnow_key),
  brand: {
    accent: form.accent.trim(),
    font: form.font,
    logo: optional(form.logo),
    favicon: optional(form.favicon),
    og_image: optional(form.og_image),
    links: form.links.map((link) => ({ label: link.label, url: link.url.trim() })),
    head: form.head,
    footer: form.footer,
    credit: form.credit,
  },
});

/** Lists the API field paths that the site form can show an error for. */
export const siteFieldPaths = (form: SiteForm): string[] => [
  "host",
  "title",
  "description",
  "aliases",
  ...form.aliases.map((_, index) => `aliases.${index}`),
  "author",
  "author_url",
  "project_name",
  "project_url",
  "cta",
  "lang",
  "home",
  "indexnow_key",
  ...["accent", "font", "logo", "favicon", "og_image", "links", "head", "footer", "credit"].map(
    (key) => `brand.${key}`,
  ),
  ...form.links.flatMap((_, index) => [`brand.links.${index}.label`, `brand.links.${index}.url`]),
];

/** Expands an accent to the `#rrggbb` form that a color input needs. */
export const toColorInputValue = (accent: string): string => {
  const value = accent.trim().toLowerCase();
  if (/^#[0-9a-f]{6}$/.test(value)) {
    return value;
  }
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/.exec(value);
  return short === null
    ? DEFAULT_ACCENT
    : `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}`;
};
