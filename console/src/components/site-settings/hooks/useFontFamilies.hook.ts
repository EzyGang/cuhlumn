import { useLoad } from "../../../hooks/useLoad.hook";
import { listFonts } from "../../../services/fonts.service";
import type { FontModel, FontValue } from "../../../types/fonts";
import type { SiteForm, SiteFormUpdate } from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { fontChoice, fontPatch } from "../../../utils/fontOptions";

const FAILED_NOTE = "Cannot load the font library. The list shows only the known fonts.";

/** Loads the font library, and returns the Font control with its setter. */
export const useFontFamilies = (
  form: SiteForm,
  errors: FieldErrors,
  update: SiteFormUpdate,
): { font: FontModel; chooseFont: (value: FontValue) => void } => {
  const library = useLoad("fonts", listFonts);
  const fonts = library.status === "ready" ? library.value : null;
  return {
    font: {
      ...fontChoice(fonts, form.font, form.font_family),
      note: library.status === "failed" ? FAILED_NOTE : null,
      error: errors["brand.font_family"],
      manageHref: "/fonts",
    },
    chooseFont: (value: FontValue): void =>
      update(fontPatch(value, form.font), "brand.font_family"),
  };
};
