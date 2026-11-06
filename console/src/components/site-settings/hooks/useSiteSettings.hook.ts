import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import type { Font } from "../../../types/api/Font";
import type { SelectItem } from "../../../types/select";
import type {
  ImageFieldModel,
  ImageKey,
  MediaChoice,
  SiteForm,
  SiteTextKey,
} from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { MAX_LINKS, emptySiteForm, toColorInputValue } from "../../../utils/siteForm";
import type { SiteSettingsContextValue } from "../context/SiteSettingsContext";
import { useMediaPicker } from "./useMediaPicker.hook";
import { useSiteFormState } from "./useSiteFormState.hook";
import { useSitePersistence } from "./useSitePersistence.hook";

const FONTS: readonly SelectItem<Font>[] = [
  { value: "serif", label: "Serif" },
  { value: "sans", label: "Sans serif" },
];

const IMAGE_KEYS: readonly ImageKey[] = ["logo", "favicon", "og_image"];

const IMAGE_LABELS: Record<ImageKey, string> = {
  logo: "Logo",
  favicon: "Favicon",
  og_image: "Share image",
};

/**
 * Returns the address to show as a thumbnail: the library address for a `/media/` path,
 * the value itself for an absolute address, and null otherwise.
 */
const thumbnailFor = (value: string, choices: readonly MediaChoice[]): string | null => {
  const url = value.trim();
  if (url.startsWith("/media/")) {
    return choices.find((choice) => choice.path === url)?.url ?? null;
  }
  return /^https?:\/\/[^/\s]+/i.test(url) ? url : null;
};

/** Describes the image fields. */
const imageFields = (
  form: SiteForm,
  errors: FieldErrors,
  setters: Record<SiteTextKey, (value: string) => void>,
  choices: readonly MediaChoice[],
): ImageFieldModel[] =>
  IMAGE_KEYS.map((name) => ({
    name,
    label: IMAGE_LABELS[name],
    value: form[name],
    thumbnail: thumbnailFor(form[name], choices),
    error: errors[`brand.${name}`],
    setValue: setters[name],
  }));

export const useSiteSettings = (): SiteSettingsContextValue => {
  const host = useParams<{ host?: string }>().host ?? null;
  const form = useSignal<SiteForm>(emptySiteForm());
  const errors = useSignal<FieldErrors>({});
  const state = useSiteFormState(form, errors);
  const persistence = useSitePersistence(host, form, errors);
  const picker = useMediaPicker(host, (key, path): void => state.actions.text[key](path));

  return {
    data: {
      mode: host === null ? "create" : "edit",
      form: form.value,
      errors: errors.value,
      aliases: state.aliases,
      aliasError: state.aliasError,
      links: state.links,
      picker,
      colorValue: toColorInputValue(form.value.accent),
      fonts: FONTS,
      imageFields: imageFields(form.value, errors.value, state.actions.text, picker.items),
      linksTitle: `Header links (${form.value.links.length}/${MAX_LINKS})`,
      canAddLink: form.value.links.length < MAX_LINKS,
    },
    status: { load: persistence.load, saving: persistence.saving },
    actions: { ...state.actions, save: persistence.save, remove: persistence.remove },
  };
};
