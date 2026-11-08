import type { Font } from "../types/api/Font";
import type { FontItem } from "../types/api/FontItem";
import type { FontChoice, FontValue } from "../types/fonts";
import type { RadioCardItem } from "../types/radioCards";

/** The brand values that a Font control value sets. */
export interface FontPatch {
  font_family: string | null;
  font: Font;
}

export const MISSING_FAMILY = "Missing from the font library";

const SYSTEM_ITEMS: readonly RadioCardItem<FontValue>[] = [
  { value: "system:serif", label: "Serif (system)", description: "A classic book face." },
  { value: "system:sans", label: "Sans (system)", description: "A clean modern face." },
];

const SYSTEM_FONTS: Record<string, Font> = { "system:serif": "serif", "system:sans": "sans" };

const FAMILY_PREFIX = "family:";

/** Encodes a family so that the value is also safe in an element ID. */
const familyValue = (family: string): FontValue => `${FAMILY_PREFIX}${encodeURIComponent(family)}`;

const familyItem = (family: string, description: string): RadioCardItem<FontValue> => ({
  value: familyValue(family),
  label: family,
  description,
});

/** Returns the unique family names of the library, sorted. */
export const libraryFamilies = (fonts: readonly FontItem[]): string[] =>
  [...new Set(fonts.flatMap((font) => font.families))].sort((a, b) => a.localeCompare(b));

/**
 * Returns the Font control items for the library, or for null while the library loads.
 * A stored family that the loaded library does not contain shows with a note.
 */
export const fontChoice = (
  fonts: readonly FontItem[] | null,
  font: Font,
  family: string | null,
): FontChoice => {
  const families = fonts === null ? [] : libraryFamilies(fonts);
  const items = [
    ...SYSTEM_ITEMS,
    ...families.map((name) => familyItem(name, "A family from the font library.")),
  ];
  if (family !== null && !families.includes(family)) {
    items.push(familyItem(family, fonts === null ? "The current font." : MISSING_FAMILY));
  }
  return {
    items,
    value: family === null ? `system:${font}` : familyValue(family),
    usesFamily: family !== null,
  };
};

/** Returns the brand values for a Font control value. A family keeps the current fallback. */
export const fontPatch = (value: FontValue, font: Font): FontPatch => {
  const system = SYSTEM_FONTS[value];
  if (system !== undefined) {
    return { font_family: null, font: system };
  }
  return { font_family: decodeURIComponent(value.slice(FAMILY_PREFIX.length)), font };
};
