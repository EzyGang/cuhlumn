import type { Brand } from "../types/api/Brand";
import type { Site } from "../types/api/Site";
import type { SiteInput } from "../types/api/SiteInput";
import type { SiteForm } from "../types/siteSettings";
import { announcementToForm, emptyAnnouncementForm, formToAnnouncement } from "./announcementForm";
import type { FieldErrors } from "./fieldErrors";

export const MAX_LINKS = 8;
export const MAX_SOCIAL = 12;
export const DEFAULT_ACCENT = "#2563eb";

type BrandForm = Pick<SiteForm, Exclude<keyof Brand, "logo" | "favicon" | "og_image">>;

/** The appearance values of a new site, equal to the defaults of the server. */
const DEFAULT_APPEARANCE: Omit<BrandForm, "accent" | "links" | "head" | "footer" | "social"> = {
  font: "serif",
  credit: true,
  palette: "paper",
  theme: "system",
  theme_toggle: true,
  width: "normal",
  home_layout: "list",
  social_in_header: false,
  code_theme: "github",
  card_style: "dark",
  card_meta: true,
};

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
  indexnow: true,
  accent: DEFAULT_ACCENT,
  logo: "",
  favicon: "",
  og_image: "",
  links: [],
  head: "",
  footer: "",
  social: [],
  ...DEFAULT_APPEARANCE,
  announcement: emptyAnnouncementForm(),
});

/** Copies the brand fields that the form keeps with the same type. */
const brandChoices = (brand: BrandForm): BrandForm => ({
  accent: brand.accent,
  font: brand.font,
  links: brand.links.map((link) => ({ label: link.label, url: link.url })),
  head: brand.head,
  footer: brand.footer,
  credit: brand.credit,
  palette: brand.palette,
  theme: brand.theme,
  theme_toggle: brand.theme_toggle,
  width: brand.width,
  home_layout: brand.home_layout,
  social: brand.social.map((link) => ({ network: link.network, url: link.url })),
  social_in_header: brand.social_in_header,
  code_theme: brand.code_theme,
  card_style: brand.card_style,
  card_meta: brand.card_meta,
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
  indexnow: site.indexnow,
  logo: site.brand.logo ?? "",
  favicon: site.brand.favicon ?? "",
  og_image: site.brand.og_image ?? "",
  ...brandChoices(site.brand),
  announcement: announcementToForm(site.announcement),
});

/** Returns the trimmed text, or null for blank text, for optional API fields. */
export const optional = (value: string): string | null => {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
};

const formToBrand = (form: SiteForm): Brand => ({
  ...brandChoices(form),
  accent: form.accent.trim(),
  logo: optional(form.logo),
  favicon: optional(form.favicon),
  og_image: optional(form.og_image),
  links: form.links.map((link) => ({ label: link.label, url: link.url.trim() })),
  social: form.social.map((link) => ({ network: link.network, url: link.url.trim() })),
});

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
  indexnow: form.indexnow,
  brand: formToBrand(form),
  announcement: formToAnnouncement(form.announcement),
});

const ANNOUNCEMENT_KEYS = ["text", "link", "link_label", "tone", "starts_at", "ends_at"];

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
  ...["accent", "logo", "favicon", "og_image", "links", "head", "footer", "social"].map(
    (key) => `brand.${key}`,
  ),
  ...form.links.flatMap((_, index) => [`brand.links.${index}.label`, `brand.links.${index}.url`]),
  ...form.social.flatMap((_, index) => [`brand.social.${index}`, `brand.social.${index}.url`]),
  "announcement",
  ...ANNOUNCEMENT_KEYS.map((key) => `announcement.${key}`),
];

/** Splits a message such as `text: is empty` into the child field name and the message. */
const CHILD_MESSAGE = /^([a-z_]+): (.+)$/s;

/**
 * Moves an error to a child field when the message starts with the name of a known child.
 * The server reports some rules on the parent object, such as `announcement` with `text: …`.
 */
export const refineFieldErrors = (errors: FieldErrors, known: readonly string[]): FieldErrors =>
  Object.fromEntries(
    Object.entries(errors).map(([path, message]): [string, string] => {
      const match = CHILD_MESSAGE.exec(message);
      const child = `${path}.${match?.[1] ?? ""}`;
      return match !== null && known.includes(child)
        ? [child, match[2] ?? message]
        : [path, message];
    }),
  );

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
