import { describe, expect, it } from "vitest";
import type { FontItem } from "../types/api/FontItem";
import { MISSING_FAMILY, fontChoice, fontPatch, libraryFamilies } from "../utils/fontOptions";

const font = (name: string, families: string[]): FontItem => ({
  name,
  families,
  source: "upload",
  size: 120_000,
  created_at: 1_700_000_000,
});

const library = (): FontItem[] => [
  font("Source-Serif-4-Regular-1a2b3c4d.ttf", ["Source Serif 4"]),
  font("Inter-Bold-5e6f7a8b.ttf", ["Inter", "Inter Display"]),
  font("Inter-Regular-9c0d1e2f.ttf", ["Inter"]),
];

const labels = (fonts: FontItem[] | null, family: string | null): string[] =>
  fontChoice(fonts, "serif", family).items.map((item) => item.label);

describe("font options", () => {
  it("selects a system option when the site has no font family", () => {
    const choice = fontChoice(library(), "sans", null);
    expect(choice.items.slice(0, 2).map((item) => item.value)).toEqual([
      "system:serif",
      "system:sans",
    ]);
    expect(choice.value).toBe("system:sans");
    expect(choice.usesFamily).toBe(false);
  });

  it("lists each library family once, sorted", () => {
    expect(libraryFamilies(library())).toEqual(["Inter", "Inter Display", "Source Serif 4"]);
    expect(labels(library(), null)).toEqual([
      "Serif (system)",
      "Sans (system)",
      "Inter",
      "Inter Display",
      "Source Serif 4",
    ]);
  });

  it("selects a library family and shows the fallback choice", () => {
    const choice = fontChoice(library(), "serif", "Source Serif 4");
    const selected = choice.items.find((item) => item.value === choice.value);
    expect(selected?.label).toBe("Source Serif 4");
    expect(choice.usesFamily).toBe(true);
    expect(choice.items.some((item) => item.description === MISSING_FAMILY)).toBe(false);
  });

  it("keeps a family that is missing from the library, with a note", () => {
    const choice = fontChoice(library(), "serif", "Harbor Grotesk");
    const selected = choice.items.find((item) => item.value === choice.value);
    expect(selected?.label).toBe("Harbor Grotesk");
    expect(selected?.description).toBe(MISSING_FAMILY);
    expect(choice.items.at(-1)).toBe(selected);
  });

  it("does not mark the stored family as missing while the library loads", () => {
    const choice = fontChoice(null, "serif", "Source Serif 4");
    const selected = choice.items.find((item) => item.value === choice.value);
    expect(labels(null, "Source Serif 4")).toEqual([
      "Serif (system)",
      "Sans (system)",
      "Source Serif 4",
    ]);
    expect(selected?.description).not.toBe(MISSING_FAMILY);
  });

  it("keeps a family value apart from the system options", () => {
    const choice = fontChoice([font("Serif-0a1b2c3d.ttf", ["system:serif"])], "serif", null);
    const values = choice.items.map((item) => item.value);
    expect(new Set(values).size).toBe(values.length);
  });
});

describe("font patch", () => {
  it("clears the family and sets the stack for a system option", () => {
    expect(fontPatch("system:sans", "serif")).toEqual({ font_family: null, font: "sans" });
    expect(fontPatch("system:serif", "sans")).toEqual({ font_family: null, font: "serif" });
  });

  it("sets the family and keeps the fallback for a library family", () => {
    const choice = fontChoice(library(), "sans", "Inter Display");
    expect(fontPatch(choice.value, "sans")).toEqual({ font_family: "Inter Display", font: "sans" });
  });

  it("reads back a family name with special characters", () => {
    const choice = fontChoice(null, "serif", "Mara's Serif: Text");
    expect(fontPatch(choice.value, "serif")).toEqual({
      font_family: "Mara's Serif: Text",
      font: "serif",
    });
  });
});
