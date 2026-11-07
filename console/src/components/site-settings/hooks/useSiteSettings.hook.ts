import { useSignal } from "@preact/signals";
import { useParams } from "wouter-preact";
import type {
  ImageFieldModel,
  ImageKey,
  MediaChoice,
  MediaPickerModel,
  SiteForm,
  SiteTextKey,
} from "../../../types/siteSettings";
import type { FieldErrors } from "../../../utils/fieldErrors";
import { MAX_LINKS, emptySiteForm, toColorInputValue } from "../../../utils/siteForm";
import type { SiteSettingsContextValue } from "../context/SiteSettingsContext";
import { useAnnouncement } from "./useAnnouncement.hook";
import { useAppearanceOptions } from "./useAppearanceOptions.hook";
import { useMediaPicker } from "./useMediaPicker.hook";
import { useSiteFormState } from "./useSiteFormState.hook";
import { useSitePersistence } from "./useSitePersistence.hook";
import { useSocialLinks } from "./useSocialLinks.hook";

type Data = SiteSettingsContextValue["data"];
type FormData = Pick<
  Data,
  "form" | "errors" | "picker" | "colorValue" | "images" | "linksTitle" | "canAddLink"
>;

const IMAGE_LABELS: Record<ImageKey, string> = {
  logo: "Logo",
  favicon: "Favicon",
  og_image: "Default image",
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

const imageField = (
  name: ImageKey,
  form: SiteForm,
  errors: FieldErrors,
  setters: Record<SiteTextKey, (value: string) => void>,
  choices: readonly MediaChoice[],
): ImageFieldModel => ({
  name,
  label: IMAGE_LABELS[name],
  value: form[name],
  thumbnail: thumbnailFor(form[name], choices),
  error: errors[`brand.${name}`],
  setValue: setters[name],
});

/** Describes the form values that come straight from the form, its errors, and the picker. */
const formData = (
  form: SiteForm,
  errors: FieldErrors,
  setters: Record<SiteTextKey, (value: string) => void>,
  picker: MediaPickerModel,
): FormData => ({
  form,
  errors,
  picker,
  colorValue: toColorInputValue(form.accent),
  images: {
    logo: imageField("logo", form, errors, setters, picker.items),
    favicon: imageField("favicon", form, errors, setters, picker.items),
    og_image: imageField("og_image", form, errors, setters, picker.items),
  },
  linksTitle: `Header links (${form.links.length}/${MAX_LINKS})`,
  canAddLink: form.links.length < MAX_LINKS,
});

export const useSiteSettings = (): SiteSettingsContextValue => {
  const host = useParams<{ host?: string }>().host ?? null;
  const form = useSignal<SiteForm>(emptySiteForm());
  const errors = useSignal<FieldErrors>({});
  const { update, actions, ...state } = useSiteFormState(form, errors);
  const appearance = useAppearanceOptions(update);
  const social = useSocialLinks(form, errors.value, update);
  const announcement = useAnnouncement(form, update);
  const persistence = useSitePersistence(host, form, errors);
  const picker = useMediaPicker(host, (key, path): void => actions.text[key](path));
  return {
    data: {
      ...formData(form.value, errors.value, actions.text, picker),
      ...state,
      mode: host === null ? "create" : "edit",
      options: appearance.options,
      social: social.social,
      announcement: announcement.announcement,
    },
    status: { load: persistence.load, saving: persistence.saving },
    actions: {
      ...actions,
      choose: appearance.choose,
      addSocial: social.addSocial,
      announcement: announcement.actions,
      save: persistence.save,
      remove: persistence.remove,
    },
  };
};
