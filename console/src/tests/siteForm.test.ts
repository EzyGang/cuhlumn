import { describe, expect, it } from "vitest";
import type { Site } from "../types/api/Site";
import {
  emptySiteForm,
  formToSiteInput,
  refineFieldErrors,
  siteFieldPaths,
  siteToForm,
} from "../utils/siteForm";

const STARTS_AT = Math.floor(new Date(2026, 10, 9, 9, 30).getTime() / 1000);

const harbor = (): Site => ({
  host: "harbor.example.com",
  title: "Harbor Notes",
  description: "Notes from the Harbor team.",
  aliases: ["www.harbor.example.com"],
  author: "Mara Lind",
  author_url: null,
  project_name: "Harbor",
  project_url: "https://harbor.example.com/app",
  cta: "",
  show_cta: false,
  lang: "en",
  home: null,
  indexnow_key: "harbor-key-0001",
  indexnow: false,
  brand: {
    accent: "#0e7490",
    font: "sans",
    logo: "/media/logo.png",
    favicon: null,
    og_image: null,
    links: [{ label: "Status", url: "https://status.harbor.example.com" }],
    head: "",
    footer: "Harbor Notes",
    credit: false,
    palette: "slate",
    theme: "dark",
    theme_toggle: false,
    width: "wide",
    home_layout: "cards",
    social: [
      { network: "github", url: "https://github.com/harbor" },
      { network: "email", url: "mailto:mara@harbor.example.com" },
    ],
    social_in_header: true,
    code_theme: "solarized",
    card_style: "accent",
    card_meta: false,
  },
  announcement: {
    text: "How we back up SQLite",
    link: "/sqlite-backups",
    link_label: "Read the post",
    tone: "accent",
    starts_at: STARTS_AT,
    ends_at: null,
    dismissible: false,
  },
  updated_at: 1_700_000_000,
});

describe("site form mapping", () => {
  it("keeps every brand, IndexNow, and announcement value from site to input", () => {
    const site = harbor();
    const input = formToSiteInput(siteToForm(site));
    expect(input.brand).toEqual(site.brand);
    expect(input.indexnow).toBe(false);
    expect(input.show_cta).toBe(false);
    expect(input.announcement).toEqual(site.announcement);
  });

  it("sends no announcement when the bar is off", () => {
    const form = siteToForm(harbor());
    const input = formToSiteInput({
      ...form,
      announcement: { ...form.announcement, enabled: false },
    });
    expect(input.announcement).toBeNull();
  });

  it("reads a site without an announcement as a disabled bar", () => {
    const form = siteToForm({ ...harbor(), announcement: null });
    expect(form.announcement.enabled).toBe(false);
    expect(formToSiteInput(form).announcement).toBeNull();
  });

  it("trims addresses and sends a blank link as null", () => {
    const form = siteToForm(harbor());
    const input = formToSiteInput({
      ...form,
      social: [{ network: "github", url: "  https://github.com/harbor  " }],
      announcement: { ...form.announcement, link: "   ", link_label: " Read " },
    });
    expect(input.brand.social).toEqual([{ network: "github", url: "https://github.com/harbor" }]);
    expect(input.announcement?.link).toBeNull();
    expect(input.announcement?.link_label).toBe("Read");
  });

  it("starts a new site with the defaults of the server", () => {
    const input = formToSiteInput(emptySiteForm());
    expect(input.indexnow).toBe(true);
    expect(input.show_cta).toBe(true);
    expect(input.announcement).toBeNull();
    expect(input.brand).toMatchObject({
      palette: "paper",
      theme: "system",
      theme_toggle: true,
      width: "normal",
      home_layout: "list",
      social: [],
      social_in_header: false,
      code_theme: "github",
      card_style: "dark",
      card_meta: true,
    });
  });
});

describe("site field errors", () => {
  const known = siteFieldPaths(siteToForm(harbor()));

  it("knows the address of each profile link", () => {
    expect(known).toContain("brand.social.1");
    expect(known).toContain("brand.social.1.url");
    expect(known).not.toContain("brand.social.2");
  });

  it("moves an announcement error to the child field that its message names", () => {
    const refined = refineFieldErrors({ announcement: "text: the text is empty" }, known);
    expect(refined).toEqual({ "announcement.text": "the text is empty" });
  });

  it("moves an end time error to the end field", () => {
    const message = "ends_at: the end time must be after the start time";
    expect(refineFieldErrors({ announcement: message }, known)).toEqual({
      "announcement.ends_at": "the end time must be after the start time",
    });
  });

  it("keeps an error when the message does not name a known child", () => {
    const errors = { announcement: "unknown field `color`", "brand.social.1": "url: bad" };
    expect(refineFieldErrors(errors, known)).toEqual({
      announcement: "unknown field `color`",
      "brand.social.1.url": "bad",
    });
  });
});
